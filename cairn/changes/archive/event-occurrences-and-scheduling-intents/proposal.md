---
cairn: change
id: event-occurrences-and-scheduling-intents
status: landed
created: 2026-10-03
---

# Event occurrences, and replies and cancellations through pimdir

A client driving calendula over a pimdir store (MOA, which replaces its direct Microsoft Graph and Google calendar calls with calendula on a store neverest syncs) needs what a calendar view gives: the events of a window with every recurring series expanded into its occurrences, each at its own instants with their UTC offset, an occurrence addressable on its own, the people an event names and its meeting link. It also needs to answer an invitation and to cancel an event it organises, which a store cannot do by itself.

calendula listed a series once, at its first `DTSTART`, verbatim and zone-less, dropped from a window it recurs in, with no attendee, organizer or location in its JSON. A pimdir window also read the master's start alone, so a series begun before the window never reached it.

## What changes

- `event list --from/--to` expands: each occurrence overlapping the window lists on its own, overrides applied and excluded dates dropped, carrying the item `id` and a `recurrenceId`, the instance identity spelled as the series' `DTSTART`. Expansion is ical-rs's `IcalRecurSet`; zones resolve through the time-zone database, the calendar's `VTIMEZONE`, or the local zone for a floating time, and a UTC `UNTIL` is moved onto the start's wall clock before the civil walk.
- An event's JSON carries `uid`, `recurrenceId`, `description`, `location`, `allDay`, `startsAt` and `endsAt` (RFC 3339 with offset, or a date), `timeZone`, `recurring`, `status`, `transparency`, `busyStatus`, `organizer`, `attendees` (with `partstat`, `role`, `rsvp`, `cutype`) and `onlineMeetingUrl`.
- `event read` takes `-r/--recurrence-id` and its JSON adds the projected `events`.
- `event find <UID>` lists the events carrying a `UID`.
- The local range filters (vdir, msgraph, pimdir) keep an item when one of its occurrences overlaps the window; an undownloaded pimdir series passes from its start to its `UNTIL`.
- `pimdir reply <ID> accept|tentative|decline [-m TEXT] [--source S]` and `pimdir cancel <ID> [-m TEXT] [--source S]` queue the `calendar-reply` and `calendar-cancel` intents (pimdir STORAGE Annex B.2), naming their performer.

## What does not change

Without a window a series lists once. Bytes are never rewritten. The intents touch no item: the performer sends the message and the sync brings its effect back.
