---
cairn: log
change: a-calendar-update-clears-and-colours-first
landed: 2026-11-15
---

# A calendar update clears, and writes its colour first

`calendar update -d ""` clears the description on CalDAV (`DAV:remove`) and Google (an empty description), where it exited 0 and left it; a CalDAV colour goes first, alone, so a server ignoring it fails the update before anything else is written; a colour CalDAV drops on create is warned of. Line K21 and I-F3 of K24 of the CLI alignment round, from the live testing campaign (I-F2, I-F3, G-F6). Capabilities **backends** and **commands** moved.

- **src/caldav/client.rs**: `proppatch` pumps io-webdav's `WebdavProppatch` (set and remove) over the client's stream, `accepted` reading the answer as io-webdav reads its own; io-webdav 0.9's calendar update sets alone.
- **src/caldav/backend.rs**: `update_calendar` writes only what the patch names, the colour first; `patch` goes through io-webdav's update for a set and `proppatch` for a clear; `create_calendar` reads a given colour back.
- **src/gcal/backend.rs**: a cleared description is sent `""`; an empty one reads as none.
- **Evidence**: on the code before, `a_cleared_description_is_removed` found no `<D:remove>`, `an_ignored_colour_writes_nothing_else` one PROPPATCH holding all three properties, `a_colour_dropped_on_create_is_read_back` no PROPFIND, `a_cleared_description_is_sent_empty` no `description` field.

CHANGELOG (Fixed, Changed). Spec: backends ("CalDAV backend", "Google Calendar backend"), commands ("A calendar update clears what it empties", added).
