---
cairn: delta
change: a-page-reads-its-own-bodies
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: CalDAV backend
A paged listing SHALL enumerate the calendar's references (`calendar-query` asking `getetag` alone), cut its window over them, and fetch that window's bodies through one `calendar-multiget`; a listing taking every item SHALL keep the one `calendar-query` with `calendar-data`. A row the server answered without the item's data SHALL be named in the `errors` of the page whose window holds it, and SHALL hold its place on that page.

### Requirement: Microsoft Graph backend
A paged listing without a range SHALL cut its window over the listed events before it reads a series master again, so a page reads again the masters it shows alone.

## REMOVED Requirements
