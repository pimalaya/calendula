---
cairn: delta
change: the-x14-dependency-pass
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Event list text
(commands) `Next page:` SHALL print after the events and one blank line, the header values two spaces past the longest label (pimalaya-cli 0.6.0, D27); `pimdir search` likewise.

### Requirement: Google Calendar backend
(backends) A series SHALL be written through `GcalEventSeries`, each override onto its listed instance; a zoned boundary projects in the zone Google names; `calendar.create.id` is `none`.

### Requirement: Microsoft Graph backend
(backends) A series SHALL be written through `MsgraphEventSeries`, each cancelled or overridden occurrence through its listed instance; `calendar.create.id` is `none`.

### Requirement: pimdir queue
(backends) `queue show` SHALL print `refusals`; `pimdir status` SHALL print `refused`.

### Requirement: Capabilities
(commands) `calendar.create.id` joins the registry, reached by `calendar create --id`.

### Requirement: Raw requests
(commands) `gcal request` and `msgraph request` SHALL refuse a full URL off the API's origin before sending.

## REMOVED Requirements
