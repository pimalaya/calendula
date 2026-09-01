---
cairn: log
change: an-event-owes-a-dtstart
landed: 2026-09-01
---

# An event owes a DTSTART, and the check that exists to say so did not

The composer landed this evening with a check that refuses what an editor hands back, and the first real `event create -i` against a Posteo calendar answered `WebDAV server returned HTTP 500`. The item was a VEVENT with a `UID`, a `DTSTAMP` and a `SUMMARY`, and no `DTSTART`.

**The rule ical-rs cannot state** (shared/ical.rs `missing_dtstart`): RFC 5545 3.6.1 makes `DTSTART` required of a VEVENT unless the calendar specifies a `METHOD`. Every other requirement the crate models is a property list on a component, and this one is a condition on the component's parent, so `IcalComponentSpec::required_props` for VEVENT is `UID` and `DTSTAMP` and stops there. Nothing was wrong with the validator; the rule is not expressible in it.

It is worth stating in calendula rather than upstream-only because it is the one property a person writing an event by hand leaves out, and because of what a server does with it. SabreDAV denormalizes `DTSTART` into its object index on write, so a VEVENT without one is not refused with a 400 naming the property: it is a 500 naming nothing. The check exists precisely for what a plain editor gets wrong, and this was the first thing it got wrong.

**The seed owed one too** (`blank_item`). Adding the rule turned calendula's own minted event into something its own check refused, which the test suite caught before anything else did. A minted VEVENT now carries a `DTSTART` of now, the same stamp its `DTSTAMP` gets. That value is a placeholder as the `UID` and the `PRODID` are, and an event with no time is not an event; the alternative, a seed guaranteed to trip the check on the way back, makes the composer prompt for a re-edit on the commonest path there is. A VTODO and a VJOURNAL owe no `DTSTART` under 3.6.2 and 3.6.3 and get none.

`event build <source>` still prints an unchecked source: what is checked is what a composer wrote, which is unchanged.

Verified: 103 tests green, three new ones over the refusal, the `METHOD` exemption and the two kinds that owe nothing; the failing draft from the report, replayed through a composer, now stops at `Component VEVENT is missing required property DTSTART (RFC 5545 3.6.1, the calendar specifying no METHOD)` and offers the re-edit.

Capabilities moved: commands (two requirements modified).
