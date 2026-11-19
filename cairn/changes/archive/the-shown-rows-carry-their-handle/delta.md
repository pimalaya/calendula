---
cairn: change
change: the-shown-rows-carry-their-handle
---

# Delta

## MODIFIED Requirements

### backends.md, pimdir reads and retracts its queue

`queue show` prints `{row, handle, state, collection, kind, attempts, error, appliedAt, seq, refusals}` and `queue cancel` `{row, handle, kind, unconfirmed}`: `row` the integer row id, `handle` the handle `<ROW>` gave, or the bare id it gave named under the store's identity as `queue list` prints it; the text names the row by that handle.
