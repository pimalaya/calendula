---
cairn: delta
change: the-x13-low-fixes
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: A calendar created twice is a duplicate
(backends) The failure SHALL read ``Cannot create calendar `<calendar>`: it exists already`` on every path, `<calendar>` the id on vdir and CalDAV and the name on Graph, `vdir create` included, a server's answer in `sources`. A vdir calendar created with an id takes `--id`.

### Requirement: A missing item reads one sentence
(commands) The backend's own answer SHALL be the sentence's one source, never a ``Read item ...`` line beside it. A vdir `item update` or `event update` given an iCalendar for an id the calendar does not hold SHALL fail so, writing nothing.

### Requirement: gcal family
(commands) `calendars`, `acl list`, `events instances` and `settings` SHALL print under `--json` every item of every page as Google sent it, under `calendars`, `rules`, `instances` and `settings`. `request` SHALL send any Calendar API request.

### Requirement: Calendar selection
(commands) `calendar create <NAME> [--id <ID>]`; the former id before the name is refused.

### Requirement: pimdir queue
(backends) `attempts` SHALL be an unsigned integer.

## REMOVED Requirements
