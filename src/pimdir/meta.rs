//! The `text/calendar` summary convention (pimdir SPEC Annex A.3).
//!
//! A pimdir store never parses an item's `meta`: it is an opaque,
//! application-defined blob whose shape the writer of a collection and
//! its readers agree on per kind. The mail and contact kinds already
//! fixed theirs; this module fixes the calendar one, so calendula, a
//! sync connector and any other reader project the same fields without
//! fetching a body.
//!
//! The item is the calendar object resource, not the component: RFC
//! 4791 4.1 keeps the components sharing a UID in one resource, so a
//! recurring series and its overrides are one item, summarised from
//! the master (the component carrying no RECURRENCE-ID).
//!
//! Times are carried verbatim, beside the TZID naming their zone, so a
//! reader re-derives an instant with its own time zone database rather
//! than trusting one the writer resolved. The single resolved
//! projection is the companion `sort_key`.

use chrono::{Datelike, NaiveDateTime, TimeDelta, Timelike};
use ical::{
    recur::IcalRecurDateTime,
    timezone::{IcalOffset, IcalTimezone},
    tree::{
        component::{vevent::VEVENT, vjournal::VJOURNAL, vtodo::VTODO},
        cst::{IcalCst, IcalItem},
        param::{tzid::TZID, value::VALUE},
        prop::{
            IcalPropSpec, dtend::DTEND, dtstart::DTSTART, due::DUE, location::LOCATION,
            rdate::RDATE, recurrence_id::RECURRENCE_ID, rrule::RRULE, summary::SUMMARY, uid::UID,
        },
    },
};
use io_replica::placement::{ReplicaLinkId, ReplicaMeta, ReplicaSortKey};
use serde::{Deserialize, Serialize};

use crate::shared::event::Event;

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
    /// The component the resource is rendered as (`VEVENT`, `VTODO`,
    /// `VJOURNAL`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    /// The item's SUMMARY. Required, and may be empty.
    #[serde(default)]
    pub summary: String,
    /// The item's LOCATION: the third thing an agenda row shows, after
    /// the time and the title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// DTSTART, the value verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtstart: Option<String>,
    /// The TZID parameter naming the zone DTSTART is local to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtstart_tzid: Option<String>,
    /// Whether DTSTART is a `date-time` or a `date`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtstart_value: Option<String>,
    /// DTEND, the value verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dtend: Option<String>,
    /// DUE, the value verbatim. A `VTODO` alone carries one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    /// Whether the item carries an RRULE or an RDATE, so a reader knows
    /// it must expand the recurrence to place its occurrences.
    #[serde(default)]
    pub recurring: bool,
    /// The UNTIL of the recurrence rule, when it is bounded. With
    /// `dtstart` it brackets the series, so a date range drops the item
    /// without materialising an occurrence. Absent means the bound is
    /// unknown: a rule bounded by COUNT states none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
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
    /// The item's start, normalised for ordering.
    pub sort_key: ReplicaSortKey,
}

/// Projects raw iCalendar bytes onto what the store needs to file them.
///
/// The link id is the resource's UID, which is what identifies the same
/// calendar object across sources (RFC 5545 3.8.4.7) and, per RFC 4791
/// 4.1, is unique within the collection holding it. Content carrying no
/// usable UID falls back to a content-derived id, so it still files
/// rather than being rejected.
pub fn project(contents: &[u8]) -> CalendarProjection {
    let Ok(cst) = IcalCst::parse(contents) else {
        return unsummarisable(contents);
    };

    let Some((component, master)) = master_component(&cst) else {
        return unsummarisable(contents);
    };

    let dtstart = stamp::<DTSTART>(master);
    let due = stamp::<DUE>(master);

    // NOTE: a VTODO is scheduled by its DUE date (RFC 5545 3.8.2.3) and
    // need not carry a DTSTART at all, so ordering it on DTSTART would
    // order most task lists on nothing.
    let start = match component {
        "VTODO" => due.as_ref().or(dtstart.as_ref()),
        _ => dtstart.as_ref(),
    };

    let sort_key = start
        .and_then(|start| normalize(&cst, start))
        .unwrap_or_default();

    let rrule = master.prop::<RRULE>().map(|rule| rule.0.into_owned());

    let meta = CalendarMeta {
        v: 1,
        uid: master.prop::<UID>().map(|uid| uid.0.into_owned()),
        component: Some(component.to_owned()),
        summary: master
            .prop::<SUMMARY>()
            .map(|text| text.0.into_owned())
            .unwrap_or_default(),
        location: master
            .prop::<LOCATION>()
            .map(|text| text.0.into_owned())
            .filter(|location| !location.is_empty()),
        dtstart: dtstart.as_ref().map(|stamp| stamp.value.clone()),
        dtstart_tzid: dtstart.as_ref().and_then(|stamp| stamp.tzid.clone()),
        dtstart_value: dtstart.as_ref().map(|stamp| stamp.value_type().to_owned()),
        dtend: stamp::<DTEND>(master).map(|stamp| stamp.value),
        due: due.map(|stamp| stamp.value),
        recurring: rrule.is_some() || master.prop::<RDATE>().is_some(),
        until: rrule.as_deref().and_then(rule_until),
        size: Some(contents.len() as u64),
    };

    let link_id = match &meta.uid {
        Some(uid) if !uid.trim().is_empty() => ReplicaLinkId(format!("uid:{}", uid.trim())),
        _ => ReplicaLinkId(format!("alt:{}:{}", meta.summary, sort_key)),
    };

    CalendarProjection {
        link_id,
        meta: ReplicaMeta(serde_json::to_string(&meta).unwrap_or_default()),
        sort_key: ReplicaSortKey(sort_key),
    }
}

/// Files content no summary can be read from: bytes that do not parse,
/// and a resource carrying no component this convention describes. The
/// item still lands, under an id derived from its own bytes, because an
/// item a reader shows blank beats one the store never received.
fn unsummarisable(contents: &[u8]) -> CalendarProjection {
    CalendarProjection {
        link_id: ReplicaLinkId(format!("alt:{:032x}", contents.len())),
        meta: ReplicaMeta(String::from("{\"v\":1,\"summary\":\"\"}")),
        sort_key: ReplicaSortKey::default(),
    }
}

/// The `UNTIL` of a recurrence rule, when the rule is bounded.
///
/// Read off the rule text rather than through a parser: `UNTIL` is one
/// `KEY=VALUE` part of it (RFC 5545 3.3.10), and nothing else here needs
/// the rule decoded.
fn rule_until(rule: &str) -> Option<String> {
    rule.split(';').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case("UNTIL")
            .then(|| value.trim().to_string())
    })
}

/// One date-time property as the calendar wrote it.
///
/// The decoded value alone drops the parameters that say how to read
/// it, and those are exactly what a reader without the body needs: the
/// zone the wall time is local to, and whether there is a time at all.
struct CalendarStamp {
    value: String,
    tzid: Option<String>,
    value_param: Option<String>,
}

impl CalendarStamp {
    /// Whether the value names a UTC instant already (RFC 5545 3.3.5
    /// form 2).
    fn is_utc(&self) -> bool {
        self.value.ends_with('Z')
    }

    /// The value type, from the `VALUE` parameter when the writer sent
    /// one and from the value's own shape otherwise, a date-time being
    /// the default (RFC 5545 3.2.20).
    fn value_type(&self) -> &str {
        match self.value_param.as_deref() {
            Some(param) if param.eq_ignore_ascii_case("DATE") => "date",
            Some(_) => "date-time",
            None if self.value.contains('T') => "date-time",
            None => "date",
        }
    }
}

/// The component a resource is summarised from, and its name.
///
/// A recurrence set is one resource (RFC 4791 4.1), so a listing
/// describes its master: the component carrying no RECURRENCE-ID, which
/// is the series itself rather than the one instance an override moved.
/// A resource holding overrides alone still projects, from the first of
/// them, since refusing to summarise it would hide the item entirely.
fn master_component<'c, 'a>(cst: &'c IcalCst<'a>) -> Option<(&'static str, &'c IcalCst<'a>)> {
    if let Some(component) = series(cst.components::<VEVENT>()) {
        return Some(("VEVENT", component));
    }

    if let Some(component) = series(cst.components::<VTODO>()) {
        return Some(("VTODO", component));
    }

    if let Some(component) = series(cst.components::<VJOURNAL>()) {
        return Some(("VJOURNAL", component));
    }

    None
}

/// The first component of a recurrence set carrying no RECURRENCE-ID,
/// falling back to the first of them.
fn series<'c, 'a>(components: impl Iterator<Item = &'c IcalCst<'a>>) -> Option<&'c IcalCst<'a>>
where
    'a: 'c,
{
    let mut first = None;

    for component in components {
        if component.prop::<RECURRENCE_ID>().is_none() {
            return Some(component);
        }

        first.get_or_insert(component);
    }

    first
}

/// Reads one date-time property with the parameters its decoded value
/// drops.
fn stamp<L: IcalPropSpec>(component: &IcalCst<'_>) -> Option<CalendarStamp> {
    let line = component.items.iter().find_map(|item| match item {
        IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case(&L::KIND) => Some(line),
        _ => None,
    })?;

    Some(CalendarStamp {
        value: line.raw_value_str().into_owned(),
        tzid: line.param::<TZID>().map(|tzid| tzid.into_owned()),
        value_param: line.param::<VALUE>().map(|value| value.into_owned()),
    })
}

/// Resolves a stamp onto the RFC 3339 UTC instant the sort key orders
/// on, at seconds precision (pimdir SPEC Annex A.3).
///
/// Only a UTC value is an instant already. A zoned one resolves through
/// the VTIMEZONE the document carries; a date-only one is read at
/// midnight and a floating one on the wall clock, both conventions
/// rather than facts, since neither names an instant at all.
fn normalize(cst: &IcalCst<'_>, stamp: &CalendarStamp) -> Option<String> {
    let local = Event {
        start: stamp.value.clone(),
        ..Default::default()
    }
    .start_at()?;

    let zoned = !stamp.is_utc() && stamp.value_type() != "date";
    let offset = match stamp.tzid.as_deref() {
        Some(tzid) if zoned => offset_at(cst, tzid, local),
        _ => 0,
    };

    let at = local - TimeDelta::seconds(offset as i64);

    Some(at.format("%Y-%m-%dT%H:%M:%SZ").to_string())
}

/// The offset in force at a local time, in seconds east of UTC.
///
/// A local time at a transition names two instants or none, so the
/// convention picks the earlier offset of a fold and the offset after a
/// gap. A zone the document does not define resolves to zero, which
/// reads the wall time as UTC exactly as a floating value: the error is
/// bounded by the offset and keeps the item near its place, where
/// dropping the key would move it to the far end of the listing. The
/// gcal backend makes that the common case (calendula#8), not a corner
/// one.
fn offset_at(cst: &IcalCst<'_>, tzid: &str, local: NaiveDateTime) -> i32 {
    let ical = cst.decode();

    let Some(zone) = IcalTimezone::of_calendar(&ical, tzid) else {
        return 0;
    };

    let local = IcalRecurDateTime {
        year: local.year(),
        month: local.month() as u8,
        day: local.day() as u8,
        hour: local.hour() as u8,
        minute: local.minute() as u8,
        second: local.second() as u8,
    };

    match zone.resolve(local) {
        IcalOffset::One(offset) => offset,
        IcalOffset::Gap { after, .. } => after,
        IcalOffset::Fold { earlier, .. } => earlier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENT: &str = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
        "BEGIN:VEVENT\r\nUID:event-1@example.org\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20260814T090000Z\r\nDTEND:20260814T100000Z\r\nSUMMARY:Stand-up\r\n",
        "LOCATION:Room 2\r\n",
        "END:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    /// A series whose 11 Aug instance moved to 14:00, in the one
    /// resource RFC 4791 4.1 requires it to share with its master.
    const SERIES: &str = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
        "BEGIN:VEVENT\r\nUID:standup@example.org\r\nDTSTAMP:20260101T000000Z\r\n",
        "RECURRENCE-ID:20260811T090000Z\r\nDTSTART:20260811T140000Z\r\n",
        "SUMMARY:Stand-up moved\r\nEND:VEVENT\r\n",
        "BEGIN:VEVENT\r\nUID:standup@example.org\r\nDTSTAMP:20260101T000000Z\r\n",
        "RRULE:FREQ=WEEKLY;BYDAY=TU;UNTIL=20261231T235959Z\r\nDTSTART:20190107T090000Z\r\n",
        "SUMMARY:Stand-up\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    /// A zoned event with the VTIMEZONE it needs: New York, standard
    /// time from November and daylight time from March.
    const ZONED: &str = concat!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
        "BEGIN:VTIMEZONE\r\nTZID:America/New_York\r\n",
        "BEGIN:STANDARD\r\nDTSTART:19701101T020000\r\n",
        "RRULE:FREQ=YEARLY;BYMONTH=11;BYDAY=1SU\r\n",
        "TZOFFSETFROM:-0400\r\nTZOFFSETTO:-0500\r\nEND:STANDARD\r\n",
        "BEGIN:DAYLIGHT\r\nDTSTART:19700308T020000\r\n",
        "RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=2SU\r\n",
        "TZOFFSETFROM:-0500\r\nTZOFFSETTO:-0400\r\nEND:DAYLIGHT\r\n",
        "END:VTIMEZONE\r\n",
        "BEGIN:VEVENT\r\nUID:zoned-1\r\nDTSTAMP:20260101T000000Z\r\n",
        "DTSTART;TZID=America/New_York:20260814T090000\r\nSUMMARY:Zoned\r\n",
        "END:VEVENT\r\nEND:VCALENDAR\r\n",
    );

    #[test]
    fn an_event_projects_its_uid_summary_and_verbatim_bounds() {
        let projection = project(EVENT.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        assert_eq!(projection.link_id.0, "uid:event-1@example.org");
        assert_eq!(projection.sort_key.0, "2026-08-14T09:00:00Z");
        assert_eq!(meta.v, 1);
        assert_eq!(meta.summary, "Stand-up");
        assert_eq!(meta.component.as_deref(), Some("VEVENT"));
        assert_eq!(meta.dtstart.as_deref(), Some("20260814T090000Z"));
        assert_eq!(meta.dtstart_value.as_deref(), Some("date-time"));
        assert!(meta.dtstart_tzid.is_none());
        assert_eq!(meta.location.as_deref(), Some("Room 2"));
        assert_eq!(meta.dtend.as_deref(), Some("20260814T100000Z"));
        assert!(!meta.recurring);
        assert_eq!(meta.size, Some(EVENT.len() as u64));
    }

    #[test]
    fn a_series_and_its_overrides_are_one_item_summarised_from_the_master() {
        let projection = project(SERIES.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        // The override comes first in the resource, so taking the first
        // component would summarise the item as the moved instance.
        assert_eq!(projection.link_id.0, "uid:standup@example.org");
        assert_eq!(meta.summary, "Stand-up");
        assert_eq!(meta.dtstart.as_deref(), Some("20190107T090000Z"));
        assert!(meta.recurring);
        // With `dtstart` this brackets the series, so a range read drops
        // it without expanding a single occurrence.
        assert_eq!(meta.until.as_deref(), Some("20261231T235959Z"));
        // The first occurrence, fixed for the life of the series: a
        // date-range read expands the recurrence above the store.
        assert_eq!(projection.sort_key.0, "2019-01-07T09:00:00Z");
    }

    #[test]
    fn a_zoned_start_resolves_through_the_vtimezone_it_carries() {
        let projection = project(ZONED.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        assert_eq!(meta.dtstart.as_deref(), Some("20260814T090000"));
        assert_eq!(meta.dtstart_tzid.as_deref(), Some("America/New_York"));
        // 09:00 in daylight time is 13:00 UTC, and the wall time stays
        // in `meta` for a reader with its own tz database.
        assert_eq!(projection.sort_key.0, "2026-08-14T13:00:00Z");
    }

    #[test]
    fn a_zone_the_document_does_not_define_reads_as_floating() {
        let orphan = ZONED.replace("America/New_York:20260814", "Europe/Paris:20260814");
        let projection = project(orphan.as_bytes());

        // Not '': an unresolvable zone keeps the item near its place
        // rather than at the far end of the listing.
        assert_eq!(projection.sort_key.0, "2026-08-14T09:00:00Z");
    }

    #[test]
    fn a_date_only_start_is_read_at_midnight() {
        let all_day = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VEVENT\r\nUID:all-day-1\r\nDTSTAMP:20260101T000000Z\r\n",
            "DTSTART;VALUE=DATE:20260811\r\nSUMMARY:Holiday\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let projection = project(all_day.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        assert_eq!(meta.dtstart.as_deref(), Some("20260811"));
        assert_eq!(meta.dtstart_value.as_deref(), Some("date"));
        assert_eq!(projection.sort_key.0, "2026-08-11T00:00:00Z");
    }

    #[test]
    fn a_todo_orders_on_its_due_date() {
        let todo = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VTODO\r\nUID:todo-1\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:Ship it\r\n",
            "DUE:20260820T170000Z\r\nEND:VTODO\r\nEND:VCALENDAR\r\n",
        );

        let projection = project(todo.as_bytes());
        let meta: CalendarMeta = serde_json::from_str(&projection.meta.0).unwrap();

        assert_eq!(projection.link_id.0, "uid:todo-1");
        assert_eq!(meta.component.as_deref(), Some("VTODO"));
        assert_eq!(meta.due.as_deref(), Some("20260820T170000Z"));
        assert!(meta.dtstart.is_none());
        assert!(meta.dtend.is_none());
        // A VTODO need not carry a DTSTART, so ordering it on one would
        // order most task lists on nothing.
        assert_eq!(projection.sort_key.0, "2026-08-20T17:00:00Z");
    }

    #[test]
    fn a_todo_with_no_date_at_all_stays_orderable() {
        let todo = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n",
            "BEGIN:VTODO\r\nUID:todo-2\r\nDTSTAMP:20260101T000000Z\r\nSUMMARY:Someday\r\n",
            "END:VTODO\r\nEND:VCALENDAR\r\n",
        );

        let projection = project(todo.as_bytes());

        // Unknown rather than nowhere: it lands at the head of an
        // ascending listing, where an undated item is visible.
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
