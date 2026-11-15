---
cairn: change
id: the-queue-is-read-and-retracted
status: landed
created: 2026-11-15
---

# The queue is read and retracted

> Line K7 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F24; cardamum's C5 is the twin.

## Why

Himalaya reads and retracts its writes through `pimdir queue list`, `show` and `cancel`; calendula had no way to see a queued creation, a queued reply or cancellation, nor to take one back but the store's own tooling.

The line also asks `pimdir status` and `pimdir cancel` to become hidden aliases. Neither is a queue command, which the audit read them as: `pimdir status` reports what the store holds (accounts, how much of each calendar is downloaded), and `pimdir cancel` queues the `calendar-cancel` intent that notifies an event's attendees. Aliasing them would drop the report and turn an event cancellation into a row cancellation, so both stay as they are.

## What

- `pimdir queue list [-k CALENDAR]` (`ls`), every calendar of the account when `-k` is omitted, as cardamum's: `{items, parked}`, each creation and each reply or cancellation waiting, the rows no listing shows, `{row, queuedAt, producer, collection, kind, linkId, itemId, summary}`, then each parked row of any kind `{row, queuedAt, producer, collection, kind, attempts, unconfirmed, error}`, as Himalaya's and cardamum's keys and kinds (kebab-case); the text through pimalaya-cli's `Listing`, `Cancel:` naming the command.
- `pimdir queue show <ROW>`: Himalaya's `{row, state, collection, kind, attempts, error, appliedAt, seq}`, `unknown` for a row of another account.
- `pimdir queue cancel <ROW> [-y]`: asks unless `-y`, drops the row as the store's owner for that call alone (`store-owned` while a sync runs), prints `{row, kind, unconfirmed}`.
- The backend opened no `PimdirStore`; the cancel now does, for one row, as Himalaya's does.
- `pimdir status` keys a parked row's kind `kind`, where it said `action`, its text column `KIND`, as cardamum's status and every queue row do. The parked report is new since 0.2.0, so no MIGRATION entry.
- Spec: backends ("pimdir reads and retracts its queue", added; the reader and producer roles requirement, modified), commands.
