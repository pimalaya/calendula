//! VEVENT projection over the shared items, plus the `event` command
//! family built on it.
//!
//! A calendar collection mixes component kinds, so the `event`
//! commands read the same items the `item` commands do and keep only
//! the VEVENTs. [`Event`] is that projection: the few fields a listing
//! or an agenda renders, pulled out of the iCalendar bytes with
//! ical-rs. The bytes themselves are never rewritten, so a projection
//! is read-only and lossy by design.

pub mod agenda;
pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod update;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use ical::tree::{
    component::vevent::VEVENT,
    cst::IcalCst,
    prop::{description::DESCRIPTION, dtend::DTEND, dtstart::DTSTART, summary::SUMMARY},
};
use serde::Serialize;

use crate::shared::items::CalendarItem;

/// A VEVENT projected out of a [`CalendarItem`]'s iCalendar bytes.
///
/// The time fields keep their iCalendar wire spelling (`YYYYMMDD` for a
/// date, `YYYYMMDDTHHMMSS` with an optional `Z` for a date-time), so
/// what a listing prints is what the calendar carries. Parsing them
/// into a [`NaiveDateTime`] is a separate, fallible step.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Event {
    /// The id of the item the event was projected from.
    pub id: String,
    /// The event's SUMMARY, empty when it carries none.
    pub summary: String,
    /// The event's DESCRIPTION, empty when it carries none. Used as
    /// the agenda label when the summary is missing.
    #[serde(skip)]
    pub description: String,
    /// The event's DTSTART, verbatim.
    pub start: String,
    /// The event's DTEND, verbatim.
    pub end: String,
}

impl Event {
    /// Projects every VEVENT carried by `item`, in source order.
    ///
    /// An item whose bytes do not parse yields no event rather than an
    /// error: a listing showing the rest of a calendar is more useful
    /// than one refusing to render because a single resource is
    /// malformed.
    pub fn project(item: &CalendarItem) -> Vec<Self> {
        let Ok(cst) = IcalCst::parse(&item.contents) else {
            return Vec::new();
        };

        cst.components::<VEVENT>()
            .map(|vevent| Self {
                id: item.id.clone(),
                summary: vevent
                    .prop::<SUMMARY>()
                    .map(|text| text.0.into_owned())
                    .unwrap_or_default(),
                description: vevent
                    .prop::<DESCRIPTION>()
                    .map(|text| text.0.into_owned())
                    .unwrap_or_default(),
                start: vevent
                    .prop::<DTSTART>()
                    .map(|stamp| stamp.0.into_owned())
                    .unwrap_or_default(),
                end: vevent
                    .prop::<DTEND>()
                    .map(|stamp| stamp.0.into_owned())
                    .unwrap_or_default(),
            })
            .collect()
    }

    /// The label an agenda cell shows: the summary, falling back to the
    /// description.
    pub fn label(&self) -> &str {
        if self.summary.is_empty() {
            &self.description
        } else {
            &self.summary
        }
    }

    /// The event's start as a date-time, midnight for a date-only
    /// DTSTART. `None` when DTSTART is absent or unparseable.
    pub fn start_at(&self) -> Option<NaiveDateTime> {
        parse_stamp(&self.start)
    }
}

/// Parses an iCalendar DATE or DATE-TIME into a [`NaiveDateTime`].
///
/// A trailing `Z` (UTC) and a leading `TZID=` parameter are both
/// already stripped by the lens, which hands over the value alone. A
/// floating or zoned stamp is read as-is: calendula renders what the
/// calendar wrote rather than resolving time zones.
fn parse_stamp(stamp: &str) -> Option<NaiveDateTime> {
    let stamp = stamp.trim_end_matches('Z');
    let (date, time) = match stamp.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (stamp, None),
    };

    let date = NaiveDate::parse_from_str(date, "%Y%m%d").ok()?;
    let time = match time {
        Some(time) => NaiveTime::parse_from_str(time, "%H%M%S").ok()?,
        None => NaiveTime::MIN,
    };

    Some(date.and_time(time))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALENDAR: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Pimalaya//calendula//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260814T090000Z\r\n",
        "DTEND:20260814T100000Z\r\n",
        "SUMMARY:Stand-up\r\n",
        "END:VEVENT\r\n",
        "BEGIN:VTODO\r\n",
        "UID:2@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Not an event\r\n",
        "END:VTODO\r\n",
        "END:VCALENDAR\r\n",
    );

    fn item(contents: &str) -> CalendarItem {
        CalendarItem {
            id: "item-1".into(),
            calendar_id: "personal".into(),
            etag: None,
            contents: contents.as_bytes().to_vec(),
        }
    }

    #[test]
    fn projection_keeps_vevents_and_drops_every_other_kind() {
        let events = Event::project(&item(CALENDAR));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "item-1");
        assert_eq!(events[0].summary, "Stand-up");
        assert_eq!(events[0].start, "20260814T090000Z");
        assert_eq!(events[0].end, "20260814T100000Z");
    }

    #[test]
    fn an_unparseable_item_projects_nothing_instead_of_failing() {
        assert!(Event::project(&item("not a calendar at all")).is_empty());
    }

    #[test]
    fn the_label_falls_back_to_the_description() {
        let summarised = Event {
            summary: "Stand-up".into(),
            description: "Daily".into(),
            ..Default::default()
        };
        assert_eq!(summarised.label(), "Stand-up");

        let bare = Event {
            description: "Daily".into(),
            ..Default::default()
        };
        assert_eq!(bare.label(), "Daily");
    }

    #[test]
    fn both_stamp_shapes_parse_and_a_date_starts_at_midnight() {
        let zoned = Event {
            start: "20260814T090000Z".into(),
            ..Default::default()
        };
        let expected = NaiveDate::from_ymd_opt(2026, 8, 14)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        assert_eq!(zoned.start_at(), Some(expected));

        let all_day = Event {
            start: "20260814".into(),
            ..Default::default()
        };
        let midnight = NaiveDate::from_ymd_opt(2026, 8, 14)
            .unwrap()
            .and_time(NaiveTime::MIN);
        assert_eq!(all_day.start_at(), Some(midnight));

        let broken = Event {
            start: "nonsense".into(),
            ..Default::default()
        };
        assert_eq!(broken.start_at(), None);
    }
}
