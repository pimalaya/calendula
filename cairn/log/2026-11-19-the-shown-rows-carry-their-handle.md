---
cairn: log
change: the-shown-rows-carry-their-handle
landed: 2026-11-19
---

# The shown rows carry their handle

`pimdir queue show` and `pimdir queue cancel` print `handle` beside the integer `row`, as Himalaya does since b6eed3e8 and pimalaya-cli output.md asks (owner ruling of 2026-11-19, cairn/changes/archive/the-shown-rows-carry-their-handle). Given a handle, `handle` is the handle given, another store's included; given a bare id, it is that id named under the store's identity, read through `PimdirBackend::store_identity` (`PimdirReader::change_cursor`) as `queue list` names its rows. The text names the row by its handle. `collection` stays the kit's shared key.

Tests: `an_applied_add_prints_its_seq`, `an_unknown_row_prints_its_id_and_state` and `cancelling_an_unconfirmed_row_says_it_may_have_taken_effect` expect `handle`, and failed on the old shape before the change; `a_row_is_named_under_the_store_identity` checks a bare id names the handle `queue list` printed and a handle is kept as given.

Moved: backends.md (pimdir reads and retracts its queue).
