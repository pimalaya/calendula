---
cairn: change
change: ical-rs-0-6
---

# Delta

## MODIFIED Requirements

### Requirement: A composed item is checked before it is written
What the composer wrote SHALL be checked against the RFC 5545 contract through ical-rs's validator rather than a look at its first line. Reading is liberal and this is the strict half: a VCALENDAR missing its required PRODID, a VEVENT missing its UID or DTSTAMP, are caught here rather than by the server or by nobody.

A VEVENT SHALL also be refused when it carries no DTSTART and the calendar specifies no METHOD (RFC 5545 3.6.1). That requirement is a condition on the enclosing calendar rather than a property list, so ical-rs's per-component contract cannot state it; calendula SHALL state it, because it is the one thing a person writing an event by hand leaves out and because a server does worse than refuse it: SabreDAV denormalizes DTSTART into its index on write and answers HTTP 500.

The check SHALL NOT refuse what RFC 5545 allows: a `DATE` value and a `TZID` on `DTSTART`, `DTEND`, `DUE`, `RECURRENCE-ID`, `EXDATE` and `RDATE`, and the `CN`, `DIR`, `SENT-BY` and scheduling parameters on `ORGANIZER`. A whole-day, zoned or organized event is not invalid. Every finding of ical-rs's validator SHALL be reported, a duration outside RFC 5545 3.3.6 included.

An item that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes. This is not a menu: the only question is whether to fix it, and declining is an error rather than an abandon.

An iCalendar given on the command line SHALL NOT be checked, going to the backend as it was written: that is the promise the projections already make, calendula never rewriting bytes it was handed.


### Requirement: A field flag sets its property and touches nothing else
Writing a field flag SHALL drop every instance of the property it names on the VEVENT and write the flag's own, a repeated flag writing one instance per value. Every other line SHALL keep its bytes. A time flag SHALL take a date, a local time, a `Z` time, or a time prefixed by an IANA zone (`Europe/Paris:2026-10-19T09:00`); `--time-zone` names the zone of the times that name none. A zone a flag names SHALL arrive with its `VTIMEZONE`, minted once from the time-zone database. On an update, `--start` given without `--end` or `--duration` SHALL move the end with it, keeping the event's length.

A line a flag writes longer than 75 octets SHALL be folded (RFC 5545 3.1), never inside a character. `--duration` SHALL take an RFC 5545 3.3.6 duration in any case, a leading `+` dropped, checked by ical-rs's validator; a negative one SHALL be refused.

### Requirement: A window expands recurring events
With a window, `event list` SHALL list every occurrence overlapping it rather than every VEVENT: the recurrence set of each `UID` (`DTSTART`, `RRULE`, `RDATE`, minus `EXDATE`, with its `RECURRENCE-ID` overrides) walked by ical-rs, an override standing in for the instance it names, and one moved into the window from an identity past it still listed. An override travelling without its series lists on its own.

A `UNTIL`, an `EXDATE`, an `RDATE` or a `RECURRENCE-ID` written on another clock than the series' `DTSTART` (UTC beside a zoned start, as RFC 5545 3.3.10 requires of `UNTIL`) SHALL be told on the `DTSTART`'s clock before it bounds, removes, adds or overrides an instance, through the zone each `TZID` resolves to.

An occurrence SHALL carry the item `id` and its `recurrenceId`, the instance identity spelled as the series' `DTSTART` (a date, a local time, or a `Z` time), which `event read -r` takes back. Occurrences SHALL list in start order. A timed occurrence overlaps when it starts before the window's end and ends after its start (a zero-length one when it starts inside); a whole-day one compares by date. A window open on its end SHALL reach a year past its start.

Without a window a series SHALL list once, at its first start.
