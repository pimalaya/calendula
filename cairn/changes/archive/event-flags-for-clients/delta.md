---
cairn: change
change: event-flags-for-clients
---

# Delta

## ADDED Requirements

### Requirement: An empty text flag removes its property
An empty value given to `--summary`, `--description`, `--location`, `--url`, `--categories`, `--organizer` or `--attendee` SHALL remove that property from the VEVENT rather than write it empty.

### Requirement: An update revises DTSTAMP
`event update` SHALL set `DTSTAMP` to the time of the write, in UTC, and `LAST-MODIFIED` to the same instant when the event carries one.

### Requirement: An online meeting can be asked for
`--online-meeting` SHALL write `X-PIMDIR-ONLINE-MEETING:TRUE` on the VEVENT (pimdir STORAGE Annex B.1). On pimdir the write SHALL be refused before it is queued when no source declares `calendar.online-meeting`; on any other backend the flag SHALL be refused by name.
