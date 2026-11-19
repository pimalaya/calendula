---
cairn: change
id: the-shown-rows-carry-their-handle
status: landed
created: 2026-11-19
---

# The shown rows carry their handle

## Why

`pimdir queue list`, `pimdir status` and the receipts print a row's `handle` beside its `row`, but `pimdir queue show` and `pimdir queue cancel` printed `row` alone. pimalaya-cli output.md (*A queue row keeps one shape*, the receipt keys) names a queued action's row id `row` and its handle `handle` in every binary, and Himalaya prints both since b6eed3e8. Owner ruling of 2026-11-19.

## What

`queue show` prints `{row, handle, state, ...}` and `queue cancel` `{row, handle, kind, unconfirmed}`: `row` the integer row id, `handle` the handle given, or a bare id given named under the store's identity (`PimdirReader::change_cursor`, the identity `queue list` names its handles under). The text names the row by its handle. `collection` stays the kit's shared key.
