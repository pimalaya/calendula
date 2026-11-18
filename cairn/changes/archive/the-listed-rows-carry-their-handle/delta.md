---
cairn: change
change: the-listed-rows-carry-their-handle
---

# Delta

## MODIFIED Requirements

### backends.md, The pimdir queue is read and retracted

`queue list` rows carry `handle` (`<store id>.<incarnation>.<row>`) beside the family's keys, printed in the text in place of the bare row; `<ROW>` takes a handle as `queue list`, `pimdir status` and a receipt print it. `pimdir status` prints each parked row's `handle`, and the handle in its ROW column.

### commands.md, pimdir status and the queue listing

`pimdir status` parked rows are `{row, handle, calendarId, kind, error}`; the text of both shows the queue row handle.
