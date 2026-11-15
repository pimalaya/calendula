---
cairn: log
change: the-queue-is-read-and-retracted
landed: 2026-11-15
---

# The queue is read and retracted

`pimdir queue list`, `pimdir queue show` and `pimdir queue cancel` read and retract the store's queue as Himalaya's do, line K7 of the CLI alignment round. `pimdir status` and `pimdir cancel` stay visible commands, not aliases: one is the store's report, the other the `calendar-cancel` intent. Capabilities **backends** and **commands** moved.

- **src/pimdir/queue.rs, src/pimdir/queue/**: the three commands; `queue list` reads every calendar of the account unless `-k` names one, the creations and the intents waiting (the rows no listing shows) then the parked rows, each keyed `row` with its `collection` and its kind as the queue spells it, as cardamum's C5 lists its address books; `queue cancel` takes the owner role for the one row it drops, as Himalaya's `cancel_queued`.
- **src/pimdir/backend.rs**: the account name kept for the listing's `Account:`; `calendar_collections` and `item` crate-visible, a row of another account's calendar reading `unknown`, an intent's summary read from its stored event.
- **src/pimdir/cli.rs, src/pimdir/status.rs, src/json_schema.rs, README.md**: the subcommand, the status help pointing at `queue cancel`, three schemas.
- **src/pimdir/status.rs**: a parked row keys its kind `kind`, where it said `action`, its text column `KIND`, as cardamum's; new since 0.2.0, no MIGRATION entry.
- **Evidence**: src/pimdir/queue.rs tests on a scratch store; none could fail before, the commands being new. Not run against a store a sync holds: the owner lock is per process, so the `store-owned` refusal is Himalaya's path, untested here.

CHANGELOG (Added). Spec: backends ("pimdir reads and retracts its queue", added; the roles requirement, modified), commands.
