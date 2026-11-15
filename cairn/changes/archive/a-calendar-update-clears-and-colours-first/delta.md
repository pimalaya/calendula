---
cairn: delta
change: a-calendar-update-clears-and-colours-first
---

# Delta

## ADDED Requirements

### Requirement: A calendar update clears what it empties
(commands) `calendar update -d ""` and `-C ""` SHALL clear the description and the colour on every backend serving them, never report success leaving them: a CalDAV `DAV:remove` of the property, an empty description sent to Google, which stores it as none, vdir clearing its own. A Google calendar always carries a colour, so clearing one stays refused by name.

#### Scenario: A description cleared on CalDAV
- GIVEN a CalDAV calendar `cal` described `Old`
- WHEN `calendula calendar update cal -d ""` runs
- THEN one PROPPATCH removes `C:calendar-description`, and the next `calendar list --json` reads its `description` `null`

## MODIFIED Requirements

### Requirement: CalDAV backend
(backends) Added: `update_calendar` SHALL write only the properties the patch names, setting a given one and removing (`DAV:remove`) a cleared one, a clear sent over the client's stream as io-webdav's own PROPPATCH, read alike: a property the server refuses, or answers nothing of (RFC 4918 §9.2.1), fails the update. A colour SHALL go first, in a PROPPATCH of its own, so a server keeping the rest of a PROPPATCH while it ignores a colour (iCloud) leaves the calendar as it was: the update fails ``Write the colour of calendar `<id>`, nothing else written``. `create_calendar` given a colour SHALL read the calendars back and log a warning naming the colour when the server kept none.

#### Scenario: A colour iCloud ignores
- GIVEN a CalDAV server ignoring `calendar-color` in a PROPPATCH
- WHEN `calendula calendar update cal -n New -d Text -C '#123456'` runs
- THEN it exits 1 naming `calendar-color`, after one PROPPATCH holding the colour alone, and the name and the description are not written

### Requirement: Google Calendar backend
(backends) Added: `update_calendar` SHALL send a cleared description as an empty string, a field a PATCH leaves out staying as it is, and an empty description SHALL read as none.

## REMOVED Requirements
