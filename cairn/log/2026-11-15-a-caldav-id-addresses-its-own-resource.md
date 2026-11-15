---
cairn: log
change: a-caldav-id-addresses-its-own-resource
landed: 2026-11-15
---

# A CalDAV id addresses its own resource

calendula moves to io-webdav 0.9.1 and refuses, before the account's secret is resolved or anything connects, a CalDAV calendar or item id that would address the home set, its parent or the server root: `calendar delete ..` could send a DELETE to the home set's parent on 0.9.0, and on 0.9.1 would fail with an uncoded I/O error once connected. calendula's part of line W2 of the CLI alignment round, cardamum's being a-carddav-id-addresses-its-own-resource. Capabilities **backends** and **commands** moved.

- **Cargo.toml, Cargo.lock**: io-webdav 0.9.1, the requirement `0.9.1`, the lock diff that crate alone.
- **src/caldav/client.rs**: `check_calendar_id` and `check_item_id` run io-webdav's `check_collection_id` and `check_resource_id`, a refused id failing `calendar-not-found` (``Calendar `<id>` not found``) or `item-not-found` (``Item `<id>` not found``), io-webdav's reason in `sources`; `check_new_calendar_id` refuses the id of a calendar to create uncoded (``Calendar id `<id>` names no CalDAV collection``). `build_caldav_client` connects from the block `caldav_config` reads, so a `caldav` command checks in between. The hand-run `proppatch` is untouched.
- **src/shared/client.rs**: `CalendarClient::check_ids` and `check_new_id` run them on a CalDAV account before the backend opens, in `calendar create`, `update` and `delete` and every item call, ahead of the MCP invitation check.
- **src/caldav/cli.rs, src/cli.rs**: `CaldavCommand::check` runs them once the account is resolved, before connecting (`caldav create`, `caldav delete`).
- **src/caldav/backend.rs**: the PROPPATCH clearing a property goes to `rfc4918::collection_path`, where it took `rfc4791::calendar::join_path`, the same function in 0.9.1.
- **Code**: the not-found codes rather than `usage-invalid`, as W2's line rules and cardamum and carillon code the same ids; a new id uncoded, as vdir's `calendar create`. `event delete` keeps its own sentence for an item it cannot find, without `sources`.
- **Through io-webdav 0.9.1**: an item listed keeps its final CRLF (W3), a relative id is sent percent-encoded (W4). calendula read no `calendar-data` through a trimming `text()` of its own.
- **Evidence**: tests/caldav_id.rs runs the binary over a CalDAV account at 127.0.0.1:9 whose password command creates a file. On io-webdav 0.9.1 before the checks, `a_calendar_id_out_of_the_home_is_not_found`, `an_item_id_out_of_its_calendar_is_not_found` and `a_new_calendar_id_out_of_the_home_is_refused` failed (the file created, ``connect 127.0.0.1:9`` uncoded); `a_clean_id_goes_on_to_open_the_account` passes on both. `a_caldav_listing_keeps_the_whole_item` (src/caldav/backend.rs) pins io-webdav's W3, whose own test fails on a trimmed read.

CHANGELOG (Changed, Fixed). Spec: backends ("A CalDAV id addresses its own resource", "A CalDAV item reads whole", added; "CalDAV backend", modified), commands (the `item-not-found` and `calendar-not-found` rows of "A JSON error carries a stable code", modified).
