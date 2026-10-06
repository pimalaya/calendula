---
cairn: log
change: event-field-flags
date: 2026-10-06
---

# Field flags for `event build`, `event create` and `event update`

A plain event is now written from the command line, with no editor and no hand-written iCalendar: `UID`, `DTSTAMP`, the zone and its `VTIMEZONE` and the escaping are calendula's. MOA can drop its own iCalendar writer for it.

## What landed

- `EventFieldsArgs` (`src/shared/event/fields.rs`), flattened into the three commands: `--uid`, `--summary`, `--description`, `--location`, `--url`, `--start`, `--end`, `--duration` (exclusive with `--end`), `--time-zone`, `--status`, `--transparency`, `--organizer`, `--attendee` (repeating), `--categories` (repeating, one `CATEGORIES` holding the list, as cardamum's `--nickname`), `--sequence`; `--method` on `event build` alone, set on the calendar.
- A flag drops every instance of its property and writes its own where the first stood, or ahead of any nested component; every other line keeps its bytes, and no flag leaves the source byte for byte. Setting the end clears `DURATION` and the reverse. Text is escaped and a line longer than 75 octets folded.
- Times: a date (`VALUE=DATE`), a local time, a `Z` time, or one prefixed by an IANA zone; `--time-zone` zones the rest. A zone named gets one `VTIMEZONE` from `tzdb::vtimezone`, ahead of the VEVENT, unless the calendar defines it; an unknown zone is refused by name.
- `event update --start` alone moves `DTEND` by the event's length as the projection reads it (a `DURATION` is kept as it is).
- A source holding several VEVENTs, or none, is refused when a flag is set. A build or create from flags alone is checked (`ensure_valid`); a source is not.
- The check no longer refuses a `DATE` or a `TZID` on the time properties nor a `CN` on `ORGANIZER`, which ical-rs 0.5.3's contract omits: whole-day, zoned and organized events were refused by the composer check too.
- `IcalBuildArgs::execute_with` takes the family's fields (`IcalFields`); `todo`, `journal` and `item` builds are unchanged.
- Capability moved: `commands` (the source, the field flags and the composer stack; the composer stays the complete surface; the composed-item check).

## Verification

`cargo test --all-features` (152 tests, 33 new: 29 in `fields.rs`, 3 in `event/build.rs` with the JSON projection of a build from flags, 1 in `ical.rs`), `cargo clippy --all-features --all-targets` clean, `cargo fmt`, `cargo build --no-default-features --features pimdir`. `calendula event build --summary … --start Europe/Paris:2026-10-19T09:00 --end 2026-10-19T10:00 --time-zone Europe/Paris --attendee Bob:bob@example.org --method request` prints a `REQUEST` with one minted `Europe/Paris` `VTIMEZONE` ahead of the VEVENT.
