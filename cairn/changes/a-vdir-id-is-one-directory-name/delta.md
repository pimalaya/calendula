---
cairn: delta
change: a-vdir-id-is-one-directory-name
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: vdir refuses an unknown calendar
(backends) A vdir calendar id SHALL name a calendar only when it is one directory name, one path segment that is neither `.` nor `..` and holds no separator (a trailing one included), naming a directory under the home. Every vdir command naming another SHALL fail before anything is read or written, coded `calendar-not-found`: `calendar update`, `calendar delete` and every `event` and `item` read and write with ``Calendar `<id>` not found``, `vdir delete` and `vdir rename` with ``Collection `<id>` not found``. A new id that is not one directory name SHALL be refused before anything is written: `calendar create` with ``Calendar id `<id>` is not one directory name`` and `vdir create` with ``Collection id `<id>` is not one directory name``, uncoded, the new name of `vdir rename` by io-vdir.

A write creates whatever directory it is handed, a listing of a missing one answers nothing and a delete removes whatever the path reaches, so without it a typo in `-k` would read as an empty calendar or invent one, an id reaching out of the home (`..`, `../keep`, `personal/..`) would remove a directory there and a create would write there.

#### Scenario: A typo in `-k`
- GIVEN the account `test` whose vdir home holds the calendar `personal` alone
- WHEN `calendula --json event list -k nope`, `event create -k nope` or `calendar delete -k ..` runs
- THEN each exits 1 with the failure object coded `calendar-not-found`, and the home and the directory holding it are left as they were

#### Scenario: A new id reaching out of the home
- GIVEN the account `test` whose vdir home `outer/home` holds the calendar `personal` alone, beside the directory `outer/keep`
- WHEN `calendula --json calendar create .. Escape`, `calendar create ../keep Escape` or `vdir create x/y` runs
- THEN each exits 1 with the failure object, its `code` null, and nothing is written in the home, `outer/keep` or `outer`

## REMOVED Requirements
