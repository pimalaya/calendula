---
cairn: change
id: an-event-owes-a-dtstart
status: landed
created: 2026-09-01
---

# A composed event with no DTSTART reached the server, which answered 500

`event create -i` against a Posteo (SabreDAV) calendar failed with `WebDAV server returned HTTP 500` on a VEVENT the composer wrote with a `SUMMARY` and no `DTSTART`. The check that is supposed to catch exactly this let it through.

## Why

RFC 5545 3.6.1 makes `DTSTART` required of a VEVENT unless the calendar specifies a `METHOD`. That is a condition on the enclosing calendar, not a property list, so ical-rs's per-component contract (`required_props` = `UID`, `DTSTAMP`) cannot state it and does not.

Leaving it uncaught is worse than an ordinary gap: SabreDAV denormalizes `DTSTART` into its index on write and answers HTTP 500 rather than refusing the request, so the failure names nothing.

## What

`check` gains the conditional rule alongside ical-rs's own, and `blank_item` mints a `DTSTART` of now on a VEVENT so the seed passes the check it will be held to. A VTODO and a VJOURNAL owe none and get none.
