---
cairn: change
id: jmap-backend
status: active
created: 2026-10-01
---

# A JMAP backend

## Why

Calendula reaches CalDAV, Google and Microsoft calendars but not a JMAP server, while cardamum and himalaya already speak JMAP to the same accounts. A Fastmail or Stalwart user configures `[jmap]` once for mail and contacts and has to fall back to CalDAV for the calendar.

The pieces below exist. io-jmap covers JMAP for Calendars (draft-ietf-jmap-calendars), read-only so far: `Calendar/get`, `Calendar/changes`, `CalendarEvent/get`, `CalendarEvent/query`, `CalendarEvent/changes`. ical-rs converts iCalendar to JSCalendar (RFC 8984) and back, losslessly through its escape hatches, behind its `jscalendar` feature.

## What

A `jmap` cargo feature and backend serving the whole shared API, in the default set, with the same `[jmap]` configuration block as cardamum.

### Decisions

**1. One CalendarEvent is one item.** The item id is the CalendarEvent `id`, verbatim. A recurring event and its overrides are one JSCalendar Event whose `recurrenceOverrides` ical-rs turns into overriding VEVENTs, so a recurrence set is one resource with no folding, unlike gcal and msgraph. An event a server split out of a series (`baseEventId` set) is a different UID and stays a separate item.

**2. The projection is ical-rs plus a JMAP envelope, kept in calendula.** Reading strips the JMAP members (`id`, `calendarIds`, `isDraft`, `isOrigin`, `utcStart`, `utcEnd`, `baseEventId`) and hands the rest to `Ical::from_jscalendar`. Writing calls `to_jscalendar`, which returns a `Group`, and SHALL find exactly one entry in it: a calendar object resource holds one UID (RFC 4791 4.1), so zero or several is refused by name. The glue is a few dozen lines in `src/jmap/project.rs`, like cardamum's JSContact one. It moves to an `ical` feature of io-jmap the day a second consumer needs it (a neverest JMAP calendar source), as the gcal and Graph projections did for that reason; until then the move would be extraction without a second caller.

**3. Updates are a patch against a fresh server copy.** `CalendarEvent/set` takes a PatchObject. The update fetches the event, re-exports the edited item, and sends the top-level members that differ plus a `null` for each removed one, never touching the envelope (`calendarIds` survives) nor nulling `uid`. Diffing against the copy fetched now, not a base cached earlier, is safe because the import is lossless: a member absent from the edited export was removed by the user, not lost by the conversion.

**4. The etag is a hash of the event JSON**, as cardamum does, since JMAP has no per-object entity tag (only an account-wide state). `--if-match` SHALL be checked against the hash of the copy the update fetches, and the update refused when it moved, as msgraph does with `changeKey`. The check is client-side and racy by one round trip; `ifInState` would be server-enforced but fails on any change to any event of the account, which is worse.

**5. A date window is pushed down.** `list_items` with a range sends `CalendarEvent/query` with `inCalendar`, `after`, `before` and `expandRecurrences: false`, so the server narrows to series having an instance in the window and still returns one object per series. Pages map to `position` and `limit`.

**6. Events only.** JMAP for Calendars models JSCalendar events. `todo` and `journal` listings SHALL answer empty without a round trip and a create or update carrying no VEVENT SHALL be refused by component name, as on gcal and msgraph. JMAP Tasks is a separate draft io-jmap does not cover.

**7. Calendars are full CRUD.** JMAP keeps a name, a description and a CSS colour, so all three map without refusal. `create_calendar` reports the server-minted id (refused up front when the account's `mayCreateCalendar` is false). `delete_calendar` sets `onDestroyRemoveEvents: true`, matching CalDAV, where deleting a collection deletes what it holds.

**8. The session must advertise calendars.** Opening the backend SHALL fail naming `urn:ietf:params:jmap:calendars` when the session lacks it, rather than every call failing later with a method-level error. The calendar account is the session's primary account for that capability.

**9. Auto order.** JMAP is a standard protocol, so `auto` takes it after CalDAV and before the vendor APIs: vdir, pimdir, CalDAV, JMAP, gcal, msgraph.

### Prerequisites outside calendula

- **io-jmap writes**: `Calendar/set` and `CalendarEvent/set` (create, update as PatchObject, destroy with `onDestroyRemoveEvents`), mirroring `AddressBook/set` and `ContactCard/set`, then a release. `CalendarEvent/query` paging (`position`, `limit`) if the query options lack it.
- **ical-rs `UNTIL` shift**: ical-rs carries an `RRULE`'s `UNTIL` across unshifted, exact for floating and UTC series, off by the zone's offset for a zoned one (its own module doc says so). Over a JMAP backend that moves the end of every zoned series stored with a UTC `UNTIL`, gaining or losing its last instance, on read and on write. The shift belongs in ical-rs behind its `tzdb` feature, not in calendula, and lands before this backend writes.
- **io-pim-discovery 0.8** with `rfc8620`, for `GET /.well-known/jmap` on a bare server name (cardamum is on it, calendula on 0.7).

## To verify before coding

- Which servers expose `urn:ietf:params:jmap:calendars` today (Stalwart, Fastmail, Cyrus), and which draft revision each follows. A live target is needed for the manual test.
- Whether `after` and `before` are UTC or local date-times in the revision those servers follow (io-jmap documents local; the draft has moved on this).
- Where ical-rs puts a `VTIMEZONE` and the other `VCALENDAR`-level content on export: on the entry (`timeZones`, kept) or on the Group (lost with the Group, since a CalendarEvent has no calendar-level home). A custom zone lost there is a moved event.
- Whether servers accept a Task `@type` in `CalendarEvent/set`, which would let `todo` work on some of them. Not planned either way.

## Not in scope

- A protocol-specific `jmap` command family (msgraph has none either).
- The wizard proposing JMAP; a follow-up change, aligned with himalaya's and cardamum's.
- JMAP Tasks, push (`EventSource`), `CalendarEvent/changes`-based sync, participant scheduling (`sendSchedulingMessages`).
- Moving cardamum's JSContact projection into io-jmap.
