---
cairn: delta
change: the-calendar-is-an-argument
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Calendar selection
(commands) A command acting on the calendar itself SHALL take it as an argument, as Himalaya's `mailbox delete <MAILBOX>` does, the former `-k/--calendar` kept as a hidden alias: `calendar update [CALENDAR]` resolves it as above, and `calendar delete <CALENDAR>` SHALL require it, never falling back to a default. `calendar create [ID] <NAME>` SHALL take the display name, and the id before it on the backends letting a client name a calendar: given none, vdir SHALL name the directory after the name, refused unless it is one directory name, and CalDAV SHALL mint a version 4 UUID for the URL segment, a display name holding what a path cannot; Google and Graph mint their own either way. `calendar create` and `calendar update` SHALL take `-d/--description` and `-C/--color`, as cardamum's `addressbook` commands do.

#### Scenario: A calendar named alone
- GIVEN a vdir account whose home holds no calendar
- WHEN `calendula --json calendar create Work`, then `calendar update Work -n Job -C '#ff0000'`, then `calendar delete Work` run
- THEN the first prints `{"id":"Work","name":"Work","queued":false}` and makes the directory `Work`, the second renames and paints it, the third removes it
- AND `calendar delete -k Work` and `calendar create work Work` are read as before

### Requirement: Protocol-specific families
(commands) `caldav create` and `vdir create` SHALL take the calendar's metadata with the letters `calendar create` and `calendar update` use: `-n/--display-name`, `-d/--description`, `-C/--color`, the former `-D` read as `-d`.

### Requirement: vdir refuses an unknown calendar
(backends) A new id that is not one directory name SHALL be refused before anything is written: `calendar create` with ``Calendar id `<id>` is not one directory name``, or, given a name and no id, ``Calendar name `<name>` is not one directory name: give the calendar an id before its name``.

## REMOVED Requirements
