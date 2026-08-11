//! VTIMEZONE synthesis from an IANA time zone name.
//!
//! The Calendar API names a time zone and stops there: a boundary
//! carries `"timeZone": "America/New_York"`, an IANA Time Zone Database
//! name, and the resource has no slot for the observances behind it.
//! iCalendar has the opposite arrangement, RFC 5545 3.2.19 requiring
//! every `TZID` a document references to resolve to a VTIMEZONE the
//! same document carries, so a projection that emits the parameter owes
//! the component. Google's own CalDAV frontend does this expansion
//! server-side; the REST API leaves it to the caller, which is why the
//! backend carries a time zone database of its own.
//!
//! ## The database is bundled, not the host's
//!
//! jiff reads the system copy of the database by default, which is the
//! right choice for an application asking what time it is locally and
//! the wrong one here. What this module writes is the document of
//! record: two machines reading the same Google account have to produce
//! the same bytes for the same event, and a container carrying no
//! zoneinfo has to produce them at all. The `gcal` feature therefore
//! turns on `jiff/tzdb-bundle-always`, so the database ships with the
//! release and the output depends on nothing outside it.
//!
//! ## The era the item names, not the zone's whole past
//!
//! A zone's record runs to hundreds of transitions, far too many to
//! repeat on every event, so [`vtimezone`] describes one era: the one
//! the item itself falls in. The United States moved its rule in 2007,
//! so an item from 1980 described by today's rule would read an hour
//! out through the weeks between the old onset and the new one.
//!
//! An observance is therefore built from the transitions bracketing the
//! anchor, and states a yearly rule only where the transitions that
//! follow it agree on one. A rule outlives any window a list of dated
//! onsets could cover, which a recurring event needs; agreement is what
//! establishes there is a rule to state.
//!
//! ## A zone at rest is described as such
//!
//! Reading transitions around an anchor would otherwise revive a rule
//! the zone has abandoned: Hong Kong last shifted in 1979, and an item
//! from today has no business carrying that summer time. A zone whose
//! nearest shift is more than [`SETTLED`] from the anchor is described
//! the way one that never shifted is, by the single offset in force.

use ical::{
    prop::IcalPropKind,
    tree::cst::IcalCst,
    value::{IcalValue, datetime::IcalDateTime, recur::IcalRecur, utc_offset::IcalUtcOffset},
};
use jiff::{
    SignedDuration, Timestamp,
    civil::{self, Weekday},
    tz::{Dst, Offset, TimeZone, TimeZoneTransition},
};

use crate::gcal::project::{component, prop, text_prop};

/// Onset given to an observance the zone dates no better itself: one
/// that never shifts, or one at rest. Its rule has no beginning to
/// state, and the epoch is the conventional stand-in.
const EPOCH: civil::DateTime = civil::DateTime::constant(1970, 1, 1, 0, 0, 0, 0);

/// How far the nearest shift must be from the anchor before the zone
/// counts as settled. Two years clears the annual pair comfortably
/// while still catching a zone that gave daylight saving up decades
/// ago.
const SETTLED: SignedDuration = SignedDuration::from_hours(24 * 365 * 2);

/// Following transitions of one kind that must fall on the same rule
/// before the observance states one, rather than standing as a single
/// dated onset.
const AGREEING_TRANSITIONS: usize = 2;

/// Transitions read on each side of the anchor before giving up on
/// finding one of a given kind. A zone shifting at all shifts twice a
/// year, so a handful is generous, and it keeps the search over a zone
/// that never shifts bounded.
const SEARCH_SPAN: usize = 8;

/// Whether an IANA name resolves to a zone this module can rebuild.
///
/// What it guards is the projection's right to drop a VTIMEZONE on the
/// way in: one that can be rebuilt from its name is regenerated on
/// every read and need not be stashed, and one that cannot has to be
/// kept verbatim or it is gone for good.
pub fn is_known(tzid: &str) -> bool {
    TimeZone::get(tzid).is_ok()
}

/// Projects an IANA time zone name onto the VTIMEZONE component a
/// document referencing it needs, or nothing when the name is not one
/// the database knows.
///
/// `anchor` is the instant the description is built around, in Unix
/// seconds, and is normally the start of the item that names the zone.
/// A zone is only obliged to be right about the times its item can
/// reach, and anchoring keeps the component to the era in force there.
pub fn vtimezone(tzid: &str, anchor: i64) -> Option<IcalCst<'static>> {
    let zone = TimeZone::get(tzid).ok()?;
    let anchor = Timestamp::from_second(anchor).ok()?;

    let mut vtimezone = component("VTIMEZONE");
    vtimezone.push(text_prop(IcalPropKind::TzId, tzid.to_owned()));

    for observance in observances(&zone, anchor)? {
        vtimezone.push_component(observance);
    }

    Some(vtimezone)
}

/// The observances describing the zone around `anchor`.
///
/// Nothing at all comes back when none can be built. A VTIMEZONE short
/// of an observance would be worse than none, the document then
/// claiming a definition it does not carry, so the whole component is
/// dropped and the TZID goes back to standing alone.
fn observances(zone: &TimeZone, anchor: Timestamp) -> Option<Vec<IcalCst<'static>>> {
    if settled(zone, anchor) {
        let info = zone.to_offset_info(anchor);

        return Some(vec![observance(
            "STANDARD",
            info.offset(),
            info.offset(),
            info.abbreviation(),
            EPOCH,
            None,
        )]);
    }

    let mut observances = Vec::new();

    for dst in [Dst::Yes, Dst::No] {
        let Some(transition) = nearest(zone, anchor, dst) else {
            continue;
        };

        let at = transition.timestamp();
        let to = transition.offset();
        let from = offset_before(zone, at)?;

        // NOTE: some zones renumber an abbreviation without moving the
        // clock, which is no observance at all.
        if from == to {
            continue;
        }

        let onset = from.to_datetime(at);
        let name = if dst == Dst::Yes {
            "DAYLIGHT"
        } else {
            "STANDARD"
        };
        let rule = yearly_rule(zone, at, dst, onset);

        observances.push(observance(
            name,
            from,
            to,
            transition.abbreviation(),
            onset,
            rule,
        ));
    }

    (!observances.is_empty()).then_some(observances)
}

/// Whether the zone holds one offset over the years around `anchor`.
///
/// Asked of the neighbouring transitions rather than by sampling the
/// offset a year either side: a year apart lands in the same season, so
/// a zone shifting every spring would read as settled.
fn settled(zone: &TimeZone, anchor: Timestamp) -> bool {
    let recent = zone
        .preceding(anchor)
        .next()
        .is_some_and(|transition| anchor.duration_since(transition.timestamp()) < SETTLED);

    let upcoming = zone
        .following(anchor)
        .next()
        .is_some_and(|transition| transition.timestamp().duration_since(anchor) < SETTLED);

    !recent && !upcoming
}

/// The transition installing the wanted kind of offset that stands at
/// `anchor`: the latest one at or before it, or the earliest later one
/// when the zone has no history of that kind yet.
fn nearest<'t>(zone: &'t TimeZone, anchor: Timestamp, dst: Dst) -> Option<TimeZoneTransition<'t>> {
    let standing = zone
        .preceding(anchor)
        .take(SEARCH_SPAN)
        .find(|transition| transition.dst() == dst);

    standing.or_else(|| {
        zone.following(anchor)
            .take(SEARCH_SPAN)
            .find(|transition| transition.dst() == dst)
    })
}

/// The offset in force immediately before `at`.
///
/// Read a moment earlier rather than off the previous transition, so a
/// zone whose record starts at `at` still answers.
///
/// NOTE: a whole second earlier, not the nanosecond it is tempting to
/// step back. A lookup against the recorded transitions resolves at
/// second granularity, so a sub-second step lands inside the second the
/// transition happens on and answers with the offset it installed, not
/// the one it replaced. No zone shifts twice within a second, so the
/// wider step reads nothing else by mistake.
fn offset_before(zone: &TimeZone, at: Timestamp) -> Option<Offset> {
    let before = at.checked_sub(SignedDuration::from_secs(1)).ok()?;
    Some(zone.to_offset(before))
}

/// The yearly rule the onset at `at` repeats on, when the following
/// transitions of the same kind fall on the same month, week of the
/// month, weekday and local time.
///
/// Stated as a rule rather than a list of dated onsets because a
/// recurring event outlives any window a list could enumerate, and an
/// occurrence past the end of it would resolve against the last
/// observance and drift by the daylight offset.
fn yearly_rule(zone: &TimeZone, at: Timestamp, dst: Dst, onset: civil::DateTime) -> Option<String> {
    let ordinal = week_of_month(onset);

    let agreeing = zone
        .following(at)
        .filter(|transition| transition.dst() == dst)
        .take(AGREEING_TRANSITIONS)
        .filter(|transition| {
            let following = offset_before(zone, transition.timestamp())
                .map(|from| from.to_datetime(transition.timestamp()));

            following.is_some_and(|following| {
                following.month() == onset.month()
                    && following.time() == onset.time()
                    && week_of_month(following) == ordinal
            })
        })
        .count();

    (agreeing == AGREEING_TRANSITIONS).then(|| recurrence(onset.month(), ordinal, onset.weekday()))
}

/// One STANDARD or DAYLIGHT observance: the offset it leaves, the
/// offset it installs, when it takes effect and the rule it repeats on.
fn observance(
    name: &'static str,
    from: Offset,
    to: Offset,
    abbreviation: &str,
    onset: civil::DateTime,
    rule: Option<String>,
) -> IcalCst<'static> {
    let mut observance = component(name);

    if !abbreviation.is_empty() {
        observance.push(text_prop(IcalPropKind::TzName, abbreviation.to_owned()));
    }

    for (kind, offset) in [
        (IcalPropKind::TzOffsetFrom, from),
        (IcalPropKind::TzOffsetTo, to),
    ] {
        let value = IcalValue::UtcOffset(IcalUtcOffset(utc_offset(offset).into()));
        observance.push(prop(kind, value));
    }

    // NOTE: RFC 5545 3.6.5 states an observance DTSTART in the local
    // time before its transition, which is the offset the observance
    // leaves, so the onset is read in `from` and needs no shifting.
    observance.push(prop(
        IcalPropKind::DtStart,
        IcalValue::DateTime(IcalDateTime(stamp(onset).into())),
    ));

    if let Some(rule) = rule {
        observance.push(prop(
            IcalPropKind::RRule,
            IcalValue::Recur(IcalRecur(rule.into())),
        ));
    }

    observance
}

/// Which occurrence of its weekday within the month a local date-time
/// falls on, as iCalendar counts them: 1 through 4 from the start, or
/// -1 for the last whatever the month's length.
///
/// A fifth occurrence is reported as the last one, since that is what
/// it is, and the rule then stays right in the months holding only
/// four.
fn week_of_month(onset: civil::DateTime) -> i8 {
    let day = onset.day();
    let last = onset.date().last_of_month().day();

    match day + 7 > last {
        true => -1,
        false => (day - 1) / 7 + 1,
    }
}

/// A month, an occurrence within it and a weekday as an RRULE value.
fn recurrence(month: i8, ordinal: i8, weekday: Weekday) -> String {
    let weekday = match weekday {
        Weekday::Monday => "MO",
        Weekday::Tuesday => "TU",
        Weekday::Wednesday => "WE",
        Weekday::Thursday => "TH",
        Weekday::Friday => "FR",
        Weekday::Saturday => "SA",
        Weekday::Sunday => "SU",
    };

    format!("FREQ=YEARLY;BYMONTH={month};BYDAY={ordinal}{weekday}")
}

/// A local date-time as an iCalendar stamp.
fn stamp(onset: civil::DateTime) -> String {
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        onset.year(),
        onset.month(),
        onset.day(),
        onset.hour(),
        onset.minute(),
        onset.second(),
    )
}

/// An offset as the iCalendar `±HHMM(SS)` form, widened for the handful
/// of historical zones running on a whole number of neither minutes nor
/// hours.
fn utc_offset(offset: Offset) -> String {
    let total = offset.seconds();
    let sign = if total < 0 { '-' } else { '+' };
    let total = total.unsigned_abs();
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);

    match seconds {
        0 => format!("{sign}{hours:02}{minutes:02}"),
        _ => format!("{sign}{hours:02}{minutes:02}{seconds:02}"),
    }
}

#[cfg(test)]
mod tests {
    use ical::{
        recur::IcalRecurDateTime,
        timezone::{IcalOffset, IcalTimezone},
        tree::cst::IcalCst,
    };

    use super::*;

    /// Zones spanning the shapes a rule can take: both hemispheres, a
    /// half-hour shift, a rule counted from the last week of a month
    /// rather than the first, and zones that never shift at all.
    const ZONES: &[&str] = &[
        "America/New_York",
        "America/Santiago",
        "America/Sao_Paulo",
        "Asia/Kolkata",
        "Asia/Tehran",
        "Australia/Lord_Howe",
        "Australia/Sydney",
        "Europe/Dublin",
        "Europe/Paris",
        "Pacific/Auckland",
        "Pacific/Chatham",
        "UTC",
    ];

    /// Midnight UTC on a civil date, as an anchor.
    fn at(year: i16, month: i8, day: i8) -> i64 {
        civil::date(year, month, day)
            .to_datetime(civil::Time::MIN)
            .to_zoned(TimeZone::UTC)
            .unwrap()
            .timestamp()
            .as_second()
    }

    /// A generated zone, read back through the resolver ical-rs runs on
    /// the observances alone.
    ///
    /// Nothing of this module survives the round trip but the bytes, so
    /// what the assertions weigh is the document rather than the code
    /// that wrote it.
    fn resolved(tzid: &str, local: (i16, i8, i8, i8, i8)) -> IcalOffset {
        let (year, month, day, hour, minute) = local;

        let raw = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{}END:VCALENDAR\r\n",
            vtimezone(tzid, at(year, month, day)).expect("a known zone")
        );

        let cst = IcalCst::parse(&raw).expect("parse");
        let zone = IcalTimezone::of_calendar(&cst.decode(), tzid).expect("a VTIMEZONE");

        zone.resolve(IcalRecurDateTime {
            year: i32::from(year),
            month: month.unsigned_abs(),
            day: day.unsigned_abs(),
            hour: hour.unsigned_abs(),
            minute: minute.unsigned_abs(),
            second: 0,
        })
    }

    /// A civil time under a generated zone resolves to the offset the
    /// database itself puts in force at the instant that time then
    /// names, which is the whole claim a VTIMEZONE makes.
    ///
    /// The sweep reaches back over the rule changes of the last half
    /// century, since the anchor is what selects the era described.
    #[test]
    fn a_generated_zone_answers_what_the_database_does() {
        let mut checked = 0;

        for tzid in ZONES {
            let zone = TimeZone::get(tzid).unwrap();

            for year in [1975, 1990, 2004, 2024, 2025, 2026] {
                for month in 1..=12 {
                    let local = (year, month, 15, 12, 0);

                    // NOTE: a sample the clock skips or repeats has no
                    // single answer to compare, and the two that do are
                    // pinned by their own case.
                    let Some(offset) = resolved(tzid, local).unambiguous() else {
                        continue;
                    };

                    let noon = at(year, month, 15) + 12 * 3600;
                    let instant = Timestamp::from_second(noon - i64::from(offset)).unwrap();

                    assert_eq!(
                        offset,
                        zone.to_offset(instant).seconds(),
                        "{tzid} {local:?}"
                    );
                    checked += 1;
                }
            }
        }

        assert_eq!(checked, ZONES.len() * 72);
    }

    /// The United States moved its rule in 2007, so an item from before
    /// it must not be described by the rule that replaced it: every
    /// occurrence in the weeks between the two onsets would read an
    /// hour out.
    #[test]
    fn the_observances_describe_the_era_the_item_names() {
        let modern = vtimezone("America/New_York", at(2024, 7, 14))
            .unwrap()
            .to_string();
        assert!(modern.contains("BYMONTH=3;BYDAY=2SU"), "{modern}");
        assert!(modern.contains("BYMONTH=11;BYDAY=1SU"), "{modern}");

        let historical = vtimezone("America/New_York", at(1980, 7, 14))
            .unwrap()
            .to_string();
        assert!(historical.contains("BYMONTH=4;BYDAY=-1SU"), "{historical}");
        assert!(historical.contains("BYMONTH=10;BYDAY=-1SU"), "{historical}");
    }

    /// Hong Kong last shifted in 1979, so an item from today must not
    /// carry that observance: the offset has been +0800 throughout
    /// living memory. While it was still shifting, it is described as
    /// shifting.
    #[test]
    fn a_zone_that_gave_daylight_saving_up_is_not_described_by_its_ghost() {
        let settled = vtimezone("Asia/Hong_Kong", at(2026, 8, 14))
            .unwrap()
            .to_string();
        assert!(!settled.contains("DAYLIGHT"), "{settled}");
        assert!(!settled.contains("RRULE"), "{settled}");
        assert!(settled.contains("TZOFFSETTO:+0800"), "{settled}");

        let shifting = vtimezone("Asia/Hong_Kong", at(1978, 8, 14))
            .unwrap()
            .to_string();
        assert!(shifting.contains("DAYLIGHT"), "{shifting}");
    }

    /// 2024-03-10T02:30 never happens in New York, and 2024-11-03T01:30
    /// happens twice. A zone carrying only its TZID could answer
    /// neither.
    #[test]
    fn the_two_local_times_that_are_not_one_instant_are_reported_as_such() {
        assert_eq!(
            resolved("America/New_York", (2024, 3, 10, 2, 30)),
            IcalOffset::Gap {
                before: -18000,
                after: -14400
            }
        );
        assert_eq!(
            resolved("America/New_York", (2024, 11, 3, 1, 30)),
            IcalOffset::Fold {
                earlier: -14400,
                later: -18000
            }
        );
    }

    /// Lord Howe shifts by thirty minutes rather than an hour, so its
    /// offsets need the minutes field to say anything at all, and
    /// Kolkata never shifts, so it states one observance and no rule. A
    /// name Google could never send resolves to nothing at all, rather
    /// than to a zone made up on the spot.
    #[test]
    fn a_half_hour_shift_and_a_fixed_zone_keep_their_shapes() {
        let anchor = at(2026, 8, 14);

        let lord_howe = vtimezone("Australia/Lord_Howe", anchor)
            .unwrap()
            .to_string();
        assert!(lord_howe.contains("TZOFFSETFROM:+1030\r\n"), "{lord_howe}");
        assert!(lord_howe.contains("TZOFFSETTO:+1100\r\n"), "{lord_howe}");

        let kolkata = vtimezone("Asia/Kolkata", anchor).unwrap().to_string();
        assert!(kolkata.contains("TZOFFSETTO:+0530\r\n"), "{kolkata}");
        assert!(!kolkata.contains("RRULE"), "{kolkata}");
        assert_eq!(kolkata.matches("BEGIN:STANDARD").count(), 1);

        assert!(vtimezone("Custom/Zone", anchor).is_none());
        assert!(!is_known("Custom/Zone"));
    }

    /// Zones did run on second-accurate offsets before the war, and
    /// RFC 5545 3.3.14 keeps room for them.
    #[test]
    fn an_offset_states_the_seconds_only_when_it_has_any() {
        let offset = |seconds| Offset::from_seconds(seconds).unwrap();

        assert_eq!(utc_offset(offset(0)), "+0000");
        assert_eq!(utc_offset(offset(-18000)), "-0500");
        assert_eq!(utc_offset(offset(19800)), "+0530");
        assert_eq!(utc_offset(offset(-177)), "-000257");
    }

    /// A fifth occurrence is the last one, and saying so keeps the rule
    /// right in the months holding only four.
    #[test]
    fn the_last_occurrence_of_a_weekday_is_counted_from_the_end() {
        let onset = |year, month, day| civil::date(year, month, day).to_datetime(civil::Time::MIN);

        assert_eq!(week_of_month(onset(2026, 3, 29)), -1);
        assert_eq!(week_of_month(onset(2024, 3, 10)), 2);
        assert_eq!(week_of_month(onset(2026, 5, 29)), -1);
        assert_eq!(week_of_month(onset(2026, 5, 22)), 4);
    }
}
