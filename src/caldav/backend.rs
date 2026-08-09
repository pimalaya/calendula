//! CalDAV adapter for the shared cross-protocol client.
//!
//! Projects [`io_webdav`]'s RFC 4791 calendars and calendar object
//! resources onto calendula's own shared types, over a connected
//! [`WebdavClientStd`] whose calendar home-set is already resolved.
//!
//! Item ids are the resource names the server returned, verbatim:
//! io-webdav neither appends nor strips a file extension, so an id a
//! listing showed addresses the same resource on every verb. Because
//! the server owns the query, a [`CalendarTimeRange`] is pushed down as
//! an RFC 4791 `time-range` filter rather than applied locally.

use anyhow::{Context, Result, anyhow};
use ical::tree::{component::vevent::VEVENT, cst::IcalCst, prop::uid::UID};
use io_webdav::{
    client::WebdavClientStd,
    rfc4791::{calendar::CaldavCalendar, item::CaldavItemEntry},
};

use crate::{
    caldav::client::connect_and_resolve,
    config::CaldavConfig,
    shared::{
        calendars::{Calendar, CalendarDiff},
        client::paginate,
        items::{CalendarItem, CalendarTimeRange},
    },
};

/// The shared-API glue over a connected CalDAV client.
pub struct CaldavBackend {
    client: WebdavClientStd,
}

impl CaldavBackend {
    /// Connects to the server and walks the discovery chain, so the
    /// calendar home-set is cached before any command runs.
    pub fn new(config: CaldavConfig) -> Result<Self> {
        Ok(Self {
            client: connect_and_resolve(&config)?,
        })
    }

    /// Lists every calendar collection under the calendar home-set.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let calendars = self.client.list_calendars()?;
        Ok(calendars.into_iter().map(calendar_from).collect())
    }

    /// Creates a calendar collection through MKCALENDAR.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        let calendar = CaldavCalendar {
            id: id.to_owned(),
            display_name: Some(name.to_owned()),
            description: description.map(str::to_owned),
            color: color.map(str::to_owned),
            ..Default::default()
        };

        self.client.create_calendar(&calendar)?;
        Ok(id.to_owned())
    }

    /// Rewrites a calendar's properties through PROPPATCH, reading the
    /// current ones first so an untouched field is written back
    /// unchanged.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        let mut calendar = self
            .client
            .list_calendars()?
            .into_iter()
            .find(|calendar| calendar.id == id)
            .ok_or_else(|| anyhow!("Calendar `{id}` not found"))?;

        if let Some(name) = patch.name {
            calendar.display_name = Some(name);
        }

        if let Some(description) = patch.description {
            calendar.description = description;
        }

        if let Some(color) = patch.color {
            calendar.color = color;
        }

        self.client.update_calendar(&calendar)?;
        Ok(())
    }

    /// Deletes a calendar collection and everything inside it.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        self.client.delete_calendar(id)?;
        Ok(())
    }

    /// Lists items through a `calendar-query` REPORT, narrowing the
    /// query server-side when a range is given.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        range: Option<&CalendarTimeRange>,
    ) -> Result<Vec<CalendarItem>> {
        let entries = self.client.list_items(calendar_id, &comp_filter(range))?;

        let items = entries
            .into_iter()
            .map(|entry| item_from(calendar_id, entry))
            .collect();

        Ok(paginate(items, page, page_size))
    }

    /// Reads one item's raw iCalendar bytes plus its entity tag.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let body = self
            .client
            .read_item(calendar_id, item_id)
            .with_context(|| format!("Read item `{item_id}` from calendar `{calendar_id}`"))?;

        Ok(CalendarItem {
            id: item_id.to_owned(),
            calendar_id: calendar_id.to_owned(),
            etag: body.etag,
            contents: body.data,
        })
    }

    /// PUTs a new resource. The server may name the resource itself, in
    /// which case the id it reported wins over the one asked for.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let id = format!("{}.ics", new_resource_name(&contents));
        let created = self.client.create_item(calendar_id, &id, contents)?;
        Ok(created.id)
    }

    /// PUTs over an existing resource, optionally gated on `if_match`.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        self.client
            .update_item(calendar_id, item_id, contents, if_match)?;
        Ok(())
    }

    /// DELETEs a resource unconditionally.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        self.client.delete_item(calendar_id, item_id, None)?;
        Ok(())
    }
}

/// The VCALENDAR child filter a listing sends: every component kind
/// when no range is asked for, or a VEVENT `time-range` when one is.
///
/// A range only narrows VEVENTs, since RFC 4791 9.9 defines the
/// overlap test against a component's own start and end, and a VTODO or
/// VJOURNAL without them would be dropped for the wrong reason.
fn comp_filter(range: Option<&CalendarTimeRange>) -> String {
    match range {
        Some(range) => format!(
            "<C:comp-filter name=\"VEVENT\">{}</C:comp-filter>",
            range.to_caldav_filter()
        ),
        None => String::new(),
    }
}

/// Projects a CalDAV calendar onto the shared [`Calendar`], falling
/// back to the collection id when the server sets no display name.
fn calendar_from(calendar: CaldavCalendar) -> Calendar {
    Calendar {
        name: calendar
            .display_name
            .clone()
            .unwrap_or_else(|| calendar.id.clone()),
        id: calendar.id,
        description: calendar.description,
        color: calendar.color,
    }
}

/// Projects a CalDAV multistatus entry onto the shared
/// [`CalendarItem`].
fn item_from(calendar_id: &str, entry: CaldavItemEntry) -> CalendarItem {
    CalendarItem {
        id: entry.id,
        calendar_id: calendar_id.to_owned(),
        etag: entry.etag,
        contents: entry.data,
    }
}

/// Derives the resource name a create PUTs to from the item's own UID,
/// so a resource is addressable by the identity its content already
/// carries. Falls back to a content digest when the bytes carry no UID.
///
/// The name is only a proposal: a server that names the resource itself
/// reports its choice in the `Location` header, and io-webdav returns
/// that instead.
fn new_resource_name(contents: &[u8]) -> String {
    let uid = IcalCst::parse(contents).ok().and_then(|cst| {
        cst.components::<VEVENT>()
            .next()
            .and_then(|vevent| vevent.prop::<UID>())
            .map(|uid| uid.0.into_owned())
    });

    match uid {
        Some(uid) if is_safe_segment(&uid) => uid,
        _ => format!("{:016x}", fnv1a(contents)),
    }
}

/// Whether a UID can be used verbatim as a URL path segment: no empty
/// value, no separator, and nothing needing percent-encoding.
fn is_safe_segment(uid: &str) -> bool {
    !uid.is_empty()
        && uid.len() <= 128
        && uid
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'))
}

/// A 64-bit FNV-1a digest, the fallback resource name for content
/// carrying no usable UID.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;

    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_range_asks_for_every_component_kind() {
        assert_eq!(comp_filter(None), "");
    }

    #[test]
    fn a_range_narrows_the_query_to_overlapping_vevents() {
        let range = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: Some("20260901T000000Z".into()),
        };

        assert_eq!(
            comp_filter(Some(&range)),
            "<C:comp-filter name=\"VEVENT\">\
             <C:time-range start=\"20260801T000000Z\" end=\"20260901T000000Z\" />\
             </C:comp-filter>"
        );
    }

    #[test]
    fn a_resource_name_comes_from_the_uid_when_it_is_url_safe() {
        let calendar = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VEVENT\r\nUID:event-1@example.org\r\nDTSTAMP:20260101T000000Z\r\n",
            "DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        assert_eq!(
            new_resource_name(calendar.as_bytes()),
            "event-1@example.org"
        );
    }

    #[test]
    fn an_unsafe_or_missing_uid_falls_back_to_a_digest() {
        let slashed = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VEVENT\r\nUID:a/b\r\nDTSTAMP:20260101T000000Z\r\n",
            "DTSTART:20260814T090000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let name = new_resource_name(slashed.as_bytes());
        assert_eq!(name.len(), 16);
        assert!(name.chars().all(|c| c.is_ascii_hexdigit()));

        // Deterministic, so the same bytes always propose the same name.
        assert_eq!(name, new_resource_name(slashed.as_bytes()));
        assert_ne!(name, new_resource_name(b"other bytes entirely"));
    }
}
