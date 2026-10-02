---
cairn: delta
change: jmap-backend
---

## ADDED Requirements

### Requirement: JMAP backend
jmap SHALL adapt io-jmap's JMAP for Calendars surface (draft-ietf-jmap-calendars) over a session advertising `urn:ietf:params:jmap:calendars`, failing at open and naming the capability when it does not. A calendar is a JMAP Calendar, its name, description and colour mapped both ways. An item is one CalendarEvent, keyed by its `id` verbatim, a recurring event carrying its overrides in itself.

`create_calendar` SHALL report the server-minted id and be refused when the account may not create calendars. `delete_calendar` SHALL remove the calendar's events with it, as on CalDAV.

### Requirement: jmap pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach jmap as the `after` and `before` filter of `CalendarEvent/query`, recurrences unexpanded, so the server narrows and still returns one object per series. Pages SHALL map to the query's `position` and `limit`.

### Requirement: jmap refuses what JMAP for Calendars cannot model
JMAP for Calendars stores events. A `todo` or `journal` listing SHALL be answered empty without a round trip, and a write carrying no VEVENT SHALL be refused by component name.

### Requirement: jmap writes a patch and checks its own etag
An item's etag SHALL be a hash of the CalendarEvent's JSON, JMAP having no per-object entity tag. `update_item` SHALL fetch the event, refuse when `if_match` is given and differs from that copy's hash, and send `CalendarEvent/set` a patch of the top-level members that differ from it, a `null` for each removed one, never the JMAP envelope and never `uid`.

### Requirement: JMAP converts through JSCalendar
(projection.md) The jmap projection SHALL convert a CalendarEvent's JSCalendar payload (RFC 8984) through ical-rs's conversion, both ways, the JMAP envelope (`id`, `calendarIds`, `isDraft`, `isOrigin`, `utcStart`, `utcEnd`, `baseEventId`) kept out of the document. What JSCalendar cannot express rides ical-rs's escape hatches, so the document round-trips. A document converting to anything but one JSCalendar object SHALL be refused.

### Requirement: The jmap block
(config.md) An account MAY carry a `jmap` block with `server`, `tls`, `alpn`, `proxy` and `auth`, shaped as cardamum's and himalaya's, so one JMAP account reads the same in all three. A bare server name SHALL be discovered through `/.well-known/jmap`.

## MODIFIED Requirements

### Requirement: Backend selection order
`auto` takes jmap after CalDAV and before gcal: vdir, pimdir, CalDAV, jmap, gcal, msgraph.

### Requirement: Native backends do not project
(projection.md) jmap joins gcal and msgraph as a projecting backend.

## REMOVED Requirements
