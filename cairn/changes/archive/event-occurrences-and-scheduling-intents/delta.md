---
cairn: change
change: event-occurrences-and-scheduling-intents
---

# Delta

## ADDED Requirements

### Requirement: A window expands recurring events
With a window, `event list` SHALL list every occurrence overlapping it rather than every VEVENT: the recurrence set of each `UID` (`DTSTART`, `RRULE`, `RDATE`, minus `EXDATE`, with its `RECURRENCE-ID` overrides) walked by ical-rs, an override standing in for the instance it names, and one moved into the window from an identity past it still listed. An override travelling without its series lists on its own.

An occurrence SHALL carry the item `id` and its `recurrenceId`, the instance identity spelled as the series' `DTSTART` (a date, a local time, or a `Z` time), which `event read -r` takes back. Occurrences SHALL list in start order. A timed occurrence overlaps when it starts before the window's end and ends after its start (a zero-length one when it starts inside); a whole-day one compares by date. A window open on its end SHALL reach a year past its start.

Without a window a series SHALL list once, at its first start.

### Requirement: Times resolve to instants
A time SHALL resolve through its `TZID` as the time-zone database names it (a writer's path prefix tolerated), else through the calendar's `VTIMEZONE` of that `TZID`, else in the local zone, as a floating time does; a `Z` time is UTC. A local time a transition skips SHALL read with the offset before the gap, one it repeats as its first occurrence (RFC 5545 3.3.5).

Expansion SHALL move a UTC `UNTIL` onto the wall clock of a zoned `DTSTART` before the civil walk, and SHALL drop the instances a calendar-defined zone skips.

### Requirement: The event projection
An event's JSON SHALL carry `id`, `uid`, `recurrenceId`, `summary`, `description`, `location`, `start` and `end` (iCalendar-spelled), `allDay`, `startsAt` and `endsAt` (RFC 3339 with offset, or `YYYY-MM-DD` with an exclusive end for a whole day), `timeZone`, `recurring`, `status`, `transparency`, `busyStatus` (Outlook's), `organizer` (`email`, `name`), `attendees` (`email`, `name`, `partstat` defaulting to `NEEDS-ACTION`, `role`, `rsvp`, `cutype`) and `onlineMeetingUrl` (`CONFERENCE`, then the vendors' properties). The end SHALL be `DTEND`, else the start plus `DURATION`, else a day for a date and the start for a time.

`event read` SHALL print the item's bytes, and its JSON SHALL add the projected `events`, or the one occurrence `-r/--recurrence-id` names.

### Requirement: An event is found by its UID
`event find <UID>` SHALL list, unexpanded, every VEVENT of the calendar carrying that `UID`, with the id the other `event` commands take.

### Requirement: pimdir queues scheduling intents
`pimdir reply <EVENT-ID> accept|tentative|decline` SHALL append one `calendar-reply` action `{ "v": 1, "source"?, "seq", "partstat": "ACCEPTED"|"TENTATIVE"|"DECLINED", "comment"? }`, and `pimdir cancel <EVENT-ID>` one `calendar-cancel` action `{ "v": 1, "source"?, "seq", "comment"? }` (pimdir STORAGE Annex B.2), anchored on the item's calendar, through the producer and its §15.6 gate, and SHALL touch no item.

`source` SHALL name the performer: `--source` when given, else the single candidate, else the recorded choice; several candidates and no choice SHALL be refused, listing them. A store whose sources declare nothing SHALL get no `source` unless one is given.

A reply to an event naming no `ORGANIZER`, or a cancellation of one naming no `ATTENDEE`, SHALL be refused when its body is local, the latter pointing at `event delete`.

## MODIFIED Requirements

### Requirement: Time-range filtering
`event list` SHALL accept `--from` and `--to` as inclusive days. The pair SHALL map onto a range whose upper bound is exclusive (the day after, at midnight UTC), so `--to` covers the whole day named.

A crossed pair SHALL be rejected by name. A range SHALL lift the default page-size cap, so a window returns every match rather than its first page.

A backend filtering locally (vdir, msgraph, pimdir) SHALL keep an item when one of its events starts in the window or one of its occurrences overlaps it. An undownloaded pimdir series the summary says recurs SHALL pass from its `DTSTART` to its `UNTIL`.

### Requirement: Protocol-specific families
`configure` (alias `wizard`) SHALL run the account wizard.

`caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the shared API has no home for.

pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, and how many creations are queued for the next sync; and `reply` and `cancel`, the scheduling intents only its sync engine can perform.

A backend MAY exist without a family where its protocol adds nothing the shared surface lacks; a family SHALL NOT push protocol-specific concepts into the shared API instead.

## REMOVED Requirements

None.
