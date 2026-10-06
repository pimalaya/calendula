---
cairn: change
id: event-field-flags
status: active
created: 2026-10-06
---

# Field flags for `event build`, `event create` and `event update`

## Why

cardamum's `card build` takes a source, field flags and the composer, stacked: `card build --full-name … --email work:…` writes a card with no editor and no hand-written vCard. calendula's `event build` takes a source and the composer only, so a script (MOA among them) that wants a plain event must write the whole iCalendar itself: `UID`, `DTSTAMP`, the zone and its `VTIMEZONE`, escaping. MOA keeps an iCalendar writer of its own for that reason (`servers/pim/ical.rs`).

The spec ruled flags out ("calendula SHALL NOT grow per-property flags"), on the grounds that a recurrence rule, a time zone and an attendee list are where a flag stops being ergonomic. cardamum drew the same line further out: flags for the common fields, the composer for the rest (`ADR` stays out). calendula can draw it there too.

## What

- `event build`, `event create` and `event update` take field flags, stacked as cardamum stacks them: the source (or the minted seed, or for an update the item the backend holds), then each flag, then `-i`. `event build` is the same pipeline stopped before the write.
- A flag sets its property on the VEVENT and touches nothing else: every instance of that property is dropped and the flag's own written, a repeated flag writing one instance per value; every other line keeps its bytes.
- Proposed flags, the fields `event list` renders and the few every calendar carries:
  - `--uid`, `--summary`, `--description`, `--location`, `--url`;
  - `--start` and `--end` (`YYYY-MM-DD` for a whole day, `YYYY-MM-DDTHH:MM[:SS]` local, or with `Z`), `--duration` (RFC 5545 duration, instead of `--end`), `--time-zone <IANA>` (the `TZID` of `--start` and `--end`, its `VTIMEZONE` minted from the database as the gcal projection already does);
  - `--status` (`tentative`, `confirmed`, `cancelled`), `--transparency` (`opaque`, `transparent`);
  - `--organizer [NAME:]ADDRESS`, `--attendee [NAME:]ADDRESS` (repeatable, `PARTSTAT=NEEDS-ACTION`, `RSVP=TRUE`), `--categories` (repeatable);
  - `--sequence`, and on `build` only `--method` (the calendar's `METHOD`, for an invitation).
- Out, left to the composer: `RRULE`, `RDATE`, `EXDATE`, overrides, `VALARM`, `CONFERENCE`, a `VTIMEZONE` given by hand.
- A build or a create from flags alone is checked as a composed item is (ical-rs validator, the `DTSTART` rule); a source still goes through untouched. A source holding several VEVENTs is refused when a flag is set, as cardamum refuses several vCards.

## Open questions

- `--start`/`--end` with a `TZID` written inline (`Europe/Paris:2026-10-19T09:00`) instead of `--time-zone`?
- Should `--attendee` take a role or a `PARTSTAT` (`[ROLE:]`, as `--email [TYPE:]`), or stay plain and leave the rest to the composer?
- Does `event update` with `--start` alone keep the length (move `DTEND` with it), or set `DTSTART` only?
