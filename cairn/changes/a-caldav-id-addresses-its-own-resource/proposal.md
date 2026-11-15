---
cairn: change
id: a-caldav-id-addresses-its-own-resource
status: landed
created: 2026-11-15
---

# A CalDAV id addresses its own resource

> calendula's part of line W2 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), after io-webdav 0.9.1 (W1, W3, W4); cardamum's twin is its change a-carddav-id-addresses-its-own-resource.

## Why

io-webdav 0.9.0 joined a collection or item id as given, and the url crate resolves dot segments when it builds the request URL: `calendula calendar delete ..` could send a DELETE to the home set's parent, `/` to the server root and `""` to the home set (`<home>//`). Found by reading (W1), never tried on a server. io-webdav 0.9.1 refuses such an id before any byte (`rfc4918::check_collection_id`, `check_resource_id`), failing `WebdavClientStdError::Io` of kind `InvalidInput`, which calendula would print uncoded, after resolving the account's secret and connecting. The hand-run PROPPATCH clearing a calendar property (src/caldav/client.rs `proppatch`) bypasses that check altogether; its path came from io-webdav's `rfc4791::calendar::join_path`.

io-webdav 0.9.1 also returns `calendar-data` whole through its typed listings (W3): a listed item whose final CRLF the server escapes (`&#13;`, iCloud and Fastmail) no longer loses it. calendula reads no `calendar-data` through a trimming `text()` of its own, so nothing else changes there.

## What

- io-webdav 0.9.1 (`cargo update -p io-webdav`, the lock diff that crate alone), the requirement raised to `0.9.1`.
- `caldav::client::check_calendar_id` and `check_item_id`: an id io-webdav refuses fails `calendar-not-found` (``Calendar `<id>` not found``) or `item-not-found` (``Item `<id>` not found``), io-webdav's reason in `sources`, as vdir codes a calendar id that is not one directory name. `check_new_calendar_id`: the id `calendar create` or `caldav create` gives a new calendar, its URL segment, fails uncoded (``Calendar id `<id>` names no CalDAV collection``), as vdir refuses a new id; a name alone takes a minted UUID and needs no check.
- The unified client runs them on a CalDAV account before it opens the backend (`CalendarClient::check_ids`, `check_new_id`), so nothing connects and no secret command runs; `caldav create` and `caldav delete` run them after resolving the account and before connecting (`CaldavCommand::check`, the account's block read by `caldav_config`, the client built by `build_caldav_client`).
- The hand-run PROPPATCH goes to `rfc4918::collection_path`, the same path in 0.9.1, named as the family names it; the `proppatch` function itself is untouched.
- Code chosen: the not-found codes, not `usage-invalid`, as W2's line rules (an id may come from `calendar.default`), as cardamum and carillon code the same ids.
- Spec: backends ("A CalDAV id addresses its own resource", "A CalDAV item reads whole", added; "CalDAV backend", modified), commands (the `item-not-found` and `calendar-not-found` rows of "A JSON error carries a stable code", modified).
