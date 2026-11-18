---
cairn: change
id: the-listed-rows-carry-their-handle
status: landed
created: 2026-11-18
---

# The listed rows carry their handle

## Why

`pimdir reply` and `pimdir cancel` print a row handle, checked by `pimdir queue show` and `cancel` against a restored copy of the store, but `pimdir queue list` and `pimdir status` printed bare row ids, which are followed unchecked. A row picked from a listing is the common way to cancel one, so the check missed the common path.

## What

Every row of `pimdir queue list` and every parked row of `pimdir status` carries `handle`, built from the store's identity as `PimdirReader::change_cursor` reads it (io-pimdir 0.17.0), and the text prints the handle in place of the bare row. `row` stays in the JSON, the family's queue row keeping its keys.
