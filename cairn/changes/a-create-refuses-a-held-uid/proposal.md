---
cairn: change
id: a-create-refuses-a-held-uid
status: landed
created: 2026-11-15
---

# A create refuses a held UID

> Line K19 (and I-F5 of K24) of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), from the live testing campaign of 2026-11-15 (its reports/calendula/gcal-google.md G-F1 and caldav-icloud.md I-F5).

## Why

On Google, `calendula -a google --json event create -k <cal> google-e1-dup.ics`, the UID of an event the calendar held and another summary, exited 0 and printed the id of that event, whose summary the listing then showed replaced: `events.import` is an upsert on the `iCalUID`, so a plain create overwrote an event nobody named. On iCloud the second create of one event failed `precondition-failed` (an HTTP 412 to the create's `If-None-Match: *`), the code commands.md keeps for a stale `--if-match`, where `item-duplicate` is "a create names a `UID` its calendar holds".

Checked on the other backends: pimdir refuses the create `item-duplicate` already; Graph accepts a second event of one UID, which the code table allows (no loss, two events); vdir mints a fresh file name, a second file of one UID.

## What

- gcal `create_item`: an event carrying a UID is looked up first, `events.list?iCalUID=<uid>` walking the pages until one holds a match (a deleted event is not listed); a match refuses the create `item-duplicate`, ``Calendar `<cal>` already holds `<uid>` as event `<id>`; update that event instead``, the id the series master's, and nothing is inserted nor imported.
- CalDAV `create_item`: a 412 is `item-duplicate`, ``Calendar `<cal>` already holds `<name>`; update that item instead``, the server's answer in `sources`: the resource name is derived from the UID, so the name taken is that UID's.
- Graph and vdir unchanged, stated in the code table.
- Spec: backends ("A server's refusal carries its code", "Google Calendar backend"), commands (code table).
