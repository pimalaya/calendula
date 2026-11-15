---
cairn: change
id: pimdir-occurrence-intents
status: landed
created: 2026-10-04
---

# `pimdir reply` and `pimdir cancel` name one occurrence

## Why

`calendar-reply` and `calendar-cancel` named an item by `seq` only, so answering or cancelling one occurrence of a series was impossible. pimdir draft-04 (io-pimdir `3a9ffb8`) adds an optional `recurrence_id`, gated on `calendar.reply.occurrence` and `calendar.cancel.occurrence`.

## What

- `-r/--recurrence-id` on both commands, carried as `recurrence_id`; the payload built by io-pimdir's `PimdirInvitation`.
- A local body is checked for that occurrence, in its own spelling.
- A performer unable to act on one occurrence is refused with `occurrence-unsupported`.
- The JSON output adds `recurrenceId`.
- io-pimdir is taken by git rev `3a9ffb8`.
