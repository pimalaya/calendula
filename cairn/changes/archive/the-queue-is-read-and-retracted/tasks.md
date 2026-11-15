---
cairn: tasks
change: the-queue-is-read-and-retracted
---

# Tasks

- [x] src/pimdir/queue.rs: `PimdirQueueCommand`, and `PimdirBackend::{queued_rows, parked_rows, queue_row, cancel_queued}`; src/pimdir/queue/list.rs, show.rs and cancel.rs, show and cancel adapted from Himalaya's.
- [x] src/pimdir/backend.rs: the backend keeps its account name; `calendar_collections` and `item` crate-visible.
- [x] src/pimdir/cli.rs, src/json_schema.rs: the `queue` subcommand, the schemas `calendula-pimdir-queue-list`, `-show`, `-cancel`.
- [x] Tests: src/pimdir/queue.rs on a scratch store (a queued add lists in its calendar and in every one with its link id and summary, shows `pending`, cancels, then reads `unknown`; an unknown calendar is `calendar-not-found`), the list text and keys, the show and cancel projections.
- [x] src/pimdir/status.rs: the parked row's `action` key is `kind`, the column `KIND`; the status test reads the JSON key.
- [x] CHANGELOG; fold the delta into backends and commands; log.
