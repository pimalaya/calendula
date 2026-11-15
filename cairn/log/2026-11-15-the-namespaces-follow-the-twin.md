---
cairn: log
change: the-namespaces-follow-the-twin
landed: 2026-11-15
---

# The namespaces follow the twin

calendula's protocol namespaces take cardamum's shape: `caldav` is the WebDAV and CalDAV methods, `msgraph` arrives nested by Graph resource, `vdir` gains an `item` subtree, and the gcal event methods move under `gcal events`. Every former name stays, hidden. Line K6 of the CLI alignment round, finding F23. Capabilities **commands** and **backends** moved.

- **caldav** (src/caldav/): `discover`, `propfind` (`list`, `ls` hidden), `proppatch`, `mkcalendar` (`create` hidden), `report query|multiget|sync|raw`, `get`, `put`, `delete` of an item or a calendar. Every id is checked before connecting, as before.
- **msgraph** (src/msgraph/): `calendars list|get|create|rename|delete`, `events list|get|create|update|delete|delta`, `profile get`, `request`; the client connects as the backend does (`connect` shared).
- **vdir** (src/vdir/item/): `item list|get|create|update|delete|copy|move`.
- **gcal** (src/gcal/events.rs): `events instances|move|quick-add`; `gcal instances`, `instance`, `move` and `quick-add` hidden commands running the same.
- **Departures from cardamum**, each stated in the proposal: `report query` takes RFC 4791's filter (`--component`, a window, a substring `--match PROP VALUE`, no `--test` nor `--limit`); the item reads print `encoding`; `mkcalendar` and `proppatch` take `--tz`, `mkcalendar` `--component`; the `create` alias is hidden; `propfind` and `discover` keep calendula's text; `msgraph events` take `-k/--calendar`, and `events delta` a window.
- **Evidence**: `namespace_tests` (src/cli.rs) parses every new path and every former name; `a_home_propfind_keeps_the_former_listing_shape` prints the `caldav list` JSON key for key; the report, Graph and vdir tests run the commands over scripted servers and a temp home.

CHANGELOG (Added, Changed), MIGRATION.md (Commands and flags). Spec: commands ("Protocol-specific families", "gcal family", "Text output is plain text", and the added "A former command name stays hidden", "caldav speaks the protocol", "msgraph family", "vdir items"), backends ("A CalDAV id addresses its own resource").
