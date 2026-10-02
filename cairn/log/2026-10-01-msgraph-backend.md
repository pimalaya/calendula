---
cairn: log
change: msgraph-backend
landed: 2026-10-01
---

# A Microsoft Graph backend

The `msgraph` backend (src/msgraph/backend.rs) reaches Outlook and Microsoft 365 calendars over io-msgraph's new calendars and events and its `ical` projection, last in the auto order after gcal. A calendar is a Graph calendar; an item a lone event or a series master with its exceptions folded in, read from the instances of the series' own range, its `changeKey` as etag.

Graph narrows by time only through the calendar view, which expands every series, so a range filters the listing locally; the filter vdir kept private became `CalendarItem::starts_within`, shared by both. Graph keeps no calendar description nor RGB colour, so naming either is refused, and only the series master is written back.

No protocol-specific `msgraph` command yet. io-msgraph and ical-rs are unreleased, so Cargo.toml patches them to their local checkouts.

Capability moved: **backends** (a Microsoft Graph backend; the selection order, modified).
