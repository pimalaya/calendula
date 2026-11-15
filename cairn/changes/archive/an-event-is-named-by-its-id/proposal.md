---
cairn: change
id: an-event-is-named-by-its-id
status: landed
created: 2026-11-15
---

# An event is named by its id

> Line K15 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F18: a UID taken as an id by every verb or by none.

## Why

`event delete` took an iCalendar `UID` where its id read nothing (event-delete-by-id-or-uid, 2026-10-04), while `event read` and `event update` took the id alone, so one argument meant two things across three verbs. The line asks for every verb when a backend resolves a `UID` cheaply, none otherwise. None does but pimdir: CalDAV, gcal, msgraph and vdir resolve one by listing and parsing the whole calendar, which is what `event search` does. cardamum's card verbs take the backend's id alone too.

## What

- None: `event read`, `event update` and `event delete` take the id `event list` shows. `event delete` still reads the id first, so a miss is `item-not-found` on every backend, its message naming `event search <UID>`, and it lists nothing more.
- Breaking for a script deleting by `UID`: MIGRATION.md gives the two steps.
- Spec: commands ("An event is deleted by its id or its UID" becomes "An event is named by its id alone").
