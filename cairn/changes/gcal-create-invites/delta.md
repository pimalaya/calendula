---
cairn: change
change: gcal-create-invites
---

# Delta

## MODIFIED Requirements

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is one calendar object resource of it: a single event, or a recurring event together with the exceptions that modify its instances (see [A recurrence set is one resource](./projection.md#requirement-a-recurrence-set-is-one-resource)).

Item ids SHALL be the event ids the API returned, verbatim, and the id of a set SHALL be the master's.

`create_item` SHALL use `events.insert` with `sendUpdates=all` when the projected event names at least one `ATTENDEE` and either no `ORGANIZER` or one whose address is the calendar id (case-insensitively), carrying its iCalendar UID as `iCalUID`, so the attendees are invited and the UID survives. Otherwise it SHALL use `events.import` when the event carries a UID, so the UID survives, and `events.insert` when it carries none. `update_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL NOT be honoured and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported instead.

`update_calendar` SHALL patch the calendar for the name and the description and the calendar list entry for the colour, since Google splits those across the two resources. A Google calendar always carries a colour, so clearing one SHALL be refused by name.

#### Scenario: An event created with attendees invites them
- GIVEN a gcal calendar and an event naming an `ATTENDEE` and no `ORGANIZER`, or the calendar's own address as `ORGANIZER`
- WHEN it is created
- THEN it is inserted with `sendUpdates=all` and its `iCalUID`, and Google mails the attendee

#### Scenario: Someone else's invitation is imported
- GIVEN a gcal calendar and an event whose `ORGANIZER` is another address
- WHEN it is created
- THEN it is imported, and nobody is mailed
