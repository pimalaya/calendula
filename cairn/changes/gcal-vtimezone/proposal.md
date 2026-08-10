---
cairn: change
id: gcal-vtimezone
status: landed
created: 2026-08-10
---

# VTIMEZONE for the zones gcal names

## Why

[Issue 8](https://github.com/pimalaya/calendula/issues/8): items the gcal backend produces reference a time zone by TZID and carry no VTIMEZONE, so the document is not self-contained. Across the reporter's 128-event calendar, the CalDAV backend returned 127 zone definitions and gcal returned none, with 118 items referencing a zone nothing in the document defines. RFC 5545 3.2.19 requires a TZID to name a VTIMEZONE the same object carries. A strict consumer may refuse such an item or fall back to UTC, and the bytes cannot be shared as a standalone `.ics`.

The gap is not a parsing miss. The Calendar API v3 Event resource carries a zone as an IANA name in `start.timeZone`, `end.timeZone` and `originalStartTime.timeZone` and nothing else: no offsets, no transition rules, no iCalendar text anywhere in the resource, and `events.list`'s `timeZone` parameter only chooses the zone the response renders in. io-gcal models all three fields faithfully. There is no property to read.

What made the gap invisible until now is the calendar-level stash. A VTIMEZONE that arrives inside a document calendula writes is stashed under `calendula.vcal.` and spliced back on read, so a zone survives a round-trip through Google. That covers only the events calendula itself wrote from an ics that already carried a definition. An event created in Google's web UI, or by any other client, has no stash and therefore no zone, which is the 118.

The same Google answers differently over CalDAV because the CalDAV wire format *is* iCalendar: RFC 4791 obliges `calendar-data` to be a valid iCalendar object, so Google's CalDAV frontend expands the name into observances server-side, from the database it obviously holds. The REST format is JSON, which carries no such obligation and offers no slot. The expansion has to happen on our side of the call.

Whose obligation it is follows the author of the document. Google emits JSON, so nothing attaches to it. calendula emits iCalendar under its own PRODID, so the moment the projection writes a TZID, 3.2.19 attaches to calendula. That the upstream resource is thin explains the gap; it does not discharge it.

## What

The `gcal` feature carries a time zone database, and the projection mints the VTIMEZONE for every zone its boundaries name.

### Only the rule in force

A zone's full history runs to hundreds of transitions, far too many to repeat on every event, so what is emitted is the POSIX rule closing the zone's TZif record: the standard and daylight pair currently in force, each as a yearly RRULE. That is the same two-observance shape Google's own CalDAV frontend serves, and what desktop clients write. An event predating the current rule resolves against a rule that was not yet in force, which is the accepted trade: shipping the historical transitions with every item costs far more than the accuracy is worth on a calendar read for the present and the near future.

### The stash stops carrying what can be rebuilt

A definition the projection can mint again from its name is no longer stashed on the way in. This matters beyond tidiness: a zone runs to a dozen lines or more, and Google caps an extended property at 1024 characters, so stashing one spends a real part of a budget shared with every other unmodelled line. A definition under a name the database does not know is still kept verbatim, since nothing could rebuild it. An event whose stash predates this change keeps its definition and is not doubled by a minted one.

## Dependency

[tzdb](https://crates.io/crates/tzdb) with [tz-rs](https://crates.io/crates/tz-rs), behind the `gcal` feature, both released.

chrono-tz is the obvious candidate and the wrong one: its public API exposes offset lookup at an instant and no transition list, so a VTIMEZONE would have to be reverse-engineered by sampling offsets day by day and emitted as a bounded window of RDATEs, which runs out exactly where a recurring series needs it. tz-rs exposes `TimeZoneRef::extra_rule()`, and its `RuleDay::MonthWeekDay` is month, week and weekday, which is an RRULE already.

The database does not belong in ical-rs. Its timezone module resolves offsets from the observances travelling inside the calendar, advertising in its own header that it does so with no time zone database and no new dependency, and that promise is worth keeping. If tcal or another consumer needs the same step, extract it then.
