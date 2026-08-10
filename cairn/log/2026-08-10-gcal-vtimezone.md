---
cairn: log
date: 2026-08-10
change: gcal-vtimezone
---

# VTIMEZONE for the zones gcal names

Items the gcal backend produced referenced a time zone by TZID and carried no VTIMEZONE, against RFC 5545 3.2.19, so the document was not self-contained: a strict consumer could refuse it or fall back to UTC, and the bytes could not be shared as a standalone `.ics`. [Issue 8](https://github.com/pimalaya/calendula/issues/8) measured it at 118 of 128 items on one calendar, against 127 definitions from the same account over CalDAV.

## What landed

[src/gcal/timezone.rs](../../src/gcal/timezone.rs) projects an IANA name onto the component it denotes, reading tzdb through tz-rs, around an anchor the caller supplies.

Which half of the record answers depends on where that anchor lands. A TZif record stops where its closing POSIX rule takes over (America/New_York's last transition is March 2007), so an anchor past that point is described by `TimeZoneRef::extra_rule()` and an earlier one by the transitions bracketing it. Either way the result is a standard and daylight pair, each with its two offsets, a DTSTART on an occurrence of its own rule, and a yearly RRULE. On the rule path `RuleDay::MonthWeekDay` becomes `BYMONTH` and `BYDAY` directly, POSIX's fifth week becoming iCalendar's `-1`; on the transition path the rule is stated only where the next two transitions of the same kind agree on month, week of the month, weekday and local time, and a dated onset stands alone otherwise.

[src/gcal/project.rs](../../src/gcal/project.rs) reads the zones off the finished document rather than off the boundaries, folds resolved and only the parameter section of each line consulted, then splices one definition per undefined zone ahead of the VEVENT. The anchor is the day the event starts.

The observance DTSTART needed no offset arithmetic: RFC 5545 3.6.5 states it in the local time before its transition, and both a POSIX rule and a recorded transition give it the same way.

## What the fix pulled with it

**The stash had to stop carrying zones.** Left alone, `calendar_remainder` would have stashed the projection's own minted definitions straight back into `extendedProperties.private` on the next write, spending a chunk of a 1024-character budget on bytes regenerated for free on every read. A VTIMEZONE whose TZID the database knows is now dropped from the remainder; one it does not know is kept verbatim, since nothing could rebuild it.

**A TZID reaches the document through more than the boundaries.** The first cut collected zones as `push_boundary` wrote DTSTART and DTEND, which left an EXDATE, an RDATE or a RECURRENCE-ID naming a zone that nothing defined, and dropping the rebuildable definitions from the stash made that strictly worse than before the change. Reading the zones off the finished document instead states the invariant that was wanted all along: no TZID without a definition, wherever the reference came from. It subsumes the stash check too, since a zone the stash already defines is simply one the document defines.

**One era is emitted, and it is the item's own.** A zone's full history runs to hundreds of transitions and would be repeated on every item. A first cut emitted the rule currently in force, which is what Google's CalDAV frontend serves, but that reads an hour out for an item predating the rule: the United States moved its onsets in 2007, so a 2004 item in the weeks between the old and new dates resolves wrong. Anchoring the description on the item's start costs one extra code path and removes the whole class.

**A settled zone is described as settled.** Reading raw transitions revives rules a zone has abandoned: Hong Kong last shifted in 1979, and an item from today would otherwise carry that summer time. A zone whose nearest shift is more than two years from the anchor states the single offset in force instead. The rule path never had this problem, since Hong Kong's closing rule is fixed, so the guard exists for the transition path alone.

**The stack now carries a time zone database, and the instant rule did not move.** The projection spec justified projecting a `Z`-stamped boundary as a UTC stamp partly on there being no database to derive a wall time with. That clause is gone, and the requirement stands on its own ground: the stamp already answers the question about the instant. Minting a definition answers a question about the zone. Having the second does not license the first.

## How it is verified

The strong test does not read this module's output as text. It generates a zone, parses it back, and resolves civil times through ical-rs's own resolver, which works from the observances alone and shares no code with the generator. The offset it reports is compared against what tzdb puts in force at the instant that civil time then names: 864 samples across twelve zones and six years spanning half a century, covering both hemispheres, a half-hour shift (Lord Howe), a quarter-hour offset (Chatham), last-week rules, zones that never shift, and the years either side of the 2007 United States rule change. The two local times that are not one instant are pinned separately: 2024-03-10T02:30 in New York comes back as a gap, 2024-11-03T01:30 as a fold, neither of which a bare TZID could express.

## What came from the community patch

A patch on the issue proposed the same fix on a different footing, and four of its ideas are in what landed: reading the undefined zones off the finished document, anchoring the description on the item, guarding a settled zone, and restricting the TZID scan to the parameter section of a line. The first of those is the one that mattered most, since it exposed a real defect in the first cut rather than merely improving on it.

Two parts of that patch were not taken. It read the host's zoneinfo through jiff, which makes the document of record depend on the machine that produced it (two hosts with different tzdata emit different bytes for one event, and a container without zoneinfo emits none at all and silently reinstates the bug); an embedded database keeps the projection deterministic. It also left `calendar_remainder` untouched, so its synthesized definitions were stashed on the next write, and the component then came back through the stash after `END:VEVENT` instead of before it, losing both the budget and the ordering the patch had argued for.

## Capabilities moved

- **projection**: added the requirement that a named zone arrives with its definition, read off the finished document and describing the item's own era; the stash requirement now excludes a rebuildable VTIMEZONE; the instant-outranks-a-display-zone requirement dropped its no-database justification and states why a database does not change it.
- **packaging**: `gcal` carries the time zone database, so a build without that backend pays neither the dependency nor its embedded data.

## Left out

The zone's full transition history, as above. `originalStartTime` also carries a zone in the Google resource, but the projection does not read that field today, so nothing references it and nothing is owed for it.

The generator is calendula's, not ical-rs's. That crate's timezone module resolves offsets from the observances travelling inside the calendar and says in its header that it needs no time zone database and no new dependency; the promise is worth keeping. If tcal or another consumer needs the same name-to-observances step, it can be extracted then.
