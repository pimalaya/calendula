---
cairn: log
change: the-live-night-findings
landed: 2026-11-24
---

# A JMAP write refused for the hatch

C1 of the-live-night-findings: Fastmail JMAP refused every event write.

- Live on fastmail-jmap, in calendar `live-2026-11-24-calendula`, `event add` of a plain `DTSTART`/`DTEND` event failed "JMAP CalendarEvent create rejected: invalidProperties (`iCalendar`)": ical-rs records the `DTEND` spelling of `duration` in the event's `iCalendar` member, which Fastmail refuses and Stalwart takes.
- `JmapBackend::create_item` and `update_item` now send a write refused `invalidProperties` naming `iCalendar` members alone again once without every one of them, nested ones included (`project::strip_hatches`, `strip_patch_hatches`), and warn naming what they kept. Any other refusal, or a second one, fails as before.
- Tests: `a_write_refused_for_the_hatch_goes_again_without_it` (red before the fix: "JMAP CalendarEvent create rejected: invalidProperties (`iCalendar`, `participants/1/iCalendar`)") and `the_hatches_go_and_what_they_kept_is_named`.
- Live: past the hatch, Fastmail refused the create again, `invalidProperties` (`version`): it requires `version` on a CalendarEvent, which io-jmap's `JmapCalendarEvent::from_ical` drops when it lifts the entry out of ical-rs's JSCalendar 2.0 Group. A raw probe confirmed it (no `version` refused, `version` 2.0 created, `version` with `iCalendar` refused on `iCalendar`); Stalwart takes `version` 2.0. With a stand-in for the io-jmap fix (a temporary build, not landed), create, update (with and without a hatch) and delete passed on Fastmail, each hatch refusal warned. The real build still fails a Fastmail create until io-jmap carries `version`.

Capabilities moved: projection.
