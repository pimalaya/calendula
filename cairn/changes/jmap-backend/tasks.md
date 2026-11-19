---
cairn: tasks
change: jmap-backend
---

# Tasks

## 0. Verify

- [ ] Find a server advertising `urn:ietf:params:jmap:calendars` and note its draft revision; it is the manual-test target. Not done: the 2026-11-18 run touched no network.
- [x] Settle the `after`/`before` date form against that revision; fix io-jmap's doc if it is UTC. Settled from io-jmap 0.10 alone (draft-ietf-jmap-calendars-27, src/calendars.rs): its filter docs say local date-time, read in the query's `timeZone`, default `Etc/UTC`, so the backend sends the UTC instant without its `Z`. Still to confirm against the server of the manual test.
- [x] Export a zoned recurring VEVENT with a custom `VTIMEZONE` and one override through `to_jscalendar`: confirm one Group entry, overrides in `recurrenceOverrides`, the zone on the entry. Whatever lands on the Group is a gap to fix in ical-rs first. One entry and the override fold as expected; the `VTIMEZONE` lands in the Group's `iCalendar` hatch (2.0 retires `timeZones`), lost with the Group. Gap left to ical-rs/io-jmap; calendula refuses a zone the time-zone database does not know meanwhile.

## 1. Upstream

- [x] ical-rs: shift `UNTIL` between UTC and the series' zone when `tzdb` is on, both directions, with tests for a zoned, a UTC and a floating series; release. Done: ical-rs 0.7.2 (released 2026-11-19), which calendula moved onto with `tzdb` on; `a_zoned_series_until_round_trips` (src/jmap/backend.rs) reads a zoned series' `until` as its UTC `UNTIL` and writes it back local, failing on 0.7.1. A UTC `UNTIL` in a fold's second pass comes back as the first pass, and a zone neither the calendar nor the database knows travels unshifted (projection.md).
- [x] io-jmap: `Calendar/set` (create, update, destroy, `onDestroyRemoveEvents`) and `CalendarEvent/set` (create, PatchObject update, destroy) coroutines and client methods, modelled on `AddressBook/set` and `ContactCard/set`; query `position`/`limit` if missing; release. Done: io-jmap 0.10 carried `CalendarEvent/set`, `JmapCalendarEventPatch::diff` and the query paging; io-jmap 0.10.1 (released 2026-11-18) adds `Calendar/set` and `JmapCalendarEvent::to_ical`, which calendula uses since the-releases-land, its own `send_raw` copy (src/jmap/calendar_set.rs) and `project::to_ical` removed.
- [x] calendula: io-pim-discovery 0.8 with `rfc8620`. Dropped 2026-11-19: the-discover-route-leaves (owner decision of 2026-11-15) removed io-pim-discovery and every DNS resolution, so RFC 8620 §2.2 SRV discovery would undo that ruling; calendula carries no io-pim-discovery; a bare server name reaches `/.well-known/jmap` through io-jmap's `session_get`, and no DNS discovery runs.

## 2. Backend

- [x] Cargo: `jmap = ["dep:io-jmap", "io-jmap/client", "ical-rs/jscalendar", "io-pim-discovery/rfc8620"]`, in `default`; TLS and `vendored` forwards. Built as `["dep:io-jmap", "dep:blake3", "dep:secrecy", "io-jmap/client", "io-jmap/ical", "ical-rs/jscalendar"]`, without discovery.
- [x] Config: `JmapConfig` (`server`, `tls`, `alpn`, `proxy`, `auth`) identical to cardamum's, `AccountConfig.jmap`, config.sample.toml.
- [x] `Backend::Jmap`, `allows_jmap`, auto order vdir, pimdir, CalDAV, JMAP, gcal, msgraph.
- [x] `src/jmap/project.rs`: `to_item` (envelope stripped, `from_jscalendar`), `to_event` (`to_jscalendar`, one Group entry or refusal), `to_patch`, `etag`; unit tests on round trip, envelope, refusals, patch nulls.
- [x] `src/jmap/backend.rs`: session open with the capability check, the nine unified operations per the proposal, VEVENT-only refusals, `--if-match` against the fetched hash.
- [x] `CalendarClient` arm, account check and account list.
- [x] Feature matrix: `jmap` alone, default, `--all-features`; clippy, fmt. CI's `features` job and CONTRIBUTING gain `rustls-ring,jmap`.

## 3. Land

- [x] Fold the delta into backends.md, projection.md and config.md; log entry; status `landed`. Folded and logged; status left `active` while the upstream tasks and the manual test are open.
- [x] README backend list, CHANGELOG `Added`.
- [ ] Manual test against the server from 0: calendar CRUD, a recurring event with an override round-tripped through `event update -i`, a range listing, a refused `todo create`. Not done: needs a live account.
