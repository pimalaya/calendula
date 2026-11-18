---
cairn: log
change: the-listed-rows-carry-their-handle
landed: 2026-11-18
---

# The listed rows carry their handle

`pimdir queue list` and `pimdir status` print every row's handle (`<store id>.<incarnation>.<row>`, pimdir STORAGE §4.5) beside its `row` in the JSON and in its place in the text, so a row cancelled from a listing is checked against a restored copy of the store as one from a receipt is (cairn/changes/archive/the-listed-rows-carry-their-handle). The identity comes from `PimdirReader::change_cursor`, public in io-pimdir 0.17.0: the 2026-11-18 the-releases-land entry was wrong to say no accessor existed, and no io-pimdir release is needed.

Tests: `a_queued_creation_lists_shows_and_cancels` follows and cancels the row by its listed handle, checked to name the store; `status_shows_the_parked_actions_of_the_account_read` parses the status handle back to the row.

Moved: backends.md (queue, status), commands.md (status, queue listing).

Left: `handle` is a key calendula's queue row adds to the family's shape (pimalaya-cli output.md, *A queue row keeps one shape*); Himalaya and cardamum print none yet.
