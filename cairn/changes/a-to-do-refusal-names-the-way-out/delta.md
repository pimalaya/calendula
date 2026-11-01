---
cairn: change
change: a-to-do-refusal-names-the-way-out
---

# Delta

## ADDED Requirements

### Requirement: pimdir refuses a to-do where its calendar takes none
A `create_item` or `update_item` whose body is a VTODO SHALL be refused, and nothing queued, when a source concerned states `x-vtodo` `none` for the calendar or source-wide (pimdir STORAGE §15.6): for a create, a source syncing the calendar; for an update, a source binding the item. The error SHALL start `Stage the pimdir action:`, say the calendar takes no to-do, name the source and its stated reason when it gives one, and point at `-k/--calendar` to store the to-do in another calendar.
