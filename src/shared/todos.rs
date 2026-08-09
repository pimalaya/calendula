//! VTODO projection over the shared items, plus the `todo` command
//! family built on it.
//!
//! A calendar collection mixes component kinds, so the `todo` commands
//! read the same items the `item` commands do and keep only the VTODOs.
//! [`Todo`] is that projection: the few properties a task list is read
//! by, pulled out of the iCalendar bytes with ical-rs. The bytes
//! themselves are never rewritten, so a projection is read-only and
//! lossy by design.

pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod update;

use ical::tree::{
    component::vtodo::VTODO,
    cst::IcalCst,
    prop::{
        due::DUE, percent_complete::PERCENT_COMPLETE, priority::PRIORITY, status::STATUS,
        summary::SUMMARY,
    },
};
use serde::Serialize;

use crate::shared::items::CalendarItem;

/// A VTODO projected out of a [`CalendarItem`]'s iCalendar bytes.
///
/// DUE keeps its iCalendar wire spelling (`YYYYMMDD` for a date,
/// `YYYYMMDDTHHMMSS` with an optional `Z` for a date-time), so what a
/// listing prints is what the calendar carries.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Todo {
    /// The id of the item the todo was projected from.
    pub id: String,
    /// The todo's SUMMARY, empty when it carries none.
    pub summary: String,
    /// The todo's DUE, verbatim, empty when it carries none.
    pub due: String,
    /// The todo's STATUS (`NEEDS-ACTION`, `IN-PROCESS`, `COMPLETED`,
    /// `CANCELLED`), empty when it carries none.
    pub status: String,
    /// The todo's PRIORITY, 1 (highest) to 9 (lowest), `None` when it
    /// carries none or an unparseable one.
    #[serde(default)]
    pub priority: Option<i64>,
    /// The todo's PERCENT-COMPLETE, 0 to 100, `None` when it carries
    /// none or an unparseable one.
    #[serde(default)]
    pub percent_complete: Option<i64>,
}

impl Todo {
    /// Projects every VTODO carried by `item`, in source order.
    ///
    /// An item whose bytes do not parse yields no todo rather than an
    /// error: a listing showing the rest of a calendar is more useful
    /// than one refusing to render because a single resource is
    /// malformed.
    pub fn project(item: &CalendarItem) -> Vec<Self> {
        let Ok(cst) = IcalCst::parse(&item.contents) else {
            return Vec::new();
        };

        cst.components::<VTODO>()
            .map(|vtodo| Self {
                id: item.id.clone(),
                summary: vtodo
                    .prop::<SUMMARY>()
                    .map(|text| text.0.into_owned())
                    .unwrap_or_default(),
                due: vtodo
                    .prop::<DUE>()
                    .map(|stamp| stamp.0.into_owned())
                    .unwrap_or_default(),
                status: vtodo
                    .prop::<STATUS>()
                    .map(|text| text.0.into_owned())
                    .unwrap_or_default(),
                priority: vtodo.prop::<PRIORITY>().and_then(|value| value.get()),
                percent_complete: vtodo
                    .prop::<PERCENT_COMPLETE>()
                    .and_then(|value| value.get()),
            })
            .collect()
    }

    /// The completion percentage as a column value: the number with a
    /// percent sign, or empty when the todo states none.
    ///
    /// A COMPLETED status implies 100 even where the property is
    /// absent (RFC 5545 3.8.1.8), so a finished task never reads as
    /// unstarted.
    pub fn progress(&self) -> String {
        match self.percent_complete {
            Some(percent) => format!("{percent}%"),
            None if self.status.eq_ignore_ascii_case("COMPLETED") => String::from("100%"),
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALENDAR: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//Pimalaya//calendula//EN\r\n",
        "BEGIN:VTODO\r\n",
        "UID:1@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DUE:20260814T170000Z\r\n",
        "SUMMARY:Write the report\r\n",
        "STATUS:IN-PROCESS\r\n",
        "PRIORITY:2\r\n",
        "PERCENT-COMPLETE:40\r\n",
        "END:VTODO\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:2@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260814T090000Z\r\n",
        "SUMMARY:Not a todo\r\n",
        "END:VEVENT\r\n",
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
    fn projection_keeps_vtodos_and_drops_every_other_kind() {
        let todos = Todo::project(&item(CALENDAR));

        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, "item-1");
        assert_eq!(todos[0].summary, "Write the report");
        assert_eq!(todos[0].due, "20260814T170000Z");
        assert_eq!(todos[0].status, "IN-PROCESS");
        assert_eq!(todos[0].priority, Some(2));
        assert_eq!(todos[0].percent_complete, Some(40));
    }

    #[test]
    fn an_unparseable_item_projects_nothing_instead_of_failing() {
        assert!(Todo::project(&item("not a calendar at all")).is_empty());
    }

    #[test]
    fn a_bare_todo_renders_empty_columns_rather_than_absent_ones() {
        let bare = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VTODO\r\nUID:1\r\nDTSTAMP:20260101T000000Z\r\nEND:VTODO\r\n",
            "END:VCALENDAR\r\n",
        );

        let todos = Todo::project(&item(bare));
        assert_eq!(todos.len(), 1);
        assert!(todos[0].summary.is_empty());
        assert!(todos[0].due.is_empty());
        assert!(todos[0].status.is_empty());
        assert_eq!(todos[0].priority, None);
        assert!(todos[0].progress().is_empty());
    }

    #[test]
    fn a_completed_todo_reads_as_finished_even_with_no_percentage() {
        let completed = Todo {
            status: "COMPLETED".into(),
            ..Default::default()
        };
        assert_eq!(completed.progress(), "100%");

        // An explicit percentage always wins over the implication.
        let explicit = Todo {
            status: "COMPLETED".into(),
            percent_complete: Some(90),
            ..Default::default()
        };
        assert_eq!(explicit.progress(), "90%");

        let started = Todo {
            percent_complete: Some(40),
            ..Default::default()
        };
        assert_eq!(started.progress(), "40%");
    }
}
