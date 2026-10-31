---
cairn: log
change: gcal-send-updates
landed: 2026-10-29
---

# A gcal write chooses who is mailed

`SendUpdatesArg` in src/shared/arg.rs gives `event create`, `event update` and `event delete` a `--send-updates all|external-only|none`, refused by name off gcal before the backend is reached. The choice travels as one more `Option<SendUpdates>` on the shared client's `create_item`, `update_item` and `delete_item`, beside `if_match`; the `todo`, `journal` and `item` commands pass `None`, and every backend but gcal ignores it. On gcal, `update_item` and `delete_item` send it as `sendUpdates` when given, and `create_path` lets it replace the `all` of an inserted invitation, passes `none` on an import and refuses `all` or `external-only` there, naming the organizer, when the event has attendees.

Verification: `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` with all features, and builds with each backend alone; checked by hand against a Google calendar, a second mailbox receiving the invitation of a control event created without the flag and nothing from a create, an update and two deletes given `--send-updates none`.

Capabilities moved: **commands** (a gcal write chooses who is mailed, added); **backends** (Google Calendar backend, modified).
