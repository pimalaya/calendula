---
cairn: change
change: event-attendee-add-remove
---

# Delta

## ADDED Requirements

### Requirement: A field flag adds or removes one attendee
`event build`, `event create` and `event update` SHALL take `--add-attendee [NAME:]ADDRESS` and `--remove-attendee ADDRESS`, each repeatable and spelled as `--attendee` is. `--add-attendee` SHALL write one `ATTENDEE` with `PARTSTAT=NEEDS-ACTION` and `RSVP=TRUE`, and a `CN` when a name is given, after the event's last `ATTENDEE`, or where `--attendee` writes when the event has none. On `event build` and `event create` the two flags work on the source or the minted seed, as every field flag does. `--remove-attendee` SHALL drop the `ATTENDEE` carrying that address. Addresses SHALL be compared case-insensitively on the `mailto:` value.

An address added that the event already carries, an address removed that it does not, an address given to both flags, and either flag given with `--attendee` SHALL each be refused naming the address or the flags, before anything is written.

#### Scenario: A guest joins
- GIVEN an event whose attendees are Ann, who accepted, and Bob, who declined
- WHEN `calendula event update --add-attendee carol@example.org ID` runs
- THEN Carol is written after Bob with `PARTSTAT=NEEDS-ACTION` and `RSVP=TRUE`, and Ann's and Bob's lines keep their bytes

#### Scenario: A guest already there
- GIVEN an event carrying `ATTENDEE:mailto:Ann@Example.org`
- WHEN `calendula event update --add-attendee "Ann B:ann@example.org" ID` runs
- THEN the update is refused naming `ann@example.org`, and the event is untouched

This folds into cairn/spec/commands.md.

## MODIFIED Requirements

### Requirement: A field flag sets its property and touches nothing else
Writing a field flag SHALL drop every instance of the property it names on the VEVENT and write the flag's own, a repeated flag writing one instance per value. `--add-attendee` and `--remove-attendee` are the exception: each SHALL touch the one `ATTENDEE` it names (see [A field flag adds or removes one attendee](#requirement-a-field-flag-adds-or-removes-one-attendee)). Every other line SHALL keep its bytes. A time flag SHALL take a date, a local time, a `Z` time, or a time prefixed by an IANA zone (`Europe/Paris:2026-10-19T09:00`); `--time-zone` names the zone of the times that name none. A zone a flag names SHALL arrive with its `VTIMEZONE`, minted once from the time-zone database. On an update, `--start` given without `--end` or `--duration` SHALL move the end with it, keeping the event's length.

A line a flag writes longer than 75 octets SHALL be folded (RFC 5545 3.1), never inside a character. `--duration` SHALL take an RFC 5545 3.3.6 duration in any case, a leading `+` dropped, checked by ical-rs's validator; a negative one SHALL be refused.

This folds into cairn/spec/commands.md.
