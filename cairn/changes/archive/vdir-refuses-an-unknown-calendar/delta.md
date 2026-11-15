---
cairn: delta
change: vdir-refuses-an-unknown-calendar
---

# Delta

## ADDED Requirements

### Requirement: vdir refuses an unknown calendar
(backends) A vdir calendar id SHALL name a calendar only when it is one path segment, neither `.` nor `..`, naming a directory under the home. Every vdir command naming another SHALL fail before anything is read or written, coded `calendar-not-found`: `calendar update`, `calendar delete` and every `event` and `item` read and write with ``Calendar `<id>` not found``, `vdir delete` and `vdir rename` with ``Collection `<id>` not found``.

A write creates whatever directory it is handed, a listing of a missing one answers nothing and a delete removes whatever the path reaches, so without it a typo in `-k` would read as an empty calendar or invent one, and an id reaching out of the home would remove a directory there.

#### Scenario: A typo in `-k`
- GIVEN the account `test` whose vdir home holds the calendar `personal` alone
- WHEN `calendula --json event list -k nope`, `event create -k nope` or `calendar delete -k ..` runs
- THEN each exits 1 with the failure object coded `calendar-not-found`, and the home and the directory holding it are left as they were

## MODIFIED Requirements

## REMOVED Requirements
