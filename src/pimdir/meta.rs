//! The `text/calendar` summary convention (pimdir SPEC 13).
//!
//! A pimdir store never parses an item's `meta`: it is an opaque,
//! application-defined blob whose shape the writer of a collection and
//! its readers agree on per kind. The mail and contact kinds already
//! fixed theirs; this module fixes the calendar one, so calendula, a
//! sync connector and any other reader project the same fields without
//! fetching a body.
//!
//! The companion `sort_key` holds DTSTART normalised to RFC 3339 in UTC
//! at seconds precision, exactly as mail normalises its `Date:`, which
//! is what lets a date-range read page a calendar with the store's own
//! statements.

use ical::tree::{
    component::{vevent::VEVENT, vjournal::VJOURNAL, vtodo::VTODO},
    cst::IcalCst,
    prop::{dtend::DTEND, dtstart::DTSTART, summary::SUMMARY, uid::UID},
};
use io_replica::placement::{ReplicaLinkId, ReplicaMeta, ReplicaSortKey};
use serde::{Deserialize, Serialize};

use crate::shared::events::Event;

/// The media type a pimdir collection declares to hold calendar items.
pub const CALENDAR_KIND: &str = "text/calendar";

/// A reader's view of a `text/calendar` summary (`v: 1`).
///
/// Every field but the summary is optional, and an absent field means
/// unknown, so an item summarised by a connector that knows less than
/// calendula still projects.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CalendarMeta {
    /// The convention version, always 1 today.
    pub v: u8,
    /// The item's UID, verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    /// The item's SUMMARY. Required, and may be empty.
    #[serde(default)]
    pub summary: String,
    /// DTSTART, normalised to RFC 3339 in UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// DTEND, normalised to RFC 3339 in UTC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// The dominant component kind (`VEVENT`, `VTODO`, `VJOURNAL`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// The raw item octets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

impl CalendarMeta {
    /// Reads a stored blob, falling back to an empty summary when it is
    /// absent or was written to a shape this version cannot read. A
    /// listing showing an item with blank columns beats one that fails.
    pub fn read(meta: Option<&ReplicaMeta>) -> Self {
        meta.and_then(|meta| serde_json::from_str(&meta.0).ok())
            .unwrap_or_default()
    }
}

/// Everything derived from an item's bytes when calendula writes it:
/// the cross-source link id, the summary blob and the sort key.
///
/// Kept together because all three come from one parse and all three
/// must be written together: a mutation that refreshes the body without
/// refreshing the key leaves the item sorted where its old start put
/// it.
pub struct CalendarProjection {
    /// The item's cross-source identity, its UID.
    pub link_id: ReplicaLinkId,
    /// The `v: 1` summary blob.
    pub meta: ReplicaMeta,
    /// DTSTART, normalised for ordering.
    pub sort_key: ReplicaSortKey,
}

/// Projects raw iCalendar bytes onto what the store needs to file them.
///
/// The link id is the item's UID, which is what identifies the same
/// calendar object across sources (RFC 5545 3.8.4.7). Content with no
/// usable UID falls back to a content-derived id, so it still files
/// rather than being rejected.
pub fn project(contents: &[u8]) -> CalendarProjection {
    let Ok(cst) = IcalCst::parse(contents) else {
        return CalendarProjection {
            link_id: ReplicaLinkId(format!("alt:{:032x}", contents.len())),
            meta: ReplicaMeta(String::from("{\"v\":1,\"summary\":\"\"}")),
            sort_key: ReplicaSortKey::default(),
        };
    };

    let (kind, uid, summary, start, end) = if let Some(vevent) = cst.components::<VEVENT>().next() {
        (
            "VEVENT",
            vevent.prop::<UID>().map(|uid| uid.0.into_owned()),
            vevent
                .prop::<SUMMARY>()
                .map(|text| text.0.into_owned())
                .unwrap_or_default(),
            vevent.prop::<DTSTART>().map(|at| at.0.into_owned()),
            vevent.prop::<DTEND>().map(|at| at.0.into_owned()),
        )
    } else if let Some(vtodo) = cst.components::<VTODO>().next() {
        (
            "VTODO",
            vtodo.prop::<UID>().map(|uid| uid.0.into_owned()),
            vtodo
                .prop::<SUMMARY>()
                .map(|text| text.0.into_owned())
                .unwrap_or_default(),
            vtodo.prop::<DTSTART>().map(|at| at.0.into_owned()),
            None,
        )
    } else if let Some(vjournal) = cst.components::<VJOURNAL>().next() {
        (
            "VJOURNAL",
            vjournal.prop::<UID>().map(|uid| uid.0.into_owned()),
            vjournal
                .prop::<SUMMARY>()
                .map(|text| text.0.into_owned())
                .unwrap_or_default(),
            vjournal.prop::<DTSTART>().map(|at| at.0.into_owned()),
            None,
        )
    } else {
        ("", None, String::new(), None, None)
    };

    let start = start.as_deref().and_then(normalize_stamp);
    let end = end.as_deref().and_then(normalize_stamp);

    let meta = CalendarMeta {
        v: 1,
        uid: uid.clone(),
        summary,
        start: start.clone(),
        end,
        kind: (!kind.is_empty()).then(|| kind.to_owned()),
        size: Some(contents.len() as u64),
    };

    let link_id = match uid {
        Some(uid) if !uid.trim().is_empty() => ReplicaLinkId(format!("uid:{}", uid.trim())),
        _ => ReplicaLinkId(format!(
            "alt:{}:{}",
            meta.summary,
            start.as_deref().unwrap_or("")
        )),
    };

    CalendarProjection {
        link_id,
        meta: ReplicaMeta(serde_json::to_string(&meta).unwrap_or_default()),
        sort_key: ReplicaSortKey(start.unwrap_or_default()),
    }
}

/// Normalises an iCalendar DATE or DATE-TIME to RFC 3339 in UTC at
/// seconds precision, the form the sort key orders on. A stamp that
/// does not parse yields nothing, which reads as unknown.
fn normalize_stamp(stamp: &str) -> Option<String> {
    let at = Event {
        start: stamp.to_owned(),
        ..Default::default()
    }
    .start_at()?;

    Some(at.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENT: &str = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
        "BEGIN:VEVENT\r\nUID:event-1@example.org\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260814T090000Z\r\nDTEND:20260814T100000Z\r\nSUMMARY:Stand-up\r\n",
        "END:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    #[test]
    fn an_event_projects_its_uid_summary_and_normalised_bounds() {
        let projection = project(EVENT.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        assert_eq!(projection.link_id.0, "uid:event-1@example.org");
        assert_eq!(projection.sort_key.0, "2026-08-14T09:00:00Z");
        assert_eq!(meta.v, 1);
        assert_eq!(meta.summary, "Stand-up");
        assert_eq!(meta.kind.as_deref(), Some("VEVENT"));
        assert_eq!(meta.start.as_deref(), Some("2026-08-14T09:00:00Z"));
        assert_eq!(meta.end.as_deref(), Some("2026-08-14T10:00:00Z"));
        assert_eq!(meta.size, Some(EVENT.len() as u64));
    }

    #[test]
    fn a_todo_projects_without_an_end_bound() {
        let todo = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VTODO\r\nUID:todo-1\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:Ship it\r\n",
            "END:VTODO\r\nEND:VCALENDAR\r\n",
        );

        let projection = project(todo.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        assert_eq!(projection.link_id.0, "uid:todo-1");
        assert_eq!(meta.kind.as_deref(), Some("VTODO"));
        assert_eq!(meta.summary, "Ship it");
        assert!(meta.end.is_none());
        // No DTSTART, so the item is orderable but unknown, and lands at
        // the head of an ascending listing rather than nowhere.
        assert!(projection.sort_key.is_unknown());
    }

    #[test]
    fn content_with_no_uid_still_files_under_a_derived_link_id() {
        let anonymous = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VEVENT\r\nDTSTAMP:20260101T000000Z\r\nDTSTART:20260814T090000Z\r\n",
            "SUMMARY:Nameless\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let projection = project(anonymous.as_bytes());
        assert_eq!(projection.link_id.0, "alt:Nameless:2026-08-14T09:00:00Z");
    }

    #[test]
    fn an_absent_or_unreadable_meta_reads_as_an_empty_summary() {
        assert_eq!(CalendarMeta::read(None).summary, "");

        let garbage = ReplicaMeta(String::from("{not json"));
        assert_eq!(CalendarMeta::read(Some(&garbage)).summary, "");
    }
}
