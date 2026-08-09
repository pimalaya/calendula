---
cairn: log
date: 2026-08-09
change: gcal-backend
---

# Google Calendar backend

calendula gained a fourth backend, `gcal`, adapting io-gcal's Calendar API v3 client. Google was reachable before only through its CalDAV bridge, which authenticates with a bearer token and nothing else, refuses `MKCALENDAR` and `MKCOL` outright, and enumerates no home-set, so an account was read-mostly and a calendar could not be created at all. The native API has none of those limits.

## What landed

`src/gcal/` carries the three usual pieces plus one the other backends do not need. `client.rs` resolves the bearer token (running its command when it is one) and opens the TLS connection to the fixed API base; there is no discovery step. `backend.rs` implements the nine shared operations: a calendar is a calendar list entry, an item is an event of it, item ids are the event ids verbatim, a create imports when the document carries a UID and inserts otherwise, and an update honours `if_match` as the API's `If-Match`. A `CalendarTimeRange` is pushed down as `timeMin` / `timeMax` after conversion to RFC 3339, and the listing walks `nextPageToken` only until the requested page is covered.

`project.rs` is the piece with no counterpart elsewhere. Google stores a JSON event and exposes no per-event iCalendar representation, so the document of record is synthesized on read and re-projected on write. Fields with a well-defined iCalendar slot are managed both ways; `created` and `updated` are managed on the way out only, since Google stamps them; Google-only fields survive an update untouched; `htmlLink`, `hangoutLink` and `conferenceData` are minted as read-only `X-GOOGLE-*` properties; and every remaining line is stashed verbatim in `extendedProperties.private`, chunked to Google's 1024-character ceiling, with an oversized line kept local rather than risking the write.

Two things the proposal did not anticipate:

- **The write is a merge, not a projection.** Google's event write replaces the whole resource, so omitting a provider-only field would clear it. An update therefore reads the current server event first and merges the projection onto it, which keeps the provider-only fields standing while the managed ones stay authoritative and a dropped property still clears its field. An extended property another client owns survives the same way.
- **The calendar level needs its own stash.** A VTIMEZONE, or an `X-WR-CALNAME`, cannot legally be spliced inside a VEVENT, and dropping a VTIMEZONE would leave every TZID reference dangling. The VEVENT remainder and the VCALENDAR remainder ride separate key families and are spliced back where they came from.

## What live testing changed

The backend was exercised against a real Google account across the whole shared surface, and two defects only the live API could surface:

- **A display zone was read as a wall time.** Google returns a boundary as an absolute instant *plus* the calendar's display zone: `dateTime: "2026-08-14T09:00:00Z"` with `timeZone: "Europe/Paris"`. The projection trusted the zone name and relabelled the literal time, shifting every event by the zone's offset, two hours in that case. Only the offset decides the instant now, and a `Z`-stamped boundary projects as a UTC stamp whatever zone is named beside it.
- **The display zone was then lost on write.** Since the projection cannot express the pair, writing the document back set the zone to UTC, and Google expands a recurring series in the zone of its start: every occurrence after a daylight-saving change would have drifted by an hour. The zone is a provider-only field in everything but name, so it is carried over from the server copy like the others. Only a UTC-stamped boundary may take it: an offset-less one is wall time in the zone it names, and relabelling that would move the event.

Relabelling a UTC instant into a named zone would need a time zone database, which the stack deliberately does not carry: ical-rs resolves offsets from a `VTIMEZONE` travelling inside the calendar, and Google sends none. Carrying the zone over is what avoids that dependency without losing the series.

Live testing also showed `create_calendar` reporting an id that does not exist, since Google mints its own and a warn-level log is not a report. The shared `create_calendar` now returns the identifier the backend assigned, as cardamum's `create_addressbook` already did, and the command reports that one.

Three shapes are refused by name rather than guessed at: a document with no VEVENT (Google models neither VTODO nor VJOURNAL), a floating DTSTART (the API needs a UTC offset or a named zone), and a VEVENT expressing its length as a DURATION rather than a DTEND. An alarm Google cannot model, meaning any action other than DISPLAY or EMAIL or any trigger that is not a whole-minute lead time, stays in the stash instead of being flattened into one it can hold.

`create_calendar` reports rather than honours the requested id, since Google mints its own, and `update_calendar` splits its patch across the two resources Google keeps a calendar in: the calendar itself for the title and the description, the calendar list entry for the colour.

Adding a fourth backend also made a gcal-only build possible, which the wizard has nothing to offer: it is now gated on the backends it actually configures, and bare `calendula` in such a build says so by name instead of walking an empty flow.

## Capabilities moved

- **backends**: added the Google Calendar backend, its pushed-down time range, its own pagination and its refusal by component name; the selection order became vdir, pimdir, CalDAV, gcal; a create now reports the identifier it was given.
- **projection**: new capability, ported from cardamum's spec of the same name, so both products treat provider quirks identically.
- **config**: added the `gcal` block (a token secret and TLS, no server and no discovery route), extended the secrets requirement to it, and wrote down that credentials nest under `auth` on every backend, one kind or several.
- **commands**: noted that a backend may exist without a protocol-specific family, and that projected bytes are the backends spec's business rather than the command layer's.
- **packaging**: `gcal` joined the per-backend feature list.
- **wizard**: written down that the wizard covers the discoverable backends only.

## Left out

The wizard entry for Google (its token broker story is the user's to settle) and a `gcal` protocol-specific family for what iCalendar cannot express: free/busy, colours, ACL, quick add and push channels. CalDAV's own module shows the end state, a backend and a family; gcal has the backend.

io-gcal is not published yet, so it rides a `[patch.crates-io]` git entry alongside the ones already there, to drop when it publishes.
