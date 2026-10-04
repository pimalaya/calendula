//! # Event
//!
//! The VEVENT projection over the shared items, and the `event` command
//! family built on it.
//!
//! A calendar collection mixes component kinds, so the `event` commands read
//! the same items the `item` commands do and keep only the VEVENTs. [`Event`]
//! is that projection: what a listing, an agenda or a script reads of an
//! event, its times resolved to instants ([`expand`]).
//!
//! The bytes themselves are never rewritten, so a projection is read-only and
//! lossy by design.

pub mod agenda;
pub mod build;
pub mod cli;
pub mod create;
pub mod delete;
pub mod expand;
pub mod find;
pub mod list;
pub mod read;
pub mod update;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::item::{CalendarItem, CalendarTimeRange};

/// A VEVENT projected out of a [`CalendarItem`]'s iCalendar bytes, or one
/// occurrence of a recurring one.
///
/// `start` and `end` keep the iCalendar wire spelling, so what a listing
/// prints is what the calendar carries; `startsAt` and `endsAt` are the
/// same times resolved, with their UTC offset.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// The id of the item the event was projected from.
    pub id: String,
    /// The entity tag of that item: the version a write to it can be
    /// gated on (`--if-match`), `null` when the backend has none.
    pub etag: Option<String>,
    /// The event's `UID`, empty when it carries none.
    pub uid: String,
    /// The instance identity of an occurrence, as a `RECURRENCE-ID`
    /// carries it: what addresses one occurrence beside `id`. `null` for
    /// an event that does not recur, and for a series listed whole.
    pub recurrence_id: Option<String>,
    /// The event's SUMMARY, empty when it carries none.
    pub summary: String,
    /// The event's DESCRIPTION, empty when it carries none.
    pub description: String,
    /// The event's LOCATION, empty when it carries none.
    pub location: String,
    /// The start, iCalendar-spelled (`YYYYMMDD`, `YYYYMMDDTHHMMSS[Z]`).
    pub start: String,
    /// The end, spelled as `start` is: `DTEND`, or the start plus the
    /// `DURATION`.
    pub end: String,
    /// Whether the event spans whole days (a `DATE` start).
    pub all_day: bool,
    /// The start as RFC 3339 with its UTC offset, or `YYYY-MM-DD` for a
    /// whole day.
    pub starts_at: Option<String>,
    /// The end, spelled as `startsAt` is; a whole day's is exclusive.
    pub ends_at: Option<String>,
    /// The zone the start is written in: its `TZID`, `UTC`, or the local
    /// zone a floating time is read in. `null` for a whole day.
    pub time_zone: Option<String>,
    /// Whether the event is a recurring series, one of its occurrences,
    /// or an override of one.
    pub recurring: bool,
    /// The `STATUS` uppercased: `CONFIRMED`, `TENTATIVE`, `CANCELLED`.
    pub status: Option<String>,
    /// The `TRANSP` uppercased: `OPAQUE` (busy) or `TRANSPARENT` (free).
    pub transparency: Option<String>,
    /// The Outlook busy status (`X-MICROSOFT-CDO-BUSYSTATUS`) uppercased:
    /// `FREE`, `TENTATIVE`, `BUSY`, `OOF`, `WORKINGELSEWHERE`.
    pub busy_status: Option<String>,
    /// The `ORGANIZER`.
    pub organizer: Option<EventPerson>,
    /// Every `ATTENDEE`, in document order.
    pub attendees: Vec<EventAttendee>,
    /// The join link of an online meeting: a `CONFERENCE`, or the
    /// vendors' own properties.
    pub online_meeting_url: Option<String>,
    /// The start in Unix seconds, for windowing and ordering.
    #[serde(skip)]
    pub(crate) start_secs: Option<i64>,
    /// The end in Unix seconds, for windowing.
    #[serde(skip)]
    pub(crate) end_secs: Option<i64>,
}

/// A calendar user an event names.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventPerson {
    /// The address, `mailto:` removed.
    pub email: String,
    /// The `CN`, when it carries one.
    pub name: Option<String>,
}

/// An `ATTENDEE`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventAttendee {
    /// The address, `mailto:` removed.
    pub email: String,
    /// The `CN`, when it carries one.
    pub name: Option<String>,
    /// The `PARTSTAT` uppercased, `NEEDS-ACTION` when absent.
    pub partstat: String,
    /// The `ROLE` uppercased, `REQ-PARTICIPANT` when absent.
    pub role: String,
    /// Whether a reply is expected (`RSVP=TRUE`).
    pub rsvp: bool,
    /// The `CUTYPE` uppercased, `INDIVIDUAL` when absent; a `ROOM` or a
    /// `RESOURCE` is a place or a thing rather than a person.
    pub cutype: String,
}

impl Event {
    /// Projects every VEVENT carried by `item`, in source order, a series
    /// once at its own start.
    ///
    /// An item whose bytes do not parse yields no event rather than an error,
    /// so one malformed resource does not stop the rest of the calendar from
    /// rendering.
    pub fn project(item: &CalendarItem) -> Vec<Self> {
        tagged(item, expand::project(item))
    }

    /// Projects the occurrences of `item`'s events overlapping `range`,
    /// recurring series expanded, in start order.
    pub fn occurrences(item: &CalendarItem, range: Option<&CalendarTimeRange>) -> Vec<Self> {
        tagged(item, expand::occurrences(item, range))
    }

    /// Projects the one occurrence of `item` that `recurrence_id` names.
    pub fn occurrence(item: &CalendarItem, recurrence_id: &str) -> Option<Self> {
        expand::occurrence(item, recurrence_id).map(|event| Event {
            etag: item.etag.clone(),
            ..event
        })
    }

    /// The label an agenda cell shows: the summary, then the description.
    pub fn label(&self) -> &str {
        if self.summary.is_empty() {
            &self.description
        } else {
            &self.summary
        }
    }

    /// The event's start as a date-time, midnight for a date-only DTSTART.
    pub fn start_at(&self) -> Option<NaiveDateTime> {
        parse_stamp(&self.start)
    }
}

/// Stamps every event projected from `item` with the item's entity tag.
fn tagged(item: &CalendarItem, events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .map(|event| Event {
            etag: item.etag.clone(),
            ..event
        })
        .collect()
}

/// Parses an iCalendar DATE or DATE-TIME into a [`NaiveDateTime`].
///
/// The lens already strips a trailing `Z` and a leading `TZID=` parameter, so
/// only the value arrives here. A floating or zoned stamp is read as-is:
/// calendula renders what the calendar wrote rather than resolving zones.
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
