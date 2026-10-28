---
cairn: change
change: gcal-send-updates
---

# Delta

## ADDED Requirements

### Requirement: A gcal write chooses who is mailed
`event create`, `event update` and `event delete` SHALL take `--send-updates all|external-only|none`, which the gcal backend SHALL pass as the API's `sendUpdates` on the request it makes. Absent, update and delete SHALL omit `sendUpdates` and create SHALL choose as the Google Calendar backend requirement says, so nothing changes for a caller that does not give it.

On `event create` the flag SHALL override that choice wherever the event is inserted. An event that is imported instead mails nobody: `none` SHALL pass, and `all` or `external-only` SHALL be refused, naming the event's `ORGANIZER`. On any other backend the three commands SHALL refuse the flag by name, before reaching the backend. `event build` SHALL NOT take it.

#### Scenario: A quiet fix
- GIVEN a gcal event with an attendee
- WHEN `calendula event update --summary "Typo fixed" --send-updates none ID` runs
- THEN the event is updated with `sendUpdates=none`, and Google mails nobody

#### Scenario: An invitation stored for later
- GIVEN a gcal account
- WHEN `calendula event create --attendee ann@example.org --send-updates none …` runs
- THEN the event is inserted with `sendUpdates=none` and its `iCalUID`, and Ann is not mailed

#### Scenario: Another backend
- GIVEN a CalDAV account
- WHEN `calendula event delete --send-updates none ID` runs
- THEN the command is refused naming `--send-updates`, and nothing is deleted

This folds into cairn/spec/commands.md.

## MODIFIED Requirements

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is one calendar object resource of it: a single event, or a recurring event together with the exceptions that modify its instances (see [A recurrence set is one resource](./projection.md#requirement-a-recurrence-set-is-one-resource)).

Item ids SHALL be the event ids the API returned, verbatim, and the id of a set SHALL be the master's.

`create_item` SHALL use `events.insert` with `sendUpdates=all` when the projected event names at least one `ATTENDEE` and either no `ORGANIZER` or one whose address is the calendar id (case-insensitively), carrying its iCalendar UID as `iCalUID`, so the attendees are invited and the UID survives; a `--send-updates` given to the command SHALL replace that `all`. Otherwise it SHALL use `events.import` when the event carries a UID, so the UID survives, and `events.insert` when it carries none. `update_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped. `update_item` and `delete_item` SHALL send `sendUpdates` only when the command was given `--send-updates`.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL NOT be honoured and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported instead.

This folds into cairn/spec/backends.md.
