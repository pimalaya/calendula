---
cairn: tasks
change: jmap-backend
---

# Tasks

## 0. Verify

- [ ] Find a server advertising `urn:ietf:params:jmap:calendars` and note its draft revision; it is the manual-test target.
- [ ] Settle the `after`/`before` date form against that revision; fix io-jmap's doc if it is UTC.
- [ ] Export a zoned recurring VEVENT with a custom `VTIMEZONE` and one override through `to_jscalendar`: confirm one Group entry, overrides in `recurrenceOverrides`, the zone on the entry. Whatever lands on the Group is a gap to fix in ical-rs first.

## 1. Upstream

- [ ] ical-rs: shift `UNTIL` between UTC and the series' zone when `tzdb` is on, both directions, with tests for a zoned, a UTC and a floating series; release.
- [ ] io-jmap: `Calendar/set` (create, update, destroy, `onDestroyRemoveEvents`) and `CalendarEvent/set` (create, PatchObject update, destroy) coroutines and client methods, modelled on `AddressBook/set` and `ContactCard/set`; query `position`/`limit` if missing; release.
- [ ] calendula: io-pim-discovery 0.8 with `rfc8620`.

## 2. Backend

- [ ] Cargo: `jmap = ["dep:io-jmap", "io-jmap/client", "ical-rs/jscalendar", "io-pim-discovery/rfc8620"]`, in `default`; TLS and `vendored` forwards.
- [ ] Config: `JmapConfig` (`server`, `tls`, `alpn`, `proxy`, `auth`) identical to cardamum's, `AccountConfig.jmap`, config.sample.toml.
- [ ] `Backend::Jmap`, `allows_jmap`, auto order vdir, pimdir, CalDAV, JMAP, gcal, msgraph.
- [ ] `src/jmap/project.rs`: `to_item` (envelope stripped, `from_jscalendar`), `to_event` (`to_jscalendar`, one Group entry or refusal), `to_patch`, `etag`; unit tests on round trip, envelope, refusals, patch nulls.
- [ ] `src/jmap/backend.rs`: session open with the capability check, the nine shared operations per the proposal, VEVENT-only refusals, `--if-match` against the fetched hash.
- [ ] `CalendarClient` arm, account check and account list.
- [ ] Feature matrix: `jmap` alone, default, `--all-features`; clippy, fmt.

## 3. Land

- [ ] Fold the delta into backends.md, projection.md and config.md; log entry; status `landed`.
- [ ] README backend list, CHANGELOG `Added`.
- [ ] Manual test against the server from 0: calendar CRUD, a recurring event with an override round-tripped through `event update -i`, a range listing, a refused `todo create`.
