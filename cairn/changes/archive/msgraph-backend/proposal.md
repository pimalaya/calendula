---
cairn: change
id: msgraph-backend
status: landed
created: 2026-10-01
---

# A Microsoft Graph backend

## Why

Outlook and Microsoft 365 calendars had no backend: io-msgraph had no calendar API. It now has calendars, events and an `ical` projection, so calendula reaches them the way it reaches Google Calendar through io-gcal.

## What

An `msgraph` backend in the default set: calendars, items as lone events or series masters with their exceptions, a range filtered locally, writes through the projection.

## Not in scope

A protocol-specific `msgraph` command family; pushing an exception edited locally.
