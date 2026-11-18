---
cairn: log
change: a-pimdir-write-nobody-would-push
landed: 2026-11-18
---

# A pimdir write nobody would push is refused

Landed without a proposal: the owner's ruling of 2026-11-18 on TWN-3 (refuse, coded), the code agreed with cardamum's session as himalaya's `no-performer`, plus CAL-14's pimdir half and a test for CAL-16.

- **TWN-3**: `PimdirBackend::check_synced` reads the sources syncing the calendar (io-pimdir `PimdirReader::capabilities`, a binding or a checkpoint) and refuses `create_item`, `update_item` and `delete_item` with `no-performer` when there is none, before the body is staged. The test fixtures now record the `caldav` source through a closed full round (`client::sync_from`, `OpenRound` then `CloseRound`: a bare checkpoint reads as a first sync still filling in, which turns every search answer incomplete), in src/pimdir/{backend,index,queue}.rs, src/shared/event/update.rs and tests/pimdir_search.rs. Test `a_write_no_source_would_push_is_refused`.
- **CAL-14, the pimdir half**: `resolve_account` refuses a `pimdir.account` that none of the store's collections is grouped under, naming those held, when the store holds any; a fresh store still loads. Test `a_configured_account_the_store_lacks_is_refused`. The Google half (a paged listing folding exceptions only within the pages fetched) stays `[plausible]` and unproven, left as found.
- **CAL-16**: fixed by 7842252, whose `contains_in` reads a stamp's day with `get(..8)`; test `a_stamp_with_a_wide_character_is_compared_whole`.

Capabilities moved: backends.
