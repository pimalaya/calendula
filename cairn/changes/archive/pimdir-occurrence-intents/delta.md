---
cairn: change
change: pimdir-occurrence-intents
---

# Delta

## MODIFIED Requirements

### Requirement: pimdir queues scheduling intents
`pimdir reply <EVENT-ID> accept|tentative|decline` SHALL append one `calendar-reply` action `{ "v": 1, "source"?, "seq", "partstat": "ACCEPTED"|"TENTATIVE"|"DECLINED", "comment"?, "recurrence_id"? }`, and `pimdir cancel <EVENT-ID>` one `calendar-cancel` action `{ "v": 1, "source"?, "seq", "comment"?, "recurrence_id"? }` (pimdir STORAGE Annex B.2), built by io-pimdir's `PimdirInvitation`, anchored on the item's calendar, through the producer and its §15.6 gate, and SHALL touch no item.

`source` SHALL name the performer: `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. A store whose sources declare nothing SHALL get no `source` unless one is given.

`-r/--recurrence-id` SHALL limit either to one occurrence, carried as `recurrence_id`, spelled as the `recurrenceId` an expanded listing shows (the series' `DTSTART` form). When the body is local, a value naming no occurrence of the series, or naming one in another spelling, SHALL be refused. A performer not declaring `calendar.reply.occurrence` or `calendar.cancel.occurrence`, and a store declaring nothing, SHALL be refused with the code `occurrence-unsupported` and an error starting `Occurrence not supported:`, naming the capability and the source; nothing is queued.

A reply to an event naming no `ORGANIZER`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, the latter pointing at `event delete`.
