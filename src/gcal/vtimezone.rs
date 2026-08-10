//! Synthesis of the VTIMEZONE a projected item's TZID refers to.
//!
//! The Calendar API names a zone and never expands it: an event carries
//! `timeZone: "Europe/Paris"`, not the observances that give the name a
//! meaning. Every other backend receives the expansion from its server,
//! so only this one has to produce it.
//!
//! Without it a projected item names a zone nothing defines, which RFC
//! 5545 3.2.19 forbids: a TZID parameter must refer to a VTIMEZONE of
//! the same iCalendar object. A lenient client resolves the name
//! against its own database and renders the event correctly; a strict
//! one is entitled to refuse the item or to read the stamp as floating,
//! which moves it.
//!
//! [`crate::gcal::project::to_ical`] splices the calendar stash back
//! before `END:VCALENDAR`, and an item calendula itself wrote carries
//! its original VTIMEZONE there. Synthesis therefore covers exactly the
//! items the stash cannot: those created in Google's own interface,
//! which never passed through the projection on the way in.

use std::fmt::Write as _;

use jiff::{
    Timestamp,
    civil::{self, Weekday},
    tz::{Dst, Offset, TimeZone},
};

/// Transitions read on each side of the anchor when looking for the
/// observances in force around it. Two a year is the common case, so
/// this spans several years either way, enough to recognise a rule
/// while staying a bounded read.
const TRANSITION_SPAN: usize = 8;

/// Following transitions of one kind that must agree before the
/// observance is expressed as a yearly rule rather than a single dated
/// onset.
const AGREEING_TRANSITIONS: usize = 2;

/// Onset given to a zone that has never changed offset. Its rule has no
/// beginning to state, and the Unix epoch is the conventional stand-in;
/// a default [`civil::DateTime`] would render as year zero, which is not
/// a date any parser should be handed.
const EPOCH: civil::DateTime = civil::DateTime::constant(1970, 1, 1, 0, 0, 0, 0);

/// How far from the item a zone's nearest shift must be before the zone
/// counts as settled. Two years clears the annual pair comfortably while
/// still catching a zone that gave daylight saving up decades ago.
const SETTLED_AFTER: jiff::SignedDuration = jiff::SignedDuration::from_hours(24 * 365 * 2);

/// One STANDARD or DAYLIGHT observance of a zone.
struct Observance {
    /// Whether the observance is the daylight-saving one.
    dst: bool,
    /// Onset, in the local time that `from` was still in force at.
    onset: civil::DateTime,
    /// Offset in force before the change.
    from: Offset,
    /// Offset the change brings into force.
    to: Offset,
    /// Abbreviation the zone goes by under `to`, such as `CEST`.
    name: String,
    /// Yearly rule the onset repeats on, when it repeats predictably.
    rule: Option<String>,
}

/// Renders the VTIMEZONE for `zone` as iCalendar lines, or [`None`] when
/// the name is not one the database knows.
///
/// `anchor` selects which era of the zone's history to describe. A zone
/// is only obliged to be correct about the times an item actually
/// names, and anchoring on the item keeps the component to the
/// observances in force around it rather than the zone's whole past.
pub(super) fn vtimezone(zone: &str, anchor: Timestamp) -> Option<Vec<String>> {
    let tz = TimeZone::get(zone).ok()?;

    let observances = if settled(&tz, anchor) {
        Vec::new()
    } else {
        observances(&tz, anchor)
    };

    let mut lines = vec![String::from("BEGIN:VTIMEZONE"), format!("TZID:{zone}")];

    if observances.is_empty() {
        // A zone that never changes offset still needs one observance
        // to be a well-formed component, and the epoch is as good an
        // onset as any for a rule that has always applied.
        let offset = tz.to_offset(anchor);
        lines.extend(render(&Observance {
            dst: false,
            onset: EPOCH,
            from: offset,
            to: offset,
            name: abbreviation(&tz, anchor),
            rule: None,
        }));
    } else {
        for observance in &observances {
            lines.extend(render(observance));
        }
    }

    lines.push(String::from("END:VTIMEZONE"));
    Some(lines)
}

/// Whether the zone holds one offset over the years around `anchor`.
///
/// A zone that gave up daylight saving long ago still has the old
/// observances in its history, and describing an item by them ships a
/// decades-dead summer time with every event. Hong Kong last shifted in
/// 1979, so an item from today would otherwise carry a 1979 DAYLIGHT
/// component. A settled zone is better described the way one that never
/// shifted at all is: a single observance at the offset actually in
/// force.
///
/// Asked of the neighbouring transitions rather than by sampling the
/// offset a year either side: a year apart lands in the same season, so
/// a zone that shifts every spring would read as settled.
fn settled(tz: &TimeZone, anchor: Timestamp) -> bool {
    let recent = tz
        .preceding(anchor)
        .next()
        .is_some_and(|transition| anchor.duration_since(transition.timestamp()) < SETTLED_AFTER);

    let upcoming = tz
        .following(anchor)
        .next()
        .is_some_and(|transition| transition.timestamp().duration_since(anchor) < SETTLED_AFTER);

    !recent && !upcoming
}

/// The most recent observance of each kind at or before `anchor`,
/// falling back to the earliest later one when the zone has no history
/// on that side yet.
fn observances(tz: &TimeZone, anchor: Timestamp) -> Vec<Observance> {
    let preceding: Vec<_> = tz.preceding(anchor).take(TRANSITION_SPAN).collect();
    let following: Vec<_> = tz.following(anchor).take(TRANSITION_SPAN).collect();

    let mut observances = Vec::new();

    for dst in [Dst::No, Dst::Yes] {
        // `preceding` runs backwards from the anchor, so its first match
        // is the latest one still in force.
        let transition = preceding
            .iter()
            .find(|transition| transition.dst() == dst)
            .or_else(|| following.iter().find(|transition| transition.dst() == dst));

        let Some(transition) = transition else {
            continue;
        };

        let to = transition.offset();
        let from = offset_before(tz, transition.timestamp());

        if from == to {
            // Not a change of offset, so nothing an observance would
            // express. Some zones renumber an abbreviation this way.
            continue;
        }

        observances.push(Observance {
            dst: dst == Dst::Yes,
            onset: from.to_datetime(transition.timestamp()),
            from,
            to,
            name: transition.abbreviation().to_string(),
            rule: yearly_rule(tz, transition.timestamp(), dst, from),
        });
    }

    observances
}

/// The offset in force immediately before `at`.
///
/// Read one nanosecond earlier rather than from the previous
/// transition, so a zone whose history starts at `at` still answers.
fn offset_before(tz: &TimeZone, at: Timestamp) -> Offset {
    tz.to_offset(at - jiff::SignedDuration::from_nanos(1))
}

/// The zone's abbreviation at `at`.
fn abbreviation(tz: &TimeZone, at: Timestamp) -> String {
    tz.to_offset_info(at).abbreviation().to_string()
}

/// The yearly rule `onset` repeats on, when the next transitions of the
/// same kind agree with it on month, weekday, week of the month and
/// local time.
///
/// Expressed as a rule rather than a list of dated onsets because a
/// recurring event outlives any window this could enumerate, and an
/// occurrence past the end of that window would otherwise resolve
/// against the last observance and drift by the DST offset.
fn yearly_rule(tz: &TimeZone, at: Timestamp, dst: Dst, from: Offset) -> Option<String> {
    let onset = from.to_datetime(at);
    let (week, weekday) = week_of_month(onset)?;

    let agreeing = tz
        .following(at)
        .filter(|transition| transition.dst() == dst)
        .take(AGREEING_TRANSITIONS)
        .filter(|transition| {
            let next =
                offset_before(tz, transition.timestamp()).to_datetime(transition.timestamp());
            week_of_month(next) == Some((week, weekday))
                && next.month() == onset.month()
                && next.time() == onset.time()
        })
        .count();

    if agreeing < AGREEING_TRANSITIONS {
        return None;
    }

    Some(format!(
        "RRULE:FREQ=YEARLY;BYMONTH={};BYDAY={}{}",
        onset.month(),
        week,
        weekday_ical(weekday),
    ))
}

/// Which occurrence of its weekday in the month `onset` falls on, as
/// iCalendar counts them: 1 through 4 from the start, or -1 for the last
/// whatever the month's length.
///
/// A fifth occurrence is reported as the last one, since that is what it
/// is, and the rule stays right in the months that have only four.
fn week_of_month(onset: civil::DateTime) -> Option<(i8, Weekday)> {
    let day = i16::from(onset.day());
    let last_day = i16::from(onset.date().last_of_month().day());

    if day + 7 > last_day {
        return Some((-1, onset.weekday()));
    }

    let week = i8::try_from((day - 1) / 7 + 1).ok()?;
    Some((week, onset.weekday()))
}

/// The two-letter iCalendar spelling of a weekday.
fn weekday_ical(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "MO",
        Weekday::Tuesday => "TU",
        Weekday::Wednesday => "WE",
        Weekday::Thursday => "TH",
        Weekday::Friday => "FR",
        Weekday::Saturday => "SA",
        Weekday::Sunday => "SU",
    }
}

/// One observance as its iCalendar lines.
fn render(observance: &Observance) -> Vec<String> {
    let name = if observance.dst {
        "DAYLIGHT"
    } else {
        "STANDARD"
    };

    let mut lines = vec![
        format!("BEGIN:{name}"),
        format!("DTSTART:{}", stamp(observance.onset)),
        format!("TZOFFSETFROM:{}", offset_ical(observance.from)),
        format!("TZOFFSETTO:{}", offset_ical(observance.to)),
    ];

    if !observance.name.is_empty() {
        lines.push(format!("TZNAME:{}", observance.name));
    }

    if let Some(rule) = &observance.rule {
        lines.push(rule.clone());
    }

    lines.push(format!("END:{name}"));
    lines
}

/// A local date and time as an iCalendar stamp.
fn stamp(datetime: civil::DateTime) -> String {
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}",
        datetime.year(),
        datetime.month(),
        datetime.day(),
        datetime.hour(),
        datetime.minute(),
        datetime.second(),
    )
}

/// An offset as the signed `+HHMM` iCalendar form, widened to `+HHMMSS`
/// for the handful of historical zones whose offset is not a whole
/// number of minutes.
fn offset_ical(offset: Offset) -> String {
    let total = offset.seconds();
    let sign = if total < 0 { '-' } else { '+' };
    let total = total.unsigned_abs();

    let mut rendered = String::with_capacity(7);
    let _ = write!(
        rendered,
        "{sign}{:02}{:02}",
        total / 3600,
        (total % 3600) / 60
    );

    if !total.is_multiple_of(60) {
        let _ = write!(rendered, "{:02}", total % 60);
    }

    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Midnight UTC on the given day, as an anchor.
    fn at(date: &str) -> Timestamp {
        let date: civil::Date = date.parse().unwrap();
        Offset::UTC
            .to_timestamp(date.to_datetime(civil::Time::MIN))
            .unwrap()
    }

    fn rendered(zone: &str, date: &str) -> String {
        vtimezone(zone, at(date)).unwrap().join("\n")
    }

    #[test]
    fn a_zone_that_shifts_states_both_observances_as_yearly_rules() {
        assert_eq!(
            rendered("Europe/Paris", "2026-08-14"),
            concat!(
                "BEGIN:VTIMEZONE\n",
                "TZID:Europe/Paris\n",
                "BEGIN:STANDARD\n",
                "DTSTART:20251026T030000\n",
                "TZOFFSETFROM:+0200\n",
                "TZOFFSETTO:+0100\n",
                "TZNAME:CET\n",
                "RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\n",
                "END:STANDARD\n",
                "BEGIN:DAYLIGHT\n",
                "DTSTART:20260329T020000\n",
                "TZOFFSETFROM:+0100\n",
                "TZOFFSETTO:+0200\n",
                "TZNAME:CEST\n",
                "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\n",
                "END:DAYLIGHT\n",
                "END:VTIMEZONE",
            )
        );
    }

    #[test]
    fn the_observances_describe_the_era_the_item_names() {
        // The United States moved its rule in 2007. An item from before
        // it must not be described by the rule that replaced it, or
        // every occurrence in the weeks between the two onsets reads an
        // hour out.
        let modern = rendered("America/New_York", "2024-07-14");
        assert!(
            modern.contains("RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU"),
            "{modern}"
        );
        assert!(
            modern.contains("RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU"),
            "{modern}"
        );

        let historical = rendered("America/New_York", "1980-07-14");
        assert!(
            historical.contains("RRULE:FREQ=YEARLY;BYMONTH=4;BYDAY=-1SU"),
            "{historical}"
        );
        assert!(
            historical.contains("RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"),
            "{historical}"
        );
    }

    #[test]
    fn the_southern_hemisphere_keeps_its_own_order() {
        let sydney = rendered("Australia/Sydney", "2026-08-14");
        assert!(sydney.contains("TZNAME:AEDT"), "{sydney}");
        // Daylight saving opens in October and closes in April there.
        assert!(
            sydney.contains("RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=1SU"),
            "{sydney}"
        );
        assert!(
            sydney.contains("RRULE:FREQ=YEARLY;BYMONTH=4;BYDAY=1SU"),
            "{sydney}"
        );
    }

    #[test]
    fn a_zone_that_never_shifts_gets_one_dated_observance() {
        assert_eq!(
            rendered("Asia/Kolkata", "2026-08-14"),
            concat!(
                "BEGIN:VTIMEZONE\n",
                "TZID:Asia/Kolkata\n",
                "BEGIN:STANDARD\n",
                "DTSTART:19700101T000000\n",
                "TZOFFSETFROM:+0530\n",
                "TZOFFSETTO:+0530\n",
                "TZNAME:IST\n",
                "END:STANDARD\n",
                "END:VTIMEZONE",
            )
        );
    }

    #[test]
    fn a_zone_that_gave_up_daylight_saving_is_not_described_by_its_ghost() {
        // Hong Kong last shifted in 1979. An item from today must not
        // carry that observance: the offset has been +0800 throughout
        // living memory, and a decades-dead summer time is noise a
        // reader has to reason past.
        let settled = rendered("Asia/Hong_Kong", "2026-08-14");
        assert!(!settled.contains("DAYLIGHT"), "{settled}");
        assert!(settled.contains("TZOFFSETTO:+0800"), "{settled}");
        assert!(!settled.contains("RRULE"), "{settled}");

        // While it was still shifting, it is described as shifting.
        let shifting = rendered("Asia/Hong_Kong", "1978-08-14");
        assert!(shifting.contains("DAYLIGHT"), "{shifting}");
    }

    #[test]
    fn a_name_the_database_does_not_know_is_refused() {
        assert!(vtimezone("Mars/Olympus_Mons", at("2026-08-14")).is_none());
        assert!(vtimezone("", at("2026-08-14")).is_none());
    }

    #[test]
    fn an_offset_renders_seconds_only_when_it_has_any() {
        assert_eq!(offset_ical(Offset::from_seconds(3600).unwrap()), "+0100");
        assert_eq!(offset_ical(Offset::from_seconds(-18000).unwrap()), "-0500");
        assert_eq!(offset_ical(Offset::from_seconds(19800).unwrap()), "+0530");
        assert_eq!(offset_ical(Offset::from_seconds(0).unwrap()), "+0000");
        // Some pre-standardisation zones are offset by an odd number of
        // seconds, which the four-digit form cannot express.
        assert_eq!(offset_ical(Offset::from_seconds(-177).unwrap()), "-000257");
    }

    #[test]
    fn the_last_occurrence_of_a_weekday_is_counted_from_the_end() {
        let last_sunday_of_march = civil::date(2026, 3, 29).to_datetime(civil::Time::MIN);
        assert_eq!(
            week_of_month(last_sunday_of_march),
            Some((-1, Weekday::Sunday))
        );

        let second_sunday_of_march = civil::date(2024, 3, 10).to_datetime(civil::Time::MIN);
        assert_eq!(
            week_of_month(second_sunday_of_march),
            Some((2, Weekday::Sunday))
        );

        // A fifth occurrence is the last one, and saying so keeps the
        // rule right in the months that have only four.
        let fifth_friday = civil::date(2026, 5, 29).to_datetime(civil::Time::MIN);
        assert_eq!(week_of_month(fifth_friday), Some((-1, Weekday::Friday)));
    }
}
