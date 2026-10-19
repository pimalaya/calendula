---
cairn: change
id: gcal-create-invites
status: active
created: 2026-10-19
---

# A gcal event created with attendees invites them

## Why

`calendula event create --attendee …` on a gcal account stores the attendees but mails none of them (#20). Every event minted from flags carries a `UID`, so `create_item` sends it through `events.import`, which Google documents as adding "a private copy of an existing event" and which takes no `sendUpdates`. The guest only hears of the event at its first update, which Google does mail.

Import was chosen so the `UID` survives. `events.insert` keeps it too: inserting an event whose body carries `"iCalUID": "prueba-insert-1792440770@calendula"` returned that same `iCalUID` (tested against Google Calendar on 2026-10-19, with `sendUpdates=none` and no attendees).

## What

`create_item` uses `events.insert` with `sendUpdates=all` when the event names at least one `ATTENDEE` and no `ORGANIZER`: an event the account is creating and inviting people to, which Google then organizes as the account. The `iCalUID` is sent in the body, so the `UID` survives as it does through import.

Every other event keeps going through `events.import` as today (with a `UID`) or `events.insert` without `sendUpdates` (without one). An event that names an `ORGANIZER` is a copy of someone's invitation, or a migration, and inserting it would make the account its organizer and mail its guests on their behalf.

Out of scope, each to its own issue: a flag to choose whether guests are mailed on create, update and delete, and adding or removing one attendee without resetting the others' `PARTSTAT`.

## Open question

An event whose `ORGANIZER` is the account itself (built by hand, or exported and created again) still goes through import and mails nobody. Comparing the organizer with the calendar's owner would cover it, but `primary` is not an address; it is left out unless you prefer to read the owner from the calendar list entry.
