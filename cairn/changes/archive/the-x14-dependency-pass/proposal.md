---
cairn: change
id: the-x14-dependency-pass
status: landed
created: 2026-11-15
---

# The X14 dependency pass

> Task X14 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15/tasks.md), with N31, K26, K22, the D28 twin and the request origins. pimalaya-cli's cairn/spec is the contract.

## Why

- X14: pimalaya-cli 0.6.0 prints `Next page:` after the listing (D27), io-msgraph 0.9.2 and io-gcal 0.5.1 bring the series writers, io-pimdir 0.14.0 follows pimdir draft-13, whose store refuses draft-11 code.
- N31: draft-13 records a server's refusal of a pushed change (pimdir STORAGE §15.4): a producer shows such a row applied and refused by that source, with the reason, and `list_refusals` reads them for a view beside the queue.
- K26: the Graph and Google backends wrote a series' master alone, so its EXDATEs (on Graph) and its overrides vanished with no word.
- K22: io-gcal 0.5.0 projected a zoned boundary in the calendar's zone, so a weekly 09:00 Paris event in a UTC calendar drifted to 10:00 after the daylight-saving change.
- D28 twin: cardamum refuses `--id` on a backend minting the id (`addressbook.create.id`); calendula ignored it on Google and Graph.
- Request origins: `gcal request` and `msgraph request` sent the bearer token to whatever full URL they were given.

## What

- X14: pimalaya-cli 0.6.0, io-msgraph 0.9.2, io-gcal 0.5.1, io-pimdir 0.14.0 (Cargo.toml requirements raised, Cargo.lock moved for those crates alone, no `[patch]`). The listing snapshots follow: `Next page:` after the rows, the header values two spaces past the longest label.
- N31: `pimdir queue show` prints `refusals: [{collection, source, reason, refusedAt}]` and, in text, `Refused by <source> in <calendar> at <when>: <reason>`; an applied row says `on event <seq>`, the event it acted on, where it said `creating event`. `pimdir status` adds `refused: [{calendarId, itemId, source, reason, refusedAt}]` and an `EVENT CALENDAR SOURCE REASON` table. Shapes as cardamum's, the domain words calendula's.
- K26: Graph writes through `MsgraphEventSeries` (master, then each instance change on the instance `event_instances` lists: delete or patch), Google through `GcalEventSeries` (master with its EXDATEs, then each override merged onto the listed instance it `replaces` and updated with its etag). A create whose occurrence fails deletes the series again; `left_out` is warned of. The `item.add` and `item.update` rows of both say so.
- K22: io-gcal 0.5.1; a regression test reads a weekly Paris series from a UTC calendar across the March change.
- D28 twin: `calendar.create.id`, full on vdir and CalDAV, none on Google, Graph and pimdir; `calendar create --id` requires it.
- Request origins: a full URL off the API's origin is refused before anything is sent (src/shared/raw_json.rs `request_url`).

Nothing here breaks a 0.2 JSON shape: the draft-13 store and `--id` reword [Unreleased] lines, the text layout is a Changed line, the rest is new or a fix.
