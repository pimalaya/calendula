---
cairn: change
id: a-restamped-event-is-no-conflict
status: landed
created: 2026-11-18
---

# A restamped event is no conflict

## Why

On Microsoft Graph, `event update` right after `event add` failed `precondition-failed` in about one run out of four (live test `graph_event_alarm_round_trip`). Probed live on 2026-11-18: Exchange moves an event's `changeKey` once, about a second after its create, and never back, the body changing only in `DTSTAMP` and `LAST-MODIFIED`. The key `event update` read no longer matched the one `update_item` read again, so calendula refused its own write. A plain re-read cannot help, the moved key being permanent.

## What

`CalendarClient::update_item` takes `read`, the body the etag was read with when calendula's own read gave it (`event update`, `item update -i`, no `--if-match`), passed to msgraph alone. msgraph's `update_item` reads the event and its base first, then refuses a moved `changeKey` unless `read` is given and differs from the base read again only in `DTSTAMP` and `LAST-MODIFIED` lines, unfolded. A `--if-match` the caller gave keeps the strict comparison. Owner ruling (option (a)) relayed by pimalaya-95 on 2026-11-18.
