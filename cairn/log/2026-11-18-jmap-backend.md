---
cairn: log
change: jmap-backend
landed: 2026-11-18
---

# A JMAP backend

Landed the code and spec of [jmap-backend](../changes/jmap-backend/proposal.md): a `jmap` cargo feature, in the default set, and a backend serving the whole unified API over JMAP for Calendars (draft-ietf-jmap-calendars-27, as io-jmap 0.10 names it), with the `[jmap]` block of cardamum and Himalaya. The change stays active: two upstream prerequisites and the manual server test are open (below).

What landed, decision by decision of the proposal:

1. One CalendarEvent is one item, keyed by its `id` verbatim, a series carrying its overrides in `recurrenceOverrides`.
2. src/jmap/project.rs holds the projection: the read strips the envelope and converts through `Ical::from_jscalendar` (`project::to_ical`, standing in for io-jmap's unreleased `to_ical`), the write through io-jmap's `JmapCalendarEvent::from_ical` (JSCalendar 2.0, one Group entry or refused as not one event), the envelope stripped again.
3. An update reads the event and sends io-jmap's `JmapCalendarEventPatch::diff` against it: each top-level member that differs, each override on its own, `null` for a removed member, never the envelope nor `uid`; an empty patch sends nothing.
4. The etag is the BLAKE3 digest of the event's JSON; `--if-match` is checked against the copy the update or delete reads, `precondition-failed` before any set.
5. A range goes as the `after`/`before` of `CalendarEvent/query`, recurrences unexpanded, sorted on `start` then `uid`, a page as `position`/`limit`, a short page followed while `total` says more match.
6. Events only: `todo` and `journal` listings answer empty without a request, a write carrying no VEVENT is refused naming what it carries.
7. Calendars are full CRUD: `Calendar/get`, and `Calendar/set` through io-jmap's `send_raw` (src/jmap/calendar_set.rs, io-jmap 0.10 having none), `mayCreateCalendar: false` refused before any request, a destroy with `onDestroyRemoveEvents: true`.
8. Opening the backend fails naming `urn:ietf:params:jmap:calendars` when the session lacks it or names no primary account for it; `account check` opens the session then lists the calendars.
9. `auto` order is vdir, pimdir, CalDAV, jmap, gcal, msgraph.

Found while building: ical-rs 0.7.1 puts a `VTIMEZONE` in the JSCalendar Group's `iCalendar` hatch, not on the entry (2.0 retires `timeZones`), so `from_ical` loses a custom zone's definition. The backend refuses a write naming a zone the time-zone database does not know rather than move the event. The `after`/`before` form was settled from io-jmap's own docs (a local date-time in the query's `timeZone`, default `Etc/UTC`), not against a server.

Left: the ical-rs `UNTIL` shift (a zoned series' `UNTIL` still crosses unshifted, which projection.md states), io-jmap's `Calendar/set` and `to_ical` released and swapped in for the stopgaps, io-pim-discovery (calendula has none, a bare server name reaches `/.well-known/jmap` through io-jmap alone), the zone gap above, a `jmap` command namespace (out of scope), and the manual test against a live JMAP calendar server.

Capabilities moved: backends, projection, config.
