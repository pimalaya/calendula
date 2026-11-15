---
cairn: tasks
change: a-caldav-id-addresses-its-own-resource
---

# Tasks

- [x] io-webdav 0.9.1 in Cargo.lock (that crate alone) and Cargo.toml (`0.9.1`).
- [x] `check_calendar_id`, `check_item_id` and `check_new_calendar_id` in src/caldav/client.rs; the unified client checks before opening a CalDAV backend (src/shared/client.rs `check_ids`, `check_new_id`, ahead of the MCP invitation check too), the `caldav` commands before connecting (src/caldav/cli.rs `check`, src/cli.rs, `build_caldav_client` split from `caldav_config`).
- [x] src/caldav/backend.rs: the PROPPATCH clearing a property goes to `rfc4918::collection_path`.
- [x] Tests: tests/caldav_id.rs runs the binary over a CalDAV account whose password command leaves a file: refused calendar and item ids on every unified command naming one and on `caldav delete` exit 1 coded, io-webdav's reason in `sources`, a new id refused uncoded on `calendar create` and `caldav create`, the file never created; a clean id goes on to open the account. With io-webdav 0.9.1 and before the checks, the three refusal tests failed (the account opened, `connect 127.0.0.1:9` uncoded). src/caldav/backend.rs `a_caldav_listing_keeps_the_whole_item` pins W3.
- [x] CHANGELOG (Changed, Fixed).
- [x] Fold the delta into backends.md and commands.md, write the log entry, set `landed`.
