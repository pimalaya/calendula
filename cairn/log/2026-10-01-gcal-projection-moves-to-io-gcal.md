---
cairn: log
change: gcal-projection-moves-to-io-gcal
landed: 2026-10-01
---

# The Google event projection moves to io-gcal

Neverest will sync Google calendars through the native API and needs the event to iCalendar projection without depending on Calendula, so it moved down into io-gcal behind its new `ical` feature, VTIMEZONE synthesis included, the way the Graph and People contact projections moved into io-msgraph and io-gpeople.

The four entry points became methods on `GcalEvent`: `to_ical`, `to_ical_series` (was `set_to_ical`), `from_ical` (was `to_event`) and `merge`, now called on the projected event with the server copy as argument. The 28 tests that came along (21 projection, 7 time zone) pass there unchanged; the one reading the document back through Calendula's own `Event` stayed here, in the Google backend's tests.

The stash prefixes (`calendula.ical.`, `calendula.vcal.`) and the `PRODID` keep their values, so events stashed by an earlier Calendula still read back and a document reads the same. In io-gcal the date handling moved from chrono to jiff, which the time zones already needed, and the `anyhow` errors became `GcalEventIcalError`, its messages unchanged.

Calendula no longer depends on jiff: the `gcal` feature now enables `io-gcal/ical`, which bundles the time zone database itself. It landed on io-gcal 0.1.2 and ical-rs 0.5.2, which gained the `IcalCst::empty` and `IcalProp::text` builders the projection uses.

Capability moved: **projection** (the Google link now points at io-gcal). No behaviour change.
