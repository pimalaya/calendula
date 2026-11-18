---
cairn: change
id: the-releases-land
status: landed
created: 2026-11-18
---

# The releases of 2026-11-18 land

## Why

pimalaya-config 0.6.0, pimalaya-cli 0.6.1, io-jmap 0.10.1 and io-pimdir 0.17.0 were released on 2026-11-18, carrying what calendula waited on: io-jmap's `Calendar/set` and `JmapCalendarEvent::to_ical`, which calendula stood in for; pimalaya-cli's `file::private_options`, the composer half of CAL-13; and io-pimdir's closing of the audit lines C1, C2 and C4 of pimdir cairn/changes/audit-2026-11-12 (draft-14).

- **C1**: calendula worded the refusal of a store of another format version itself, ordering versions by parsing `storage-draft-N` on both sides, so under `storage-v1` every draft store would read as a newer engine's, "upgrade calendula". io-pimdir's `version::PimdirFormatRefusal` is the one wording, ordering by STORAGE §4.2.
- **C2**: pimdir STORAGE §18 has a producer remove every `X-PIMDIR-` property from a resource the user did not write. calendula's raw `item create` queued an imported file whole and read its `X-PIMDIR-ONLINE-MEETING` as the user's ask, a departure its spec stated (owner decision of 2026-11-13). io-pimdir's default `enqueue_with_body` now strips, `enqueue_user_body` keeping a body the user wrote.
- **C4**: a queue row was followed and cancelled by a bare integer, without the store id and incarnation STORAGE §4.5 asks a producer to record, so a restored copy could hand the id to another row. io-pimdir answers a `PimdirActionHandle` from every enqueue and checks it.

## What

- Cargo: io-jmap 0.10.1, io-pimdir 0.17.0, pimalaya-cli 0.6.1, pimalaya-config 0.6.0, from crates.io, no `[patch]`.
- jmap: the calendar writes go through `JmapClientStd::calendar_set`, its `JmapCalendarSetItemError` coded as before; src/jmap/calendar_set.rs and `project::to_ical` removed for io-jmap's.
- C1: `store_error` words `store-version` through `PimdirFormatRefusal` naming calendula.
- C2: `ItemOrigin` (`User`, `Foreign`) on `CalendarClient::create_item` and `update_item`, read by pimdir alone: the `event` commands and a raw item composed with `-i` from the blank or the stored item are the user's, queued whole and gated for an online meeting; an iCalendar a raw `item` command is given is foreign, queued stripped and ungated.
- C4: the intent receipt carries `handle`; `pimdir queue show` and `pimdir queue cancel` take a handle or a bare id, a handle of another store or incarnation reading `reset`, refused `store-reset` by `cancel`.
- CAL-13: the composer's draft is created with `file::private_options` and `create_new`.

Left: `pimdir queue list` and `pimdir status` keep printing bare row ids, io-pimdir 0.17 exposing no store identity outside an enqueue's answer. ical-rs's `UNTIL` shift is unreleased (jmap-backend's line stays open).
