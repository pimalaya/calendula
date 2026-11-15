---
cairn: change
id: event-attendee-add-remove
status: landed
created: 2026-10-28
---

# A field flag adds or removes one attendee

## Why

`event update ID --attendee …` replaces the whole attendee list, and every attendee it writes is invited plainly (`PARTSTAT=NEEDS-ACTION`, `RSVP=TRUE`) (#22). Adding one guest to a meeting means repeating everyone already on it, and forgets who had accepted or declined. The way round today is a wrapper that reads the event, edits its `ATTENDEE` lines and feeds the iCalendar back to `event update`, with its own folding of long lines.

## What

- `event build`, `event create` and `event update` take `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS`, each repeatable, spelled as `--attendee` is (a bare address or a `mailto:` URI, in any case).
- `--add-attendee` writes one `ATTENDEE` invited plainly, as `--attendee` does, after the event's last `ATTENDEE` (confirmed by Clément in #22; where `--attendee` would write it when the event has none). `--remove-attendee` drops the `ATTENDEE` carrying that address. Every other line keeps its bytes, the other attendees' `PARTSTAT`, `ROLE`, `RSVP` and `CN` included.
- Addresses are compared case-insensitively on the `mailto:` value, a leading `mailto:` in any case dropped first.
- Refused, naming the address, so a call that changes nothing never looks like it succeeded (decided with Clément in #22):
  - adding an address the event already carries, whatever name is given with it (`--add-attendee "New Name:ann@example.org"` on an existing Ann);
  - removing an address the event does not carry;
  - giving `--attendee` together with either flag.
- Giving one address to both flags in one call is refused as well, naming it: removing and adding it back would reset its answer, which is what `--attendee` already does on purpose (confirmed by Clément in #22).
- Every check runs before anything is written, so a refused call leaves the event untouched; the existing rule that a field flag is refused on an item holding several VEVENTs applies to both flags.
- The requirement "A field flag sets its property and touches nothing else" is amended: these two flags touch one `ATTENDEE` instead of dropping every instance.

## Out of scope

- Mailing the attendee added or removed: on gcal that is the `--send-updates` of #21.
- Changing one attendee's answer or role: that is a `REPLY` (`pimdir reply`) or the composer.

## Scope

`event build`, `event create` and `event update`, as Clément asked in #22: the three share the field flags ("A source, the field flags and the composer stack"), and on `build` or `create` the two flags work on the source or the minted seed.
