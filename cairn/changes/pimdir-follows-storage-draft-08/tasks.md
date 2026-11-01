---
cairn: tasks
change: pimdir-follows-storage-draft-08
---

# Tasks

- [x] The 2026-10-30 work committed first (user, 2026-11-01)
- [x] The compile port; Cargo.lock moves io-pimdir alone; `cargo test` green
- [x] The to-do gate on `calendar.component.vtodo`: a source stating `none` refuses, one stating nothing accepts; tests failing first
- [x] Reply and cancel payloads refused as the schemas refuse them; tests failing first
- [x] Error vocabulary and messages (`store-stale`, `store-newer` out; `store-version`, `store-exposed`, `flag-invalid`, `action-malformed` in)
- [x] End to end on a scratch store neverest synced from a test account: event and todo list, create, update, delete, reply queued, drained by neverest; an events-only calendar refusing a todo (2026-11-01, on local Stalwart IMAP and Radicale DAV rather than a provider test account: every queued write reached its server after `neverest sync`); the events-only refusal is covered by the unit tests, not this run
- [x] Pin to io-pimdir bff5737 by git rev, path patch dropped
- [x] Fold into spec/backends.md, spec/commands.md, MIGRATION.md, CHANGELOG; log
- [x] Land
