---
cairn: change
change: event-delete-by-id-or-uid
---

# Delta

## ADDED Requirements

### Requirement: An event is deleted by its id or its UID
`event delete <EVENT-ID>` SHALL take the id `event list` reports or the event's iCalendar `UID`. It SHALL try the id first, by a read, then list the calendar's events and look for an item of that id, then for the items carrying that `UID`. One item SHALL be deleted whole; several carrying the `UID` SHALL be refused, naming their ids; none SHALL be a clear miss.
