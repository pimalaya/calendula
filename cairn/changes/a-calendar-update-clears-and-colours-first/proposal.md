---
cairn: change
id: a-calendar-update-clears-and-colours-first
status: landed
created: 2026-11-15
---

# A calendar update clears, and writes its colour first

> Line K21 and the iCloud colour item (I-F3) of line K24 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), from the live testing campaign of 2026-11-15 (its reports/calendula/caldav-icloud.md I-F2 and I-F3, gcal-google.md G-F6).

## Why

`calendar update -d ""` exited 0 on iCloud and Google and the description stayed: the CalDAV adapter set the description to `None`, which io-webdav 0.9's set-only calendar update leaves out of the PROPPATCH, and the gcal adapter built its PATCH with `description: patch.description.flatten()`, a field a PATCH leaves out staying as it is. vdir cleared it. Clearing a CalDAV colour (`-C ""`) wrote nothing either.

iCloud keeps no `calendar-color` in the namespace io-webdav writes: `calendar create --color` exited 0 with the colour dropped, and `calendar update -n -d --color` exited 1 (`WebDAV server ignored the property update: calendar-color`) after the name and the description had changed, the server applying the rest of one PROPPATCH.

## What

- CalDAV `update_calendar` writes only what the patch names, where it read the calendar and wrote every property back: a cleared description or colour is a `DAV:remove`, pumped as io-webdav's `WebdavProppatch` over the client's stream (src/caldav/client.rs `proppatch`, read as io-webdav reads its own: a property refused or not answered fails it), a set alone goes through io-webdav's `update_calendar`.
- The colour is written first, alone: refused or ignored, the update fails ``Write the colour of calendar `<id>`, nothing else written`` before the name and the description are sent.
- CalDAV `create_calendar` given a colour reads the calendars back and warns when the server kept none.
- gcal `update_calendar` sends a cleared description as `""`, which Google stores as none, and an empty description reads as none.
- Spec: backends ("CalDAV backend", "Google Calendar backend", modified), commands ("A calendar update clears what it empties", added).
