---
cairn: log
change: a-sync-report-names-what-it-could-not-read
landed: 2026-11-24
---

# A sync report names what it could not read

io-webdav 0.14.0 returns the member rows a `sync-collection` REPORT or its `PROPFIND` fallback answered with another non-2xx status than 404 and 507 (a 403, a 5xx) as `WebdavSyncDelta::unreadable`, where it dropped them; its docs ask a consumer reading the delta as a full listing to read it as incomplete. `caldav report sync` printed `changed` and `vanished` alone, so such a member was missing from its listing, which `--fallback` prints as a full snapshot to diff: the diff read it as removed. Owner's standing instruction (go with the recommended option), as carillon and the Android app already treat these rows.

- **Behaviour**: `caldav report sync` prints `errors`, one entry per unreadable row, coded by `caldav::backend::unreadable_row` as `caldav report query` codes a listing row (a 403 `permission-denied`, `item-unreadable` when no status names a code), neither changed nor vanished; the text prints them after the sync token; a non-empty `errors` exits 3 (`partial_when`).
- **Other readers**: `report sync` is calendula's one reader of a `WebdavSyncDelta`. The pimdir backend reads the store and runs no sync; `caldav propfind <CALENDAR>` enumerates through `enum_items`, whose `CaldavItemEnumOk` keeps a row answered with an error as a reference with no entity tag, so it infers no removal either.
- **Test**: `caldav::report::sync::tests::a_member_the_server_refused_is_an_error_and_exits_3`, a scripted server answering a change, a 403 and a 404, failed on 981325e (`called Result::unwrap_err() on an Ok value: ()`, src/caldav/report/sync.rs:230).

Capabilities moved: commands (`caldav report sync` prints `errors` and exits 3).
