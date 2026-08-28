---
cairn: change
change: calendar-resource-granularity
---

# Delta

## ADDED Requirements

### Requirement: A recurrence set is one resource
Google returns a modified instance of a recurring event as an event of its own, carrying `recurringEventId` and `originalStartTime`. The projection SHALL emit a RECURRENCE-ID for such an event, naming the instance it replaces (RFC 5545 3.8.4.4), or the exception reads as an unrelated event.

A listing SHALL fold the exceptions of a series into the document of the master they modify, so a series and its modified instances are one item rather than several. The folded components SHALL carry the master's UID, since the components of one resource share one UID (RFC 4791 4.1) and an exception projected alone would otherwise mint its own. The item SHALL keep the master's event id as its identifier, and an exception whose master the batch did not return SHALL still file on its own rather than be dropped.

Reading one item SHALL fold the same way, gathering the exceptions through a listing filtered by the series' iCalUID: every event of a series carries the series' own iCalUID, the API offers no query for the children of an event, and expanding the instances would return the occurrences the recurrence rule already generates rather than the modifications alone.

The zones SHALL be minted over the folded document, so an exception moved into a zone the master never named still arrives with the definition RFC 5545 3.2.19 owes it.

## MODIFIED Requirements

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is one calendar object resource of it: a single event, or a recurring event together with the exceptions that modify its instances (see [A recurrence set is one resource](./projection.md#requirement-a-recurrence-set-is-one-resource)). Item ids SHALL be the event ids the API returned, verbatim, and the id of a set SHALL be the master's. `create_item` SHALL use `events.import` when the projected event carries an iCalendar UID, so the UID survives, and `events.insert` otherwise. `update_item` SHALL honour `if_match` as the API's `If-Match` header, so the optimistic concurrency Google offers is not silently dropped.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL NOT be honoured and the [assigned one](#requirement-a-create-reports-the-identifier-it-was-given) SHALL be reported instead. `update_calendar` SHALL patch the calendar for the name and the description and the calendar list entry for the colour, since Google splits those across the two resources; a Google calendar always carries a colour, so clearing one SHALL be refused by name.

### Requirement: The text/calendar summary convention
calendula SHALL write, and read, the pimdir `text/calendar` summary at `v: 1`: an optional `uid`, an optional `component` (`VEVENT`, `VTODO` or `VJOURNAL`), a required (possibly empty) `summary`, an optional `dtstart` carried verbatim beside its `dtstart_tzid` and `dtstart_value` (`date-time` or `date`), an optional verbatim `dtend`, an optional verbatim `due`, whether the item is `recurring`, and an optional `size`.

Times SHALL be carried verbatim rather than as resolved instants, so a reader with a time zone database re-derives an instant in its own zone and a reader without one displays the wall time the calendar wrote instead of a UTC claim the writer fabricated. The resolved instant SHALL NOT be duplicated in the summary: the `sort_key` is returned by the store's paging reads and is the single resolved projection.

The summary SHALL describe the master of a recurrence set, the component carrying no RECURRENCE-ID, since that is the item as a reader lists it. A resource carrying overrides alone SHALL still be summarised, from the first of them.

The companion `sort_key` SHALL hold the item's start normalised to RFC 3339 in UTC at seconds precision, read ascending: DTSTART for a VEVENT or a VJOURNAL, and DUE then DTSTART for a VTODO, which is scheduled by its due date (RFC 5545 3.8.2.3) and need not carry a DTSTART at all.

A UTC value SHALL be taken verbatim. A zoned one SHALL resolve through the VTIMEZONE the document carries, taking the earlier offset when the local time is ambiguous and the offset after the transition when it does not exist, since a local time at a transition names two instants or none. A zoned one whose zone the document does not define SHALL be read as floating rather than left unknown: the error is bounded by the offset and keeps the item near its place, where dropping the key would move every such item to the far end of the listing. A date-only value SHALL be read at midnight UTC and a floating one on the wall clock, both conventions rather than facts. An item with nothing parseable at all SHALL keep an empty key, which reads as unknown.

A recurring item SHALL key on its first occurrence, which is what DTSTART holds and is fixed for the life of the series. A date-range read over recurring items therefore needs the recurrence expanded above the store, expansion being a function of when you ask.
