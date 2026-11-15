---
cairn: change
id: capability-shared-api
status: landed
created: 2026-11-12
---

# The unified API serves the union of the backends

## Why

Change 9 of the-family-standards (owner, 2026-11-12): calendula adopts Himalaya's capability union (himalaya capability-shared-api, landed 2026-11-02), as cardamum did the same day. The unified API was a strict least common denominator in name, yet five backends already served it unevenly: `--send-updates` was gcal's alone, `--online-meeting` pimdir's alone, `--if-match` refused on a vdir delete and ignored on a vdir update, a description and a colour refused on a Graph calendar after its token command ran, and every calendar write refused on pimdir once the store was opened. Each refusal was ad hoc, uncoded, and reached at a different point of the command, so a script could not tell "not served here" from "it broke", and a person could not find out which backend serves what without running it.

## What

- src/capability.rs: `Capability`, fifteen names in pimdir STORAGE Annex B's grammar without the domain word, the collection read as the calendar and the calendar resource as calendula's `item`: `calendar.list`, `calendar.create`, `calendar.rename`, `calendar.delete`, `calendar.description`, `calendar.color`, `item.list`, `item.read`, `item.add`, `item.update`, `item.remove`, `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`. `Support` at `full`, `partial`, `server` or `none`, each but `full` with a detail. One table per backend (vdir, pimdir, caldav, gcal, msgraph), keyed by the name `--backend` takes and compiled into every build, seeded from the adapters' refusals and limits.
- Every unified command requires the capabilities its verb and its flags reach (`--if-match`, `--online-meeting`, `--send-updates`, a calendar's name, description or colour) through `CalendarClient::require` before anything is read, opened or connected. An unserved one fails with the new code `capability-missing`, naming the backend, the capability, why, and the first other backend of the account serving it with the `--backend` reaching it. The ad hoc refusals the tables cover go (`SendUpdatesArg::check_backend`, the online-meeting half of `EventFieldsArgs::check_backend`); the adapters keep theirs as a second guard.
- `account capability`: for each backend the account configures that `--backend` allows, in the order `auto` tries them, every capability with its level and detail, connecting nowhere: `{account, backends: [{backend, capabilities: [{name, level, detail}]}]}`, `Columns` in text.
- Each unified leaf's `--help` ends with a `Backends:` footer generated from the tables (`after_long_help`), per capability the backends grouped by level, then each detail with the backends it applies to. `build` reaches no backend and carries none; the top-level help keeps pimalaya-cli's `footer!`.
- A served verb prints what it printed: the tables refuse only what an adapter refused or dropped already.

## Decided

- **`auto` skips no backend**, as Himalaya's storage verbs: every unified verb exchanges calendar or item ids the others take, so all resolve to the first configured backend `--backend` allows, and a refusal names the backend of the account serving the capability instead. Himalaya's one exception, an account feature resolving on its own (`vacation`), has no counterpart in calendula.
- **Annex B names, with two readings.** `calendar.rename`, not `calendar.update`: Annex B has `collection.rename` and no `collection.update`, and `calendar update` reaches the rename, the description and the colour as three capabilities, as cardamum's `addressbook update` does. `--send-updates` is `scheduling.send-updates`, an option of Annex B's `calendar.scheduling`, not `scheduling` itself: a CalDAV server under RFC 6638 and Graph do notify the attendees, but take no per-write choice of whom, so a bare `scheduling` declared `none` on them would give the name a second meaning. `scheduling` alone, `reply` and `cancel` reach no unified verb (`pimdir reply` and `pimdir cancel` are pimdir's own) and are left out, the registry holding the unified verbs only.
- **vdir refuses `--if-match` on an update too**, where it ignored it: a guard dropped silently reads as one that held, the outcome cardamum's tables refuse. The vdir update adapter keeps ignoring an `if_match`, which only the command could pass.
- **pimdir's rows are static**, as cardamum's: its item writes `partial` (queued, refused when a source does not support them), `online-meeting` `partial`, the store's runtime check of what its sources declare refining it before anything is queued.
- **CalDAV calendar creation and deletion are `server`**: a server may refuse `MKCALENDAR` or the `DELETE` of a collection, which calendula reports as the server's error.
- **gcal and msgraph `online-meeting` stay `none`**: io-gcal 0.3.1 and io-msgraph 0.7.1 already project `X-PIMDIR-ONLINE-MEETING`, but the gcal backend sends no `conferenceDataVersion` and neither was ever served; serving them is its own change.
- **Alarm leads stay value checks.** `--alarm` is served everywhere, a lead gcal or msgraph cannot hold refused by `EventFieldsArgs::check_backend` as before.

## Compatibility

Every verb a backend serves keeps its output. The refusals the tables cover change their message and gain `code: "capability-missing"`, and the pimdir calendar writes fail with it before the store is opened, where they failed with the store's own error first: **breaking** for a script reading the message or `code: null`. `--if-match` on a vdir update is refused where it was ignored. The `json-schema` output gains `calendula-account-capability` and the `capability-missing` code in `calendula-error` and `calendula-account-check`.

## Scope / non-goals

The MCP server listing a tool only when the account's capabilities serve it is the-shared-repl-and-mcp's. A backend serving `online-meeting` beyond pimdir, and reading pimdir's rows from what the store's sources declare, as Himalaya does, are left for later changes.
