//! VJOURNAL projection over the shared items, plus the `journal`
//! command family built on it.
//!
//! A calendar collection mixes component kinds, so the `journal`
//! commands read the same items the `item` commands do and keep only
//! the VJOURNALs. [`Journal`] is that projection: a journal entry is a
//! dated note, so it carries a date and no end. The bytes themselves
//! are never rewritten, so a projection is read-only and lossy by
//! design.

pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod update;

use ical::tree::{
    component::vjournal::VJOURNAL,
    cst::IcalCst,
    prop::{dtstart::DTSTART, status::STATUS, summary::SUMMARY},
};
use serde::Serialize;

use crate::shared::items::CalendarItem;

/// A VJOURNAL projected out of a [`CalendarItem`]'s iCalendar bytes.
///
/// DTSTART keeps its iCalendar wire spelling, so what a listing prints
/// is what the calendar carries.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Journal {
    /// The id of the item the entry was projected from.
    pub id: String,
    /// The entry's SUMMARY, empty when it carries none.
    pub summary: String,
    /// The entry's DTSTART, verbatim, empty when it carries none. A
    /// journal entry is dated rather than scheduled, so it has no end.
    pub start: String,
    /// The entry's STATUS (`DRAFT`, `FINAL`, `CANCELLED`), empty when
    /// it carries none.
    pub status: String,
}

impl Journal {
    /// Projects every VJOURNAL carried by `item`, in source order.
    ///
    /// An item whose bytes do not parse yields no entry rather than an
    /// error, for the same reason the other component families do.
    pub fn project(item: &CalendarItem) -> Vec<Self> {
        let Ok(cst) = IcalCst::parse(&item.contents) else {
            return Vec::new();
        };

        cst.components::<VJOURNAL>()
            .map(|vjournal| Self {
                id: item.id.clone(),
                summary: vjournal
                    .prop::<SUMMARY>()
                    .map(|text| text.0.into_owned())
                    .unwrap_or_default(),
                start: vjournal
                    .prop::<DTSTART>()
                    .map(|stamp| stamp.0.into_owned())
                    .unwrap_or_default(),
                status: vjournal
                    .prop::<STATUS>()
                    .map(|text| text.0.into_owned())
                    .unwrap_or_default(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALENDAR: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Pimalaya//calendula//EN\r\n",
        "BEGIN:VJOURNAL\r\n",
        "UID:1@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART;VALUE=DATE:20260814\r\n",
        "SUMMARY:Retrospective notes\r\n",
        "STATUS:FINAL\r\n",
        "END:VJOURNAL\r\n",
        "BEGIN:VTODO\r\n",
        "UID:2@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "SUMMARY:Not a journal\r\n",
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
    fn projection_keeps_vjournals_and_drops_every_other_kind() {
        let entries = Journal::project(&item(CALENDAR));

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "item-1");
        assert_eq!(entries[0].summary, "Retrospective notes");
        assert_eq!(entries[0].start, "20260814");
        assert_eq!(entries[0].status, "FINAL");
    }

    #[test]
    fn an_unparseable_item_projects_nothing_instead_of_failing() {
        assert!(Journal::project(&item("not a calendar at all")).is_empty());
    }
}
