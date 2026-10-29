---
cairn: change
id: event-alarm-flag
status: active
created: 2026-10-27
---

# An alarm from the command line

## Why

The field flags leave alarms to the composer (spec/commands.md, "The field flags are a convenience, not the surface"): writing one VALARM by hand for "remind me 15 minutes before" is the most common edit a person makes to an event, and a script creating events has no composer. With the alarm daemon (decided 2026-10-27), an alarm set from the command line is one a desktop will fire.

## What

`event build`, `event create` and `event update` take:

- `--alarm <LEAD>`: one display alarm, `LEAD` before the start, as an RFC 5545 duration or a short form (`15m`, `1h`, `2d`). It writes a VALARM with `ACTION:DISPLAY`, `TRIGGER:-<LEAD>` and the event's summary as `DESCRIPTION` (required for DISPLAY, RFC 5545 3.6.6).
- `--no-alarm`: removes every VALARM.

As every field flag, `--alarm` replaces what it names: every VALARM of the VEVENT. Several alarms, an alarm after the end, an absolute one or an EMAIL one stay the composer's.

One alarm before the start is what every backend holds: CalDAV, vdir and pimdir store the VALARM; Google takes it as a reminder override within its four weeks in whole minutes (spec/projection.md already maps it); Graph holds one `reminderMinutesBeforeStart`. A lead Google or Graph cannot hold (seconds, beyond four weeks on Google) is refused before the write, never rounded.

`event list` and `event read` gain an `alarms` field in `--json` (each alarm's trigger as written) and `event read` shows them; the table stays as it is.

The spec requirement excluding alarms from the field flags is modified: the composer stays the complete surface, and `--alarm` covers the one alarm every backend holds.

## Open questions

- **Graph's mapping.** spec/projection.md covers Google's reminders; whether the msgraph backend maps a VALARM onto `reminderMinutesBeforeStart` and `isReminderOn` today is unchecked (no `reminder` in src/msgraph). If not, this change adds it.
