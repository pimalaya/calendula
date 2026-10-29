---
cairn: tasks
change: event-alarm-flag
---

- [x] Check the msgraph VALARM mapping (io-msgraph's `ical` feature, calendula src/msgraph); add it if missing
- [x] delta.md: MODIFIED "The field flags are a convenience, not the surface" and "The composer is the authoring surface"; ADDED the `--alarm` and `--no-alarm` requirements with scenarios (a create, an update replacing two alarms, a lead Google refuses)
- [x] `--alarm` and `--no-alarm` in src/shared/event/fields.rs, written through the CST, everything else byte for byte
- [x] Lead parsing: RFC 5545 duration through ical-rs's validator, plus the short forms
- [x] Refusal of a lead gcal or msgraph cannot hold, before the write
- [x] `alarms` in the event JSON output, registered in src/json_schema.rs (no registry change: `Event` already rides `EventListOutput`, `EventReadOutput` and `EventBuildOutput`, so `EventAlarm` lands in their schemas)
- [x] Tests per requirement; `--help` of each verb in a debug build (clap collisions)
- [ ] Live check on the Google and Microsoft test accounts only
- [x] `cargo fmt`, `cargo test`, `cargo clippy`; README and CHANGELOG (Added) (README lists no field flag, so it is unchanged)
- [ ] Fold into spec/commands.md and spec/projection.md, log entry

## Notes

- **What exists (checked 2026-10-27).** calendula's src/msgraph carries no alarm code: the projection lives in io-msgraph's `ical` feature (rev `2ae2637`, src/v1/rest/users/events/ical.rs). It already maps a VALARM onto `isReminderOn` and `reminderMinutesBeforeStart`: the first DISPLAY or AUDIO alarm whose trigger is a negative duration in whole minutes becomes the reminder, every other alarm goes to the stash, and a document with no such alarm writes `isReminderOn: false`. On read it mints `ACTION:DISPLAY`, `TRIGGER:-PT<n>M`, `DESCRIPTION:Reminder`. io-gcal (rev `9053761`) maps DISPLAY and EMAIL alarms with a lead in whole minutes up to 40320 onto reminder overrides, reads days (`-P1D` is 1440), refuses weeks and seconds into the stash, and falls back to the calendar's default reminders when no VALARM projects. No mapping had to be added.
- **Provisional: msgraph refuses a lead naming days or weeks.** io-msgraph's `reminder_minutes` reads the lead through `jiff::Span::total(Unit::Minute)`, which errors on a span carrying days or weeks without a relative date, so `-P2D` is stashed rather than set as the reminder. calendula refuses such a lead on msgraph naming the hours or minutes spelling (`give 48h instead`). The fix belongs in io-msgraph (`SpanTotal::days_are_24_hours()`, or ical-rs's `IcalDuration::seconds`); once released, the refusal can go.
- **Provisional: `--no-alarm` on gcal does not silence the event.** Per spec/projection.md a document with no VALARM inherits the calendar's default reminders; `--no-alarm` follows that rule and the help says so. Silencing a Google event would need an empty override list with the defaults off, which no iCalendar shape expresses today.
- **Provisional: the JSON shape.** `alarms` is `[{action, trigger}]`, both `null` when absent: `action` uppercased (added beside the trigger the proposal names, since a trigger alone does not say DISPLAY from EMAIL), `trigger` the value as written (a duration, or a UTC time for an absolute one). A `RELATED=END` parameter is not carried, so `P1D` related to the end reads like one related to the start; adding `related` is open.
- **Provisional: the short forms.** Lowercase `m`, `h` and `d` after a whole number only (`15m`, `1h`, `2d`), as the proposal lists; `15M`, `1w`, `1.5h` are refused. A negative lead (`-PT15M`) is refused, a lead being a length before the start. `--alarm` takes one value, not a repeated flag.
- **Provisional: the alarm description.** `DESCRIPTION` holds the summary the event ends up with after the flags (a `--summary` given on the same line wins), and `Reminder` when it has none, the word io-gcal and io-msgraph mint.
- **Provisional: placement.** The new VALARM lands where the first dropped VALARM stood, else after the VEVENT's last item, as a property flag lands where its first instance stood.
- **Open: pimdir.** A pimdir store syncing to a Google or Microsoft source takes any lead; the sync engine meets the same limits. Whether calendula should check the store's sources (as `--online-meeting` does) is left for review.
- **Live check not run.** The night session forbids live writes; the gcal and msgraph behaviour rests on the io-gcal and io-msgraph sources read above and on unit tests.
