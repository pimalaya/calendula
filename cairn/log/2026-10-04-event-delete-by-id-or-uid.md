---
cairn: log
change: event-delete-by-id-or-uid
landed: 2026-10-04
---

# `event delete` takes the id or the UID

`event delete` resolves its argument as the item id, then as an iCalendar `UID` among the calendar's events, refusing a `UID` several items carry. The other delete commands document the id their listing reports.

Capabilities moved: **commands** (an event is deleted by its id or its UID).
