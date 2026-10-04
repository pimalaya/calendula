---
cairn: log
change: pimdir-occurrence-intents
landed: 2026-10-04
---

# `pimdir reply` and `pimdir cancel` name one occurrence

Both take `-r/--recurrence-id`, carried as `recurrence_id` in the intent io-pimdir's `PimdirInvitation` builds (pimdir draft-04, io-pimdir `3a9ffb8` by git rev). A local body is checked for the occurrence in its own spelling; a performer not declaring `calendar.reply.occurrence` or `calendar.cancel.occurrence`, or a store declaring nothing, is refused with `occurrence-unsupported`. Their JSON adds `recurrenceId`.

Capabilities moved: **backends** (pimdir queues scheduling intents, modified).
