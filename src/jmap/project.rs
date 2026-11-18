//! # JSCalendar projection
//!
//! Projects a JMAP CalendarEvent (draft-ietf-jmap-calendars) onto an
//! iCalendar document and back, through ical-rs's JSCalendar conversion.
//!
//! The event's JSCalendar payload converts losslessly, what JSCalendar
//! cannot express riding ical-rs's escape hatches both ways. A recurring
//! event carries its overrides in `recurrenceOverrides`, which ical-rs
//! turns into overriding VEVENTs, so a series is one item. The JMAP
//! envelope ([`ENVELOPE`]) is kept out of the document both ways.
//!
//! JMAP has no per-event entity tag, so the version [`to_item`] surfaces
//! is the BLAKE3 digest of the event's JSON, the same from one build to
//! the next, which `--if-match` is compared with before a write.

use anyhow::{Context, Result, anyhow, bail};
use ical::{ical::Ical, param::IcalParam, tree::cst::IcalCst};
use io_jmap::calendars::calendar_event::{
    JmapCalendarEvent, ical::JmapCalendarEventIcalError, set::JmapCalendarEventPatch,
};
use jiff::tz::TimeZone;
use serde_json::{Map, Value};

use crate::shared::item::CalendarItem;

/// The members a CalendarEvent carries beside its JSCalendar payload,
/// which no document holds and no write sends as content.
pub const ENVELOPE: [&str; 7] = [
    "id",
    "calendarIds",
    "isDraft",
    "isOrigin",
    "utcStart",
    "utcEnd",
    "baseEventId",
];

/// Projects a CalendarEvent of `calendar_id` onto the shared item: its id
/// verbatim, the hash of its JSON as the etag, its JSCalendar payload as
/// the iCalendar contents.
pub fn to_item(calendar_id: &str, event: &JmapCalendarEvent) -> Result<CalendarItem> {
    let id = event
        .id
        .clone()
        .ok_or_else(|| anyhow!("JMAP CalendarEvent is missing its id"))?;

    Ok(CalendarItem {
        id,
        calendar_id: calendar_id.to_owned(),
        etag: Some(etag(event)),
        contents: to_ical(&event.event)?.into_bytes(),
        stand_in: None,
    })
}

/// The iCalendar document of a JSCalendar Event or Task, the envelope
/// stripped first.
///
/// Stands in for the `to_ical` io-jmap has not released yet.
pub fn to_ical(event: &Map<String, Value>) -> Result<String> {
    let mut event = event.clone();
    strip(&mut event);

    let event = Value::Object(event);
    let ical = Ical::from_jscalendar(&event).context("Cannot read the JMAP event as JSCalendar")?;

    Ok(ical.to_string())
}

/// The CalendarEvent of an iCalendar document, with no id and no
/// calendar.
///
/// A document carrying no VEVENT is refused by the components it does
/// carry, JMAP for Calendars storing events; one converting to anything
/// but one JSCalendar object, several UIDs or none, is refused as not one
/// event, a calendar object resource holding one UID (RFC 4791 4.1).
///
/// A time in a zone the time-zone database does not know is refused too:
/// ical-rs leaves a `VTIMEZONE` on the JSCalendar Group, not on the event,
/// so its definition would be lost and the event moved.
pub fn to_event(contents: &[u8]) -> Result<JmapCalendarEvent> {
    let cst = IcalCst::parse(contents).context("Cannot parse the iCalendar contents")?;
    let ical = cst.decode();

    let kinds: Vec<&str> = ical
        .components
        .iter()
        .map(|component| &*component.name)
        .filter(|name| !name.eq_ignore_ascii_case("VTIMEZONE"))
        .collect();

    if !kinds.iter().any(|name| name.eq_ignore_ascii_case("VEVENT")) {
        let carried = match kinds.as_slice() {
            [] => String::from("no component"),
            kinds => kinds.join(", "),
        };
        bail!(
            "The jmap backend stores events only: the iCalendar contents carry no VEVENT \
             but {carried}"
        );
    }

    let unknown = ical
        .components
        .iter()
        .flat_map(|component| &component.props)
        .flat_map(|prop| &prop.params)
        .find_map(|param| match param {
            IcalParam::TzId(tzid) if TimeZone::get(tzid).is_err() => Some(tzid),
            _ => None,
        });

    if let Some(tzid) = unknown {
        bail!(
            "The jmap backend names a zone by its IANA id and cannot store the definition of \
             `{tzid}`, which would move the event: give its times in an IANA zone or in UTC"
        );
    }

    let mut event = JmapCalendarEvent::from_ical(&ical).map_err(|err| match err {
        JmapCalendarEventIcalError::NotOneEvent => anyhow!(
            "The iCalendar contents are not one JMAP event: a JMAP event holds one UID, its \
             overrides included, and no other component"
        ),
        err => err.into(),
    })?;
    strip(&mut event.event);

    Ok(event)
}

/// The `CalendarEvent/set` patch from the server copy `base` to `staged`:
/// each top-level member that differs, an override of
/// `recurrenceOverrides` at a time, a `null` for each one removed, never
/// the envelope and never `uid`.
pub fn to_patch(staged: &JmapCalendarEvent, base: &JmapCalendarEvent) -> JmapCalendarEventPatch {
    let mut base = base.event.clone();
    strip(&mut base);

    let mut patch = JmapCalendarEventPatch::diff(&staged.event, &base);
    patch.0.retain(|key, _| !ENVELOPE.contains(&key.as_str()));
    patch
}

/// The version of a CalendarEvent: the BLAKE3 digest of its JSON, in hex.
///
/// serde_json maps are key-sorted, so the digest does not depend on the
/// order the server sent the members in, and BLAKE3 is a fixed function,
/// so a version printed by one build matches in another.
pub fn etag(event: &JmapCalendarEvent) -> String {
    let json = serde_json::to_string(event).unwrap_or_default();
    blake3::hash(json.as_bytes()).to_hex().to_string()
}

/// Removes the JMAP envelope from a JSCalendar payload.
fn strip(event: &mut Map<String, Value>) {
    for member in ENVELOPE {
        event.remove(member);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// A zoned weekly series with one occurrence moved, as a JMAP server
    /// answers it, envelope included.
    fn series() -> JmapCalendarEvent {
        serde_json::from_value(json!({
            "id": "E1",
            "calendarIds": {"C1": true},
            "isDraft": false,
            "isOrigin": true,
            "utcStart": "2026-10-01T07:00:00Z",
            "utcEnd": "2026-10-01T07:30:00Z",
            "@type": "Event",
            "uid": "standup",
            "title": "Standup",
            "start": "2026-10-01T09:00:00",
            "timeZone": "Europe/Paris",
            "duration": "PT30M",
            "recurrenceRule": {"@type": "RecurrenceRule", "frequency": "weekly"},
            "recurrenceOverrides": {
                "2026-10-08T09:00:00": {"start": "2026-10-08T10:00:00", "title": "Later"}
            },
        }))
        .unwrap()
    }

    /// A series reads as one document holding the master and its override,
    /// and that document writes back as the same JSCalendar payload: an
    /// update of an unedited read patches nothing.
    #[test]
    fn a_series_round_trips_through_its_document() {
        let base = series();
        let item = to_item("C1", &base).unwrap();
        let ical = String::from_utf8(item.contents.clone()).unwrap();

        assert_eq!(item.id, "E1");
        assert_eq!(ical.matches("BEGIN:VEVENT").count(), 2, "{ical}");
        assert!(ical.contains("RRULE:FREQ=WEEKLY"), "{ical}");
        assert!(ical.contains("RECURRENCE-ID"), "{ical}");
        assert!(ical.contains("SUMMARY:Later"), "{ical}");

        let staged = to_event(&item.contents).unwrap();
        let patch = to_patch(&staged, &base);
        assert!(patch.0.is_empty(), "{:?}", patch.0);
    }

    /// The envelope leaves no trace in the document, not even in the
    /// escape hatch, and none in the event the document writes.
    #[test]
    fn the_envelope_stays_out_of_the_document() {
        let item = to_item("C1", &series()).unwrap();
        let ical = String::from_utf8(item.contents.clone()).unwrap();

        for member in ENVELOPE {
            assert!(!ical.contains(member), "{member}: {ical}");
        }

        let event = to_event(&item.contents).unwrap();
        assert_eq!(event.id, None);
        assert!(event.calendar_ids.is_empty());
        for member in ENVELOPE {
            assert!(!event.event.contains_key(member), "{member}");
        }
    }

    /// An edit patches what it changed, a member it removed nulled, and
    /// neither `uid` nor the envelope.
    #[test]
    fn an_edit_patches_its_changes_and_nulls_its_removals() {
        let base = series();
        let ical = String::from_utf8(to_item("C1", &base).unwrap().contents).unwrap();
        let edited = ical
            .replacen("SUMMARY:Standup", "SUMMARY:Daily", 1)
            .replace("DURATION:PT30M\r\n", "");
        assert_ne!(edited, ical);

        let staged = to_event(edited.as_bytes()).unwrap();
        let patch = to_patch(&staged, &base);

        assert_eq!(patch.0.get("title"), Some(&json!("Daily")), "{:?}", patch.0);
        assert_eq!(patch.0.get("duration"), Some(&Value::Null), "{:?}", patch.0);
        assert!(!patch.0.contains_key("uid"), "{:?}", patch.0);
        for member in ENVELOPE {
            assert!(!patch.0.contains_key(member), "{member}");
        }
    }

    /// A document of two events, or of an event and a to-do, is no JMAP
    /// event; one carrying no VEVENT is refused by what it carries.
    #[test]
    fn a_document_that_is_not_one_event_is_refused() {
        const HEAD: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//EN\r\n";
        const EVENT: &str = "BEGIN:VEVENT\r\nUID:a\r\nDTSTAMP:20261001T000000Z\r\n\
            DTSTART:20261001T090000Z\r\nEND:VEVENT\r\n";
        const OTHER: &str = "BEGIN:VEVENT\r\nUID:b\r\nDTSTAMP:20261001T000000Z\r\n\
            DTSTART:20261002T090000Z\r\nEND:VEVENT\r\n";
        const TODO: &str = "BEGIN:VTODO\r\nUID:t\r\nDTSTAMP:20261001T000000Z\r\nEND:VTODO\r\n";
        const TAIL: &str = "END:VCALENDAR\r\n";

        for body in [format!("{EVENT}{OTHER}"), format!("{EVENT}{TODO}")] {
            let err = to_event(format!("{HEAD}{body}{TAIL}").as_bytes()).unwrap_err();
            assert!(err.to_string().contains("not one JMAP event"), "{err}");
        }

        let err = to_event(format!("{HEAD}{TODO}{TAIL}").as_bytes()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "The jmap backend stores events only: the iCalendar contents carry no VEVENT but \
             VTODO"
        );

        assert!(to_event(format!("{HEAD}{EVENT}{TAIL}").as_bytes()).is_ok());
    }

    /// The version is the BLAKE3 digest of the event's JSON, its members
    /// sorted whatever order the server sent them in, so it moves with any
    /// change and only then.
    #[test]
    fn the_version_is_the_digest_of_the_json() {
        let event = series();
        let reordered: JmapCalendarEvent =
            serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
        assert_eq!(etag(&event), etag(&reordered));

        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            etag(&event),
            blake3::hash(json.as_bytes()).to_hex().to_string()
        );

        let mut moved = event.clone();
        moved.event.insert("title".into(), json!("Daily"));
        assert_ne!(etag(&event), etag(&moved));
    }

    /// A zoned series with its VTIMEZONE and an override writes as one
    /// event, the override folded into it and the zone named by its IANA
    /// id; the same series in a zone the database does not know is
    /// refused, its definition having no home on the event.
    #[test]
    fn a_zone_is_written_by_its_iana_id_and_a_custom_one_refused() {
        let series = |tzid: &str| {
            format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//EN\r\n\
                 BEGIN:VTIMEZONE\r\nTZID:{tzid}\r\nBEGIN:STANDARD\r\n\
                 DTSTART:19700101T000000\r\nTZOFFSETFROM:+0300\r\nTZOFFSETTO:+0300\r\n\
                 END:STANDARD\r\nEND:VTIMEZONE\r\n\
                 BEGIN:VEVENT\r\nUID:z\r\nDTSTAMP:20261001T000000Z\r\n\
                 DTSTART;TZID={tzid}:20261001T090000\r\nDURATION:PT1H\r\n\
                 RRULE:FREQ=WEEKLY\r\nSUMMARY:Zoned\r\nEND:VEVENT\r\n\
                 BEGIN:VEVENT\r\nUID:z\r\nDTSTAMP:20261001T000000Z\r\n\
                 RECURRENCE-ID;TZID={tzid}:20261008T090000\r\n\
                 DTSTART;TZID={tzid}:20261008T100000\r\nDURATION:PT1H\r\n\
                 SUMMARY:Zoned later\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
            )
        };

        let event = to_event(series("Europe/Istanbul").as_bytes()).unwrap();
        assert_eq!(event.event["timeZone"], "Europe/Istanbul");
        let overrides = event.event["recurrenceOverrides"].as_object().unwrap();
        assert_eq!(overrides.len(), 1, "{overrides:?}");

        let err = to_event(series("Custom/Zone").as_bytes()).unwrap_err();
        assert!(err.to_string().contains("`Custom/Zone`"), "{err}");
    }
}
