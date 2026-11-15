---
cairn: change
change: pimdir-honours-if-match
---

# Delta

## ADDED Requirements

### Requirement: A pimdir item's version is its body hash
The pimdir backend SHALL report as an item's `etag` the store's hash of its body, the pending queue folded in, so a staged update moves it at once. An item whose body is not local SHALL report none.

`update_item` and `delete_item` SHALL refuse, before anything is queued, a write whose `if_match` is not that version, surrounding double quotes ignored, with an error starting `Precondition failed:` and naming the version found and the one expected. An item with no local body matches no version.

### Requirement: A delete can be gated on a version
`event delete`, `todo delete`, `journal delete` and `item delete` SHALL take `--if-match <ETAG>`. CalDAV SHALL send it as `If-Match`, msgraph SHALL compare it with the event's `changeKey`, and pimdir SHALL check it as an update does. A backend that cannot gate a delete (vdir, gcal) SHALL refuse the flag rather than drop it.

`item read` SHALL add the `etag` of the version read to its JSON, `null` when the backend has none.

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir SPEC 15.1) through a producer opened for that write and dropped after it: `create_item` to `add`, `update_item` to `update`, `delete_item` to `remove`.

The body SHALL reach the blob tree through the blob writer, durably, before the row that pins it is appended, and the action SHALL address the item by the public `seq` that is already the item's shared id.

`update_item` and `delete_item` SHALL honour `if_match` against the item's version. The engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's.

Because a queued create carries no public id until the owner applies it, `create_item` SHALL report the item's link id instead.

### Requirement: The event projection
An event's JSON SHALL carry `id`, `etag` (the entity tag of its item, `null` when the backend has none), `uid`, `recurrenceId`, `summary`, `description`, `location`, `start` and `end` (iCalendar-spelled), `allDay`, `startsAt` and `endsAt` (RFC 3339 with offset, or `YYYY-MM-DD` with an exclusive end for a whole day), `timeZone`, `recurring`, `status`, `transparency`, `busyStatus` (Outlook's), `organizer` (`email`, `name`), `attendees` (`email`, `name`, `partstat` defaulting to `NEEDS-ACTION`, `role`, `rsvp`, `cutype`) and `onlineMeetingUrl` (`CONFERENCE`, then the vendors' properties). The end SHALL be `DTEND`, else the start plus `DURATION`, else a day for a date and the start for a time.

`event read` SHALL print the item's bytes, and its JSON SHALL add the projected `events`, or the one occurrence `-r/--recurrence-id` names.
