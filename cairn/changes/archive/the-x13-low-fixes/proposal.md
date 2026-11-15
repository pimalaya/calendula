---
cairn: change
id: the-x13-low-fixes
status: landed
created: 2026-11-15
---

# The X13 low fixes

> Task X13 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15/tasks.md): X9c, K28, the raw gcal listings, the queue row's `attempts`, and the owner's rulings D26, D27 and D28. pimalaya-cli's cairn/spec is the contract.

## Why

- X9c: a duplicate collection was worded per path, ``Calendar `x` already exists`` on `calendar create`, ``Collection `x` already exists`` on `vdir create`, Graph's own words on Graph, where Himalaya says ``Cannot create mailbox `x`: it exists already``.
- K28: a vdir `item update <missing-id> <ICAL>` (and `event update`) created the item, an upsert, where every other backend fails `item-not-found`; and a read's not-found failure listed ``Read item `x` from calendar `y` `` as a first source above the backend's answer, a sentence the not-found one already says (X3 L19).
- Raw gcal listings: `gcal calendars`, `gcal acl list`, `gcal events instances` and `gcal settings` read every page and printed rows calendula shaped, where pimalaya-cli output.md (*Protocol commands are raw-faithful*) has a protocol listing print the provider's items as sent, under its collection key, as Himalaya's Gmail listings and cardamum's gpeople listings do.
- `attempts`: the queue row printed a signed count; cardamum's is `u64`.
- D26: calendula had `msgraph request` but no `gcal request`, where cardamum has `gpeople request`.
- D27: `Next page:` prints after the listing everywhere.
- D28: `calendar create [ID] <NAME>` takes an optional positional before a required one, which cardamum's `addressbook create` does not; the owner rules `calendar create <NAME> [--id <ID>]`.

## What

- X9c: one sentence, ``Cannot create calendar `<calendar>`: it exists already``, on `calendar create` over vdir, CalDAV (the id) and Graph (the name), and on `vdir create`; the code stays `calendar-duplicate`, a server's answer in `sources` (src/error.rs `calendar_duplicate`, `existing_calendar`).
- K28: the vdir backend locates the item before it overwrites it, an id naming none failing `item-not-found` with the family sentence; the CalDAV, Google, Graph and vdir reads add ``Read item ...`` only to a failure that is not `item-not-found` (src/error.rs `read_failed`), so a miss reads the sentence over the backend's own answer alone.
- Raw gcal listings, before and after (`--json`):
  - `gcal calendars`: `{calendars: [{id, summary, description, timeZone, accessRole, primary, selected, color, defaultReminders: ["10m"]}]}`, then `{calendars: [<calendarListEntry as sent>]}` (`summary`, `summaryOverride`, `backgroundColor`, `defaultReminders: [{method, minutes}]`, `kind`, `etag` and the rest).
  - `gcal acl list`: `{rules: [{id, scope, value, role}]}`, then `{rules: [<aclRule as sent>]}` (`scope: {type, value}`).
  - `gcal events instances`: `{instances: [{id, summary, start, end, status, originalStart}]}` (strings), then `{instances: [<event as sent>]}` (`start`, `end`, `originalStartTime` objects).
  - `gcal settings`: `{settings: [{id, value}]}`, then `{settings: [<setting as sent>]}` (`kind`, `etag`, `id`, `value`).
  - Each still reads every page (no `nextPage`, as before), the items read as `serde_json::Value` through a raw `GcalSend` (src/gcal/client.rs `raw_items`), as Himalaya's `raw_list`. The text is unchanged.
- `attempts`: `u64` (schema `uint64`, minimum 0), the store's count clamped at 0.
- D26: `gcal request <METHOD> <PATH> [JSON]`, cardamum's `gpeople request` and `msgraph request` copied: the path joined onto `v3/` or a full URL, `{response}`.
- D27: calendula builds no `Next page:` line outside pimalaya-cli's `Listing` but in the Graph listings, which print it after the records already; `Listing` itself moves in pimalaya-cli 0.5.3, bumped in a later pass.
- D28: `calendar create <NAME> [--id <ID>]`. The old two-positional form cannot stay as a hidden alias and is refused (exit 2).

Breaking against 0.2: the gcal listings' JSON, D28, and the vdir update no longer creating. X9c and the read's sources reword [Unreleased] lines (the code and the sentence are new since 0.2); `attempts` and `gcal request` are new.
