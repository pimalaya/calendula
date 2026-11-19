---
cairn: change
change: the-releases-of-2026-11-19-land
---

# Delta

## MODIFIED Requirements

(backends.md, pimdir backend) A collection SHALL be listed as a calendar when its kind's media type is `text/calendar`; one declaring no kind belongs to no application's kind and SHALL NOT be listed nor counted toward the account read. A `pimdir.account` on a store holding no collection yet SHALL be taken as configured. Scenarios added: a collection of no kind is no calendar; a configured account on an empty store.

(commands.md, pimdir search) The scope is io-pimdir's `PimdirQuery::scoped`, and the `Incomplete:` counts io-pimdir's `PimdirSearchCoverage::unread_counted`.

(commands.md, the code table and the request output) `request-origin` added: `gcal request` and `msgraph request` refuse a full URL off the API's origin coded `request-origin`, exit 1, in pimalaya-cli's message.

(projection.md, jmap) An `RRULE`'s `UNTIL` SHALL be told between UTC and the series' zone both ways (ical-rs 0.7.2, `tzdb`); a UTC `UNTIL` in a fold's second pass comes back as the first pass, and one of a zone neither the calendar nor the database knows travels unshifted.
