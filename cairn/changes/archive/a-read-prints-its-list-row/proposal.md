---
cairn: change
id: a-read-prints-its-list-row
status: landed
created: 2026-11-15
---

# A read prints its list row

> Lines K4 and K14 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), findings F9 and F14, decision D7, lines A12 and A13 of its schema review; cardamum's C12 and C13 are the twins.

## Why

On a scratch vdir, 2026-11-15:

- `item list --json` printed each item's `contents` as an array of integers, `item read --json` as a string, and both reads decoded a body that is not UTF-8 lossily, U+FFFD in place of its bytes: pimalaya-cli's output.md says iCalendar is a string when it is valid UTF-8, base64 otherwise, never decoded lossily.
- `item read --json` printed `{contents, etag}`, no `id`; `event read --json` printed `{contents, events}`, neither the item's `id` nor its `etag`: output.md (D7) has a read print the row its list prints, a file holding several events printing them as `events` beside the file's `id` and `etag`.
- An event carrying no `DESCRIPTION` or `LOCATION` printed `""`, which output.md keeps for the empty value.

## What

- An item prints as one row, in `item list`, `item read` and `event read` alike: `{id, calendarId, etag, contents, encoding}`. `contents` is the iCalendar text, with `encoding` `utf-8`, or the base64 of bytes that are not valid UTF-8 (standard alphabet, padded, base64-codec as Himalaya and cardamum encode), with `encoding` `base64`; both are `null` when the item carries no bytes (a pimdir body not local yet). Nothing is decoded lossily under `--json`; the text view of a read replaces what does not decode with U+FFFD.
- `item read` prints that row; `event read` prints it and `events`.
- An event's `description` and `location` are `null` when it carries none (the same for an empty property, which the projection reads as none).
- Breaking: MIGRATION.md gives the before and after.
- Spec: commands ("The event projection", "A delete can be gated on a version", "Text output is plain text"), plus an added requirement on bytes.
