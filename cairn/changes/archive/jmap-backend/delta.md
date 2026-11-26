---
cairn: delta
change: jmap-backend
---

## ADDED Requirements

### Requirement: JMAP backend
(backends.md) jmap SHALL adapt io-jmap's JMAP for Calendars surface (draft-ietf-jmap-calendars) over a session advertising `urn:ietf:params:jmap:calendars` and naming a primary account for it, failing at open, `account check` included, naming the capability when it does not. A calendar is a JMAP Calendar, its name, description and colour mapped both ways. An item is one CalendarEvent, keyed by its `id` verbatim, a recurring event carrying its overrides in itself.

io-jmap 0.10 has no `Calendar/set`, so the calendar writes SHALL go as a `Calendar/set` request calendula builds and sends through `send_raw`. `create_calendar` SHALL report the server-minted id and be refused `permission-denied` before any request when the account states `mayCreateCalendar: false`. `delete_calendar` SHALL remove the calendar's events with it (`onDestroyRemoveEvents: true`), as on CalDAV. A read, an update or a delete SHALL act only on an event whose `calendarIds` hold the calendar named, `item-not-found` otherwise, and a listing of a calendar the account does not hold SHALL fail `calendar-not-found`.

### Requirement: jmap pushes the time range down
(backends.md) A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach jmap as the `after` and `before` filter of `CalendarEvent/query`, recurrences unexpanded, so the server narrows and still returns one object per series. The bounds SHALL go as the local date-times of their UTC instants, the query's `timeZone` left to the server's default `Etc/UTC`, as io-jmap documents the filter. The query SHALL sort on `start` then `uid`, and pages SHALL map to its `position` and `limit`, a short page followed.

### Requirement: jmap refuses what JMAP for Calendars cannot model
(backends.md) JMAP for Calendars stores events. A `todo` or `journal` listing SHALL be answered empty without a round trip, and a write carrying no VEVENT SHALL be refused by component name. A write naming a zone the time-zone database does not know SHALL be refused naming it, ical-rs leaving a `VTIMEZONE` on the JSCalendar Group, where it would be lost.

### Requirement: jmap writes a patch and checks its own etag
(backends.md) An item's etag SHALL be the BLAKE3 digest of the CalendarEvent's JSON, JMAP having no per-object entity tag. `update_item` SHALL fetch the event, refuse with `precondition-failed` when `if_match` is given and differs from that copy's digest, and send `CalendarEvent/set` a patch of the top-level members that differ from it (io-jmap's `JmapCalendarEventPatch::diff`, each override on its own), a `null` for each removed one, never the JMAP envelope and never `uid`.

### Requirement: JMAP converts through JSCalendar
(projection.md) The jmap projection SHALL convert a CalendarEvent's JSCalendar payload (RFC 8984) through ical-rs's conversion, both ways: `Ical::from_jscalendar` on read, behind calendula's `project::to_ical` until io-jmap releases its own, io-jmap's `JmapCalendarEvent::from_ical` on write. The JMAP envelope (`id`, `calendarIds`, `isDraft`, `isOrigin`, `utcStart`, `utcEnd`, `baseEventId`) is kept out of the document. What JSCalendar cannot express rides ical-rs's escape hatches, so the document round-trips. A document converting to anything but one JSCalendar object SHALL be refused. An `RRULE`'s `UNTIL` is carried across unshifted (ical-rs 0.7).

### Requirement: The jmap block
(config.md) An account MAY carry a `jmap` block with `server`, `tls`, `alpn`, `proxy` and `auth`, shaped as cardamum's and himalaya's, so one JMAP account reads the same in all three. A bare server name SHALL be discovered through `/.well-known/jmap` (io-jmap's session fetch), no DNS discovery running.

## MODIFIED Requirements

### Requirement: Backend selection order
(backends.md) `auto` takes jmap after CalDAV and before gcal: vdir, pimdir, CalDAV, jmap, gcal, msgraph.

### Requirement: Native backends do not project
(projection.md) jmap joins gcal and msgraph as a projecting backend.

### Requirement: Backend blocks
(config.md) The `jmap` block joins the backend blocks, its `proxy` the account proxy's slots and its `alpn` the `http/1.1` defaults.

## REMOVED Requirements
