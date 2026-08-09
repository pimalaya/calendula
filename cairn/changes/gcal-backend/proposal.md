---
cairn: change
id: gcal-backend
status: landed
created: 2026-08-09
---

# Google Calendar backend

## Why

calendula speaks CalDAV, vdir and pimdir. Google Calendar is reachable over CalDAV, but only in a crippled form: the account authenticates with an OAuth2 bearer token and nothing else, `MKCALENDAR` and `MKCOL` are refused outright, and the discovery entry point behaves off-spec. A Google account is therefore read-mostly through the CalDAV backend, and calendar creation is impossible.

The Calendar API v3 has none of those limits, and [io-gcal](https://github.com/pimalaya/io-gcal) now covers its whole surface: 38 methods over 8 resources, live-tested against the real API. Adding it as a fourth backend makes Google a first-class account instead of a degraded CalDAV one.

## What

A `gcal` backend adapting io-gcal's `GcalClientStd`, in the shape every other backend has: `src/gcal/backend.rs`, a `GcalConfig` block, a variant in `BackendClient` and in the `--backend` enum.

Two things do not follow the existing backends, and both already have an answer elsewhere in the project.

### The document of record is synthesized

The shared `CalendarItem.contents` is documented as raw iCalendar bytes, exactly as the backend stored them. Google stores a JSON `Event` and exposes no per-event iCalendar representation at all, so that invariant cannot hold for this backend.

cardamum met exactly this problem three times over (JMAP, Microsoft Graph, Google People) and settled it in its own `cairn/spec/projection.md`: a backend with no native document representation *projects* its wire resource onto a document of record and re-projects on the way back, and the shared type's documentation names the exception. `Card.contents` says so out loud. This change ports that policy to calendars, requirement for requirement:

- **managed** fields have a well-defined iCalendar slot and are authoritative both ways, so clearing the property clears the field;
- **provider-only** fields (`colorId`, `eventType`, `guestsCanModify`, `birthdayProperties`, `workingLocationProperties`, ...) stay out of every write and survive untouched;
- **minted** fields that mean nothing outside Google (`htmlLink`, `hangoutLink`, `conferenceData`) become read-only `X-GOOGLE-*` properties, dropped on write;
- the **remainder**, every line the projection neither manages nor mints, is stashed verbatim and spliced back on read.

The stash slot is `extendedProperties.private`, the Calendar analogue of People's `clientData`. It differs in one way that matters: Google caps an extended property value at 1024 characters, so unlike cardamum's single slot the remainder has to be chunked across numbered keys, and a line too long to fit stays local rather than risking the write.

Only VEVENT projects. A VTODO or VJOURNAL has no Google equivalent, so `create_item` refuses it by name, which is what the backends spec already demands of an operation a backend cannot model.

### The bearer token comes from a command

Google issues an access token that expires within the hour, so a token written into the configuration is useless by the second run. calendula already has the answer in `CaldavAuthConfig::Bearer`, whose documentation says it plainly: an OAuth 2.0 token broker is a command like any other. `GcalConfig` takes the same `Secret`, so `gcloud`, ortie or any refresh script fits with no OAuth machinery inside calendula.

## Scope

In: the backend, the projection, the configuration block, the dispatcher and `--backend` wiring, the spec and the tests.

Out, deliberately: the wizard (a Google entry needs the token-broker story settled first, and the wizard capability is specified separately), and a `gcal` protocol-specific command family for what iCalendar cannot express (free/busy, colours, ACL, quick add, push channels). Both are natural follow-ups, and CalDAV's own module shows the end state: a backend *and* a protocol-specific family.

## Dependency

io-gcal is not published yet, so this rides a `[patch.crates-io]` git entry alongside the ones already there for io-webdav, io-pimdir and io-replica, dropped when it publishes.

One gap in io-gcal blocks full parity and is fixed there, not here: the shared `update_item` carries an `if_match` etag, CalDAV honours it, and Google supports `If-Match` on event writes, but io-gcal's send primitive has no way to set the header. Until it does, the backend would have to ignore `if_match` the way vdir does, which is a silent loss of the optimistic concurrency Google actually offers. Adding an optional `If-Match` to io-gcal's write methods is a small change and should land first.
