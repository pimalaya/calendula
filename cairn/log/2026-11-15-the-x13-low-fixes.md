---
cairn: log
change: the-x13-low-fixes
landed: 2026-11-15
---

# The X13 low fixes

Task X13 of the CLI alignment round: X9c, K28, the raw gcal listings, the queue row's `attempts`, and the owner's rulings D26, D27 and D28. Capabilities moved: commands, backends.

- **X9c**: a calendar created twice fails ``Cannot create calendar `<calendar>`: it exists already`` on vdir, CalDAV, Graph and `vdir create`, Himalaya's form, the code `calendar-duplicate` and the server's answer in `sources` unchanged (src/error.rs).
- **K28**: a vdir `item update` or `event update` given an iCalendar for a missing id fails `item-not-found` with the family sentence, where it created the item (src/vdir/backend.rs); a read's miss lists the backend's answer alone under the sentence, the ``Read item ...`` line kept for the other failures (src/error.rs `read_failed`).
- **Raw gcal listings**: `gcal calendars`, `gcal acl list`, `gcal events instances` and `gcal settings` print every item of every page as Google sent it, under `calendars`, `rules`, `instances` and `settings`, read as JSON through `GcalClient::raw_items`; the text is unchanged.
- **`attempts`**: the queue row's count is `u64` (schema `uint64`, minimum 0), as cardamum's.
- **D26**: `gcal request <METHOD> <PATH> [JSON]` prints `{response}`, as `msgraph request` and cardamum's `gpeople request`.
- **D27**: no change; calendula's own `Next page:` lines (the Graph listings) print after the records already, and pimalaya-cli's `Listing` moves in its 0.5.3.
- **D28**: `calendar create <NAME> [--id <ID>]`; the former `calendar create <ID> <NAME>` exits 2.
- **Breaking against 0.2**: the gcal listings' JSON, D28 and the vdir update, each with a CHANGELOG line and a MIGRATION.md entry; X9c and the read's sources reword [Unreleased] lines.
