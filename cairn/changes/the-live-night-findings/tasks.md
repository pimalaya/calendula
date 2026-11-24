---
cairn: tasks
change: the-live-night-findings
---

# Tasks

- [x] C1: on `invalidProperties` naming `iCalendar`, write again without it and warn. Test: a scripted JMAP server refusing the member once; live re-run event create, update and delete on fastmail-jmap. Landed 2026-11-24 (log 2026-11-24-a-jmap-write-refused-for-the-hatch); live passed only with a stand-in for C1b.
- [ ] C1b: Fastmail then refuses the create `invalidProperties` (`version`): io-jmap's `JmapCalendarEvent::from_ical` drops the JSCalendar 2.0 Group's `version` when lifting the entry out. Waits on io-jmap; then pin it and re-run the live create, update and delete on fastmail-jmap.
- [ ] C2: ask Fastmail about the member; fold the answer into projection.md.
- [ ] Next live run: microsoft-pimdir calendars (add `msgraph.calendar` to the live neverest rc), `pimdir reply`/`cancel` drained with a run-owned guest, the google and microsoft suites.
