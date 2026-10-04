---
cairn: tasks
change: pimdir-occurrence-intents
---

# Tasks

- [x] Depend on io-pimdir `3a9ffb8`; build the intents with `PimdirInvitation`.
- [x] `-r/--recurrence-id` on `pimdir reply` and `pimdir cancel`; `recurrenceId` in their JSON.
- [x] Check the occurrence against a local body; refuse an unsupported one with its code.
- [x] Tests: payloads with `recurrence_id`, a missing or misspelled occurrence, a performer lacking the capability, an undeclared store.
- [x] Fold the delta into [backends](../../spec/backends.md); write the log entry.
