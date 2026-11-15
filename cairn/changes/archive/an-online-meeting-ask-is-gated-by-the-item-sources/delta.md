---
cairn: change
change: an-online-meeting-ask-is-gated-by-the-item-sources
---

# Delta

## MODIFIED Requirements

### Requirement: pimdir refuses an online meeting no source can create
A `create_item` or `update_item` whose body newly asks for an online meeting (`X-PIMDIR-ONLINE-MEETING:TRUE`, an ask the item already carried being none) SHALL be refused, before its body is staged and anything is queued, unless a source concerned declares `calendar.online-meeting` fully or in part (STORAGE Annex B.1, §15.6): for a create, a source syncing the calendar; for an update, a source binding the item, the one that pushes it. A store whose sources declare nothing is refused too, since no sync engine there would act on the ask. The error SHALL start `Stage the pimdir action:` as the store's other refusals do, and name the capability and the sources declaring without it, or say no source of the store declares it. A source declaring it in part passes, with the store's partial-support warning.
