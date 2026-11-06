---
cairn: log
change: pimdir-search-reads-local-days
landed: 2026-11-06
---

# pimdir search reads local days

On io-pimdir 5c18a02 (change pimdir-follows-storage-draft-09), `pimdir search` and `pimdir count` read `date:` days, `today` and `yesterday` in the zone the tables read times in, and say when the index reads floating times in another.

- `search_clock` (src/pimdir/index.rs) builds the `PimdirSearchClock` the commands pass: `zone` is `TimeZone::try_system()` (`TZ`, then the system's) by its IANA name; a zone with no IANA name keeps UTC with one `warn!` pointing at `TZ`. `count_items` now takes the clock, as `search_items` did through its options, so tests inject a zone at that boundary rather than through the process environment.
- `PimdirCalendarZones` (the index's `PimdirIndex::floating_zone`, the clock's zone, `UTC` when it states none) rides on the page and the count when the two differ, and prints one line after the coverage line: "The index reads floating times and dates in UTC, this search reads days in Europe/Paris". It is `serde(skip)`: the JSON output is unchanged.
- The `pimdir search` help states local days and the zone line; `pimdir count`'s help drops "days read in UTC". `index-zone-unknown` now also covers a search zone the database does not know (io-pimdir raises `IndexZone` for either), in src/error.rs and the spec's code table.
- Deviation from the delta as proposed: the zone line is printed by `pimdir count` too, under its count, since it reads days the same way; the delta was amended before folding.

Tests (src/pimdir/index.rs, src/pimdir/search.rs): `a_day_is_the_local_day` (an event at 00:30 Europe/Paris on 2026-11-07, its index stating Europe/Paris, found on the 7th and not the 6th, counted once), `an_all_day_event_stays_on_its_day` (an all-day event on 2026-11-07 indexed and searched in Europe/Paris and in America/New_York, found on the 7th alone), `an_index_read_in_another_zone_is_named` (a UTC index searched in Europe/Paris names both zones on search and count; a UTC or zoneless clock names none), and the rendering test now checking the line after the coverage. `a_day_is_a_utc_day` is replaced by the first. Checked 2026-11-06 by forcing the tests' clock to no zone (UTC): all three failed (Paris's all-day event listed on the 6th, New York's on the 8th); with `PimdirCalendarZones::of` answering `None`, the third failed.

`cargo test --all-features` 270 passed; `cargo clippy --all-features --all-targets` clean, and each of caldav, gcal, msgraph, vdir and pimdir alone with rustls-ring, and the default set; `pimdir` alone 222 passed.

Not verified here: neverest stating the user's zone as `floating_zone`, in progress in neverest (its change the-index-reads-the-user-s-zone, uncommitted when this landed). Until it does, a store neverest indexes in UTC and a search in a zone east or west of it are off for floating and all-day items, which the zone line now says.

Capabilities moved: **commands** (pimdir searches calendar items through the store's index, a pimdir index or query failure is coded, a JSON error carries a stable code, modified; a pimdir search names an index read in another zone, added).
