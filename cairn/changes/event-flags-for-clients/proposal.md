---
cairn: change
id: event-flags-for-clients
status: landed
created: 2026-10-06
---

# Field flags a client can edit events with

## Why

MOA moves its event writes onto `event create` and `event update` with field flags (`event-field-flags`). Three gaps keep it writing iCalendar of its own:

1. An empty text flag writes an empty property (`--location ""` gives `LOCATION:`). A client emptying a field means the field is gone.
2. `event update` leaves `DTSTAMP` as it was. RFC 5545 3.8.7.2: in a stored calendar it is the date the information was last revised; scheduling clients compare it.
3. No flag asks the sync engine for an online meeting. pimdir STORAGE Annex B.1 names `X-PIMDIR-ONLINE-MEETING:TRUE` for that, which neverest turns into a Meet or a Teams link where the source declares `calendar.online-meeting`.

## What

1. An empty value given to a text flag (`--summary`, `--description`, `--location`, `--url`) removes the property; a repeated flag whose values are all empty (`--categories ""`) removes it too. `--attendee ""` and `--organizer ""` remove every `ATTENDEE`, or the `ORGANIZER`. Times, `--status`, `--transparency` and `--sequence` keep refusing an empty value.
2. `event update` sets `DTSTAMP` to now (UTC) whenever it writes, field flags or not, and `LAST-MODIFIED` to the same instant when the event carries one.
3. `--online-meeting` writes `X-PIMDIR-ONLINE-MEETING:TRUE` on the VEVENT. On the pimdir backend the write is refused before it is queued when no source of the store declares `calendar.online-meeting` (as unsupported writes already are, STORAGE 15.6). Other backends refuse the flag by name.
