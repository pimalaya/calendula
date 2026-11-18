---
cairn: log
change: event-alarm-flag
landed: 2026-11-18
---

# An alarm from the command line

Landed [event-alarm-flag](../changes/archive/event-alarm-flag/proposal.md): `--alarm <LEAD>` and `--no-alarm` on `event build`, `event add` and `event update`, and `alarms` in the event JSON. The code landed in 4319651 (2026-11-12); this entry closes the change after its live check.

Live check of 2026-11-18, the owner's grant of the day, on the test accounts only: tests/google.rs `gcal_event_alarm_round_trip` and tests/microsoft.rs `graph_event_alarm_round_trip` each create a calendar named for the run (`calendula-live-alarm-<millis>`, `ScratchCalendar` in tests/common.rs, deleted when the test ends, failed or not), add an event with `--alarm 15m`, update it with `--alarm 1h` and then `--no-alarm`, reading the alarms back after each step. Both answer `-PT15M`, then `-PT60M`, then no alarm: Google and Graph hold a lead in minutes, so `1h` reads back as `-PT60M`, which the spec now says. Google needs the full `calendar` scope to create a calendar, so its test runs on the suite's CalDAV token. Two runs each passed; an earlier Graph run failed once on a calendar name a run had just deleted, hence the name per run.

Spec folded into commands.md: "The composer is the authoring surface", "The field flags are a convenience, not the surface" and "The event projection" modified; "One display alarm can be set from a flag", "Every alarm can be removed from a flag" and "A lead a backend cannot hold is refused before the write" added, written with `event add`, the verb `event create` became. projection.md already held the mappings.

Left provisional, as the change's notes say: msgraph refuses a lead naming days or weeks until io-msgraph reads it, `--no-alarm` on gcal falls back to the calendar's defaults, and `alarms` carries no `related`.

Capabilities moved: commands.
