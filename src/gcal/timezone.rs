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
//! ## The era the item names, not the zone's whole past
//!
//! A zone's record runs to hundreds of transitions, far too many to
//! repeat on every event, so [`vtimezone`] describes one era: the one
//! the item itself falls in. The United States moved its rule in 2007,
//! so an item from 1980 described by today's rule would read an hour
//! out through the weeks between the old onset and the new one.
//!
//! Which half of the record answers depends on where the anchor lands.
//! A TZif record stops at the point its closing POSIX rule takes over,
//! so an anchor past that point is described by that rule, and an
//! earlier one by the transitions bracketing it. Either way the result
//! is the standard and daylight pair in force then, each as a yearly
//! rule, which is the shape Google's CalDAV frontend and every desktop
//! client emit.
//!
//! ## A zone at rest is described as such
//!
//! Reading raw transitions around an anchor would revive a rule the
//! zone has abandoned: Hong Kong last shifted in 1979, and an item from
//! today has no business carrying that summer time. A zone whose
//! nearest shift is more than [`SETTLED_SECONDS`] from the anchor is
//! therefore described the way one that never shifted is, by the single
//! offset actually in force.

use chrono::{DateTime, Datelike, Days, Duration, Months, NaiveDate, NaiveDateTime, Weekday};
use ical::{
    prop::IcalPropKind,
    tree::cst::IcalCst,
    value::{IcalValue, datetime::IcalDateTime, recur::IcalRecur, utc_offset::IcalUtcOffset},
};
use tz::timezone::{LocalTimeType, RuleDay, TimeZoneRef, Transition};

use crate::gcal::project::{component, prop, text_prop};

/// Year an observance anchors its DTSTART in when the zone offers no
/// date of its own: one that never shifted, or one whose record carries
/// no transition to date the closing rule from.
const EPOCH_YEAR: i32 = 1970;

/// How far the nearest shift must be from the anchor before the zone
/// counts as settled. Two years clears the annual pair comfortably
/// while still catching a zone that gave daylight saving up decades
/// ago.
const SETTLED_SECONDS: i64 = 2 * 365 * 24 * 3600;

/// Following transitions of one kind that must fall on the same rule
/// before the observance states one, rather than standing as a single
/// dated onset.
const AGREEING_TRANSITIONS: usize = 2;

/// Whether an IANA name resolves to a zone this module can rebuild.
///
/// What it guards is the projection's right to drop a VTIMEZONE on the
/// way in: one that can be rebuilt from its name is regenerated on
/// every read and need not be stashed, and one that cannot has to be
/// kept verbatim or it is gone for good.
pub fn is_known(tzid: &str) -> bool {
    tzdb::tz_by_name(tzid).is_some()
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
    let zone = tzdb::tz_by_name(tzid)?;

    let mut vtimezone = component("VTIMEZONE");
    vtimezone.push(text_prop(IcalPropKind::TzId, tzid.to_owned()));

    for observance in observances(zone, anchor)? {
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
fn observances(zone: TimeZoneRef<'static>, anchor: i64) -> Option<Vec<IcalCst<'static>>> {
    let last = zone
        .transitions()
        .last()
        .map(|transition| unix_time(zone, transition));

    // NOTE: a record stops where its closing POSIX rule takes over, so
    // an anchor past the last transition is the rule's business and an
    // earlier one the transitions'.
    match last {
        Some(last) if anchor <= last => recorded(zone, anchor),
        _ => closing(zone),
    }
}

/// The observances the recorded transitions around `anchor` describe.
fn recorded(zone: TimeZoneRef<'static>, anchor: i64) -> Option<Vec<IcalCst<'static>>> {
    if settled(zone, anchor) {
        let standing = zone.find_local_time_type(anchor).ok()?;
        return Some(vec![observance(
            "STANDARD",
            standing,
            standing,
            epoch()?,
            None,
        )]);
    }

    let mut observances = Vec::new();

    for daylight in [true, false] {
        let Some(index) = nearest(zone, anchor, daylight) else {
            continue;
        };

        let transition = &zone.transitions()[index];
        let at = unix_time(zone, transition);
        let to = zone
            .local_time_types()
            .get(transition.local_time_type_index())?;
        let from = zone.find_local_time_type(at - 1).ok()?;

        // NOTE: some zones renumber an abbreviation without moving the
        // clock, which is no observance at all.
        if from.ut_offset() == to.ut_offset() {
            continue;
        }

        let onset = local(at, from.ut_offset())?;
        let name = if daylight { "DAYLIGHT" } else { "STANDARD" };
        let rule = yearly_rule(zone, index, daylight, onset);

        observances.push(observance(name, from, to, onset, rule));
    }

    (!observances.is_empty()).then_some(observances)
}

/// The observances the POSIX rule closing the record describes.
///
/// The rule names both halves of the year at once, so one rule yields
/// both observances: each transition starts in the offset the other one
/// leaves behind. A zone that never shifts states one observance and no
/// rule, the offset it installs standing for good.
fn closing(zone: TimeZoneRef<'static>) -> Option<Vec<IcalCst<'static>>> {
    // NOTE: the rule took effect where the record stops, so its own
    // year dates the observances rather than the epoch: a DTSTART in
    // 1970 would claim a rule that reached back decades further.
    let year = zone
        .transitions()
        .last()
        .and_then(|transition| local(unix_time(zone, transition), 0))
        .map_or(EPOCH_YEAR, |onset| onset.year());

    match zone.extra_rule() {
        Some(tz::timezone::TransitionRule::Alternate(alternate)) => Some(vec![
            observance(
                "DAYLIGHT",
                alternate.std(),
                alternate.dst(),
                onset_of(alternate.dst_start(), alternate.dst_start_time(), year)?,
                Some(recur(alternate.dst_start())),
            ),
            observance(
                "STANDARD",
                alternate.dst(),
                alternate.std(),
                onset_of(alternate.dst_end(), alternate.dst_end_time(), year)?,
                Some(recur(alternate.dst_end())),
            ),
        ]),
        Some(tz::timezone::TransitionRule::Fixed(fixed)) => {
            Some(vec![observance("STANDARD", fixed, fixed, epoch()?, None)])
        }
        None => {
            let standing = standing(zone)?;
            Some(vec![observance(
                "STANDARD",
                standing,
                standing,
                epoch()?,
                None,
            )])
        }
    }
}

/// Whether the zone holds one offset over the years around `anchor`.
///
/// Asked of the transitions themselves rather than by sampling the
/// offset a year either side: a year apart lands in the same season, so
/// a zone shifting every spring would read as settled.
fn settled(zone: TimeZoneRef<'_>, anchor: i64) -> bool {
    zone.transitions()
        .iter()
        .all(|transition| (unix_time(zone, transition) - anchor).abs() > SETTLED_SECONDS)
}

/// The transition installing the wanted kind of offset that stands at
/// `anchor`: the latest one at or before it, or the earliest later one
/// when the zone has no history of that kind yet.
fn nearest(zone: TimeZoneRef<'_>, anchor: i64, daylight: bool) -> Option<usize> {
    let types = zone.local_time_types();
    let installs = |transition: &Transition| {
        types
            .get(transition.local_time_type_index())
            .is_some_and(|kind| kind.is_dst() == daylight)
    };

    let transitions = zone.transitions();

    let standing = transitions
        .iter()
        .enumerate()
        .rfind(|(_, transition)| unix_time(zone, transition) <= anchor && installs(transition));

    let found = standing.or_else(|| {
        transitions
            .iter()
            .enumerate()
            .find(|(_, transition)| installs(transition))
    });

    found.map(|(index, _)| index)
}

/// The yearly rule the onset at `index` repeats on, when the following
/// transitions of the same kind fall on the same month, week of the
/// month, weekday and local time.
///
/// Stated as a rule rather than a list of dated onsets because a
/// recurring event outlives any window a list could enumerate, and an
/// occurrence past the end of it would resolve against the last
/// observance and drift by the daylight offset.
fn yearly_rule(
    zone: TimeZoneRef<'_>,
    index: usize,
    daylight: bool,
    onset: NaiveDateTime,
) -> Option<String> {
    let types = zone.local_time_types();
    let ordinal = week_of_month(onset)?;

    let agreeing = zone
        .transitions()
        .iter()
        .skip(index + 1)
        .filter(|transition| {
            types
                .get(transition.local_time_type_index())
                .is_some_and(|kind| kind.is_dst() == daylight)
        })
        .take(AGREEING_TRANSITIONS)
        .filter(|transition| {
            let at = unix_time(zone, transition);

            let following = zone
                .find_local_time_type(at - 1)
                .ok()
                .and_then(|from| local(at, from.ut_offset()));

            following.is_some_and(|following| {
                following.month() == onset.month()
                    && following.time() == onset.time()
                    && week_of_month(following) == Some(ordinal)
            })
        })
        .count();

    (agreeing == AGREEING_TRANSITIONS).then(|| recurrence(onset.month(), ordinal, onset.weekday()))
}

/// One STANDARD or DAYLIGHT observance: the offset it leaves, the
/// offset it installs, when it takes effect and the rule it repeats on.
fn observance(
    name: &'static str,
    from: &LocalTimeType,
    to: &LocalTimeType,
    onset: NaiveDateTime,
    rule: Option<String>,
) -> IcalCst<'static> {
    let mut observance = component(name);

    let designation = to.time_zone_designation();
    if !designation.is_empty() {
        observance.push(text_prop(IcalPropKind::TzName, designation.to_owned()));
    }

    for (kind, offset) in [
        (IcalPropKind::TzOffsetFrom, from.ut_offset()),
        (IcalPropKind::TzOffsetTo, to.ut_offset()),
    ] {
        let value = IcalValue::UtcOffset(IcalUtcOffset(utc_offset(offset).into()));
        observance.push(prop(kind, value));
    }

    let stamp = onset.format("%Y%m%dT%H%M%S").to_string();
    observance.push(prop(
        IcalPropKind::DtStart,
        IcalValue::DateTime(IcalDateTime(stamp.into())),
    ));

    if let Some(rule) = rule {
        observance.push(prop(
            IcalPropKind::RRule,
            IcalValue::Recur(IcalRecur(rule.into())),
        ));
    }

    observance
}

/// The occurrence of a POSIX transition rule falling in `year`, as the
/// local date-time it lands on.
///
/// RFC 5545 3.6.5 states an observance DTSTART in the local time before
/// its transition, which is the offset the observance leaves, and a
/// POSIX rule states its time the same way, so the two line up and the
/// seconds need no shifting. The time is a second count rather than a
/// clock reading, and rules do name 24:00 and later, so the days it
/// runs to move the date and only the remainder becomes the time.
fn onset_of(day: &RuleDay, seconds: i32, year: i32) -> Option<NaiveDateTime> {
    let date = match day {
        RuleDay::MonthWeekDay(rule) => {
            let weekday = weekday(rule.week_day())?;
            let month = u32::from(rule.month());

            // NOTE: week 5 means the last one, which is the fourth in a
            // month the weekday falls in four times only.
            match rule.week() {
                5 => NaiveDate::from_weekday_of_month_opt(year, month, weekday, 5)
                    .or_else(|| NaiveDate::from_weekday_of_month_opt(year, month, weekday, 4))?,
                week => NaiveDate::from_weekday_of_month_opt(year, month, weekday, week)?,
            }
        }
        // NOTE: both Julian forms count days from the start of the
        // year, and they part company on 29 February only.
        RuleDay::Julian1WithoutLeap(rule) => NaiveDate::from_yo_opt(year, u32::from(rule.get()))?,
        RuleDay::Julian0WithLeap(rule) => NaiveDate::from_yo_opt(year, u32::from(rule.get()) + 1)?,
    };

    Some(date.and_hms_opt(0, 0, 0)? + Duration::seconds(i64::from(seconds)))
}

/// The yearly recurrence a POSIX transition rule denotes.
fn recur(day: &RuleDay) -> String {
    match day {
        RuleDay::MonthWeekDay(rule) => {
            // NOTE: iCalendar counts the last weekday of a month
            // backwards, where POSIX numbers it five.
            let ordinal = match rule.week() {
                5 => -1,
                week => i8::try_from(week).unwrap_or(1),
            };

            let weekday = weekday(rule.week_day()).unwrap_or(Weekday::Sun);
            recurrence(u32::from(rule.month()), ordinal, weekday)
        }
        // NOTE: a day counted from the start of the year lands on the
        // same calendar date every year, the leap day aside, so the
        // onset's own month and day restate it.
        RuleDay::Julian1WithoutLeap(_) | RuleDay::Julian0WithLeap(_) => {
            match onset_of(day, 0, EPOCH_YEAR) {
                Some(date) => date
                    .format("FREQ=YEARLY;BYMONTH=%-m;BYMONTHDAY=%-d")
                    .to_string(),
                None => String::from("FREQ=YEARLY"),
            }
        }
    }
}

/// A month, an occurrence within it and a weekday as an RRULE value.
fn recurrence(month: u32, ordinal: i8, weekday: Weekday) -> String {
    let weekday = match weekday {
        Weekday::Mon => "MO",
        Weekday::Tue => "TU",
        Weekday::Wed => "WE",
        Weekday::Thu => "TH",
        Weekday::Fri => "FR",
        Weekday::Sat => "SA",
        Weekday::Sun => "SU",
    };

    format!("FREQ=YEARLY;BYMONTH={month};BYDAY={ordinal}{weekday}")
}

/// Which occurrence of its weekday within the month a local date-time
/// falls on, as iCalendar counts them: 1 through 4 from the start, or
/// -1 for the last whatever the month's length.
///
/// A fifth occurrence is reported as the last one, since that is what
/// it is, and the rule then stays right in the months holding only
/// four.
fn week_of_month(onset: NaiveDateTime) -> Option<i8> {
    let day = onset.day();

    let last = onset
        .date()
        .with_day(1)?
        .checked_add_months(Months::new(1))?
        .checked_sub_days(Days::new(1))?
        .day();

    if day + 7 > last {
        return Some(-1);
    }

    i8::try_from((day - 1) / 7 + 1).ok()
}

/// The offset a zone with no closing rule stands at: the one its last
/// recorded transition installed, or its first type when it never
/// transitions at all.
fn standing(zone: TimeZoneRef<'static>) -> Option<&'static LocalTimeType> {
    let types = zone.local_time_types();

    match zone.transitions().last() {
        Some(transition) => types.get(transition.local_time_type_index()),
        None => types.first(),
    }
}

/// Midnight on the first day of [`EPOCH_YEAR`], the onset of an
/// observance the zone dates itself.
fn epoch() -> Option<NaiveDateTime> {
    NaiveDate::from_ymd_opt(EPOCH_YEAR, 1, 1)?.and_hms_opt(0, 0, 0)
}

/// An instant as the local date-time it reads as under `offset`.
fn local(at: i64, offset: i32) -> Option<NaiveDateTime> {
    let shifted = at.checked_add(i64::from(offset))?;
    DateTime::from_timestamp(shifted, 0).map(|stamp| stamp.naive_utc())
}

/// The Unix time a transition happens at.
///
/// A TZif record times its transitions on a scale that counts leap
/// seconds, so the corrections standing at that point come back off.
/// The database ships none today, which makes this the identity, but
/// reading it off the record costs nothing and cannot go stale.
fn unix_time(zone: TimeZoneRef<'_>, transition: &Transition) -> i64 {
    let leap_time = transition.unix_leap_time();

    let correction = zone
        .leap_seconds()
        .iter()
        .take_while(|leap| leap.unix_leap_time() <= leap_time)
        .last()
        .map_or(0, |leap| leap.correction());

    leap_time - i64::from(correction)
}

/// Seconds east of UTC as the iCalendar `±HHMM(SS)` offset.
fn utc_offset(seconds: i32) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let seconds = seconds.unsigned_abs();
    let (hours, minutes, seconds) = (seconds / 3600, (seconds % 3600) / 60, seconds % 60);

    match seconds {
        0 => format!("{sign}{hours:02}{minutes:02}"),
        _ => format!("{sign}{hours:02}{minutes:02}{seconds:02}"),
    }
}

/// A POSIX weekday, counted from Sunday, as a chrono one.
fn weekday(day: u8) -> Option<Weekday> {
    match day {
        0 => Some(Weekday::Sun),
        1 => Some(Weekday::Mon),
        2 => Some(Weekday::Tue),
        3 => Some(Weekday::Wed),
        4 => Some(Weekday::Thu),
        5 => Some(Weekday::Fri),
        6 => Some(Weekday::Sat),
        _ => None,
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
    fn at(year: i32, month: u32, day: u32) -> i64 {
        NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|date| date.and_hms_opt(0, 0, 0))
            .unwrap()
            .and_utc()
            .timestamp()
    }

    /// A generated zone, read back through the resolver ical-rs runs on
    /// the observances alone.
    ///
    /// Nothing of this module survives the round trip but the bytes, so
    /// what the assertions weigh is the document rather than the code
    /// that wrote it.
    fn resolved(tzid: &str, local: (i32, u8, u8, u8, u8)) -> IcalOffset {
        let (year, month, day, hour, minute) = local;

        let raw = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\n{}END:VCALENDAR\r\n",
            vtimezone(tzid, at(year, u32::from(month), u32::from(day))).expect("a known zone")
        );

        let cst = IcalCst::parse(&raw).expect("parse");
        let zone = IcalTimezone::of_calendar(&cst.decode(), tzid).expect("a VTIMEZONE");

        zone.resolve(IcalRecurDateTime {
            year,
            month,
            day,
            hour,
            minute,
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
            for year in [1975, 1990, 2004, 2024, 2025, 2026] {
                for month in 1..=12 {
                    let local = (year, month, 15, 12, 0);

                    // NOTE: a sample the clock skips or repeats has no
                    // single answer to compare, and the two that do are
                    // pinned by their own case.
                    let Some(offset) = resolved(tzid, local).unambiguous() else {
                        continue;
                    };

                    let civil = NaiveDate::from_ymd_opt(year, u32::from(month), 15)
                        .and_then(|date| date.and_hms_opt(12, 0, 0))
                        .unwrap();
                    let instant = civil.and_utc().timestamp() - i64::from(offset);

                    let expected = tzdb::tz_by_name(tzid)
                        .unwrap()
                        .find_local_time_type(instant)
                        .unwrap()
                        .ut_offset();

                    assert_eq!(offset, expected, "{tzid} at {local:?}");
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

    /// A rule dates its observances from the year it took effect, so a
    /// reader is not told a 2007 rule reached back to 1970.
    #[test]
    fn a_closing_rule_is_dated_from_where_the_record_stops() {
        let new_york = vtimezone("America/New_York", at(2026, 8, 14))
            .unwrap()
            .to_string();
        assert!(
            new_york.contains("DTSTART:20070311T020000\r\n"),
            "{new_york}"
        );
        assert!(
            new_york.contains("DTSTART:20071104T020000\r\n"),
            "{new_york}"
        );
    }

    /// Zones did run on second-accurate offsets before the war, and
    /// RFC 5545 3.3.14 keeps room for them.
    #[test]
    fn an_offset_states_the_seconds_only_when_it_has_any() {
        assert_eq!(utc_offset(0), "+0000");
        assert_eq!(utc_offset(-18000), "-0500");
        assert_eq!(utc_offset(19800), "+0530");
        assert_eq!(utc_offset(-177), "-000257");
    }

    /// A fifth occurrence is the last one, and saying so keeps the rule
    /// right in the months holding only four.
    #[test]
    fn the_last_occurrence_of_a_weekday_is_counted_from_the_end() {
        let onset = |year, month, day| {
            NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .unwrap()
        };

        assert_eq!(week_of_month(onset(2026, 3, 29)), Some(-1));
        assert_eq!(week_of_month(onset(2024, 3, 10)), Some(2));
        assert_eq!(week_of_month(onset(2026, 5, 29)), Some(-1));
        assert_eq!(week_of_month(onset(2026, 5, 22)), Some(4));
    }
}
