---
cairn: log
change: zero-length-whole-day-ends-on-its-start
landed: 2026-10-19
---

# A zero-length whole day ends on its start

`table_times` in src/shared/event/list.rs clamps a whole-day event's last day to its first, so an event whose `DTEND` equals its `DTSTART` (an `endsAt` equal to its `startsAt`) no longer shows its end the day before its start. The JSON payload is untouched. Follow-up to `event-list-local-times`, found reviewing pimalaya/calendula#19.

Verification: `cargo test` (the new test fails without the clamp), `cargo clippy --all-targets --all-features` and `cargo fmt --check`.

Capabilities moved: **commands** (the event table never ends a whole day before its start, modified).
