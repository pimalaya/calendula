---
cairn: log
change: a-restamped-event-is-no-conflict
landed: 2026-11-18
---

# A restamped event is no conflict

On Microsoft Graph, `event update` and `item update -i` no longer refuse their own write when Exchange restamped the event between calendula's read and the write (cairn/changes/archive/a-restamped-event-is-no-conflict). A live probe on the test account showed the `changeKey` moving once about a second after a create, in 2 creates of 3, and never back, the body changing only in `DTSTAMP` and `LAST-MODIFIED`.

`CalendarClient::update_item` takes `read`, the body behind an etag calendula read itself, given to msgraph alone; msgraph's `update_item` reads the base before the check and lets a moved key through when `restamped_only(read, base)`. A caller's `--if-match` keeps the strict comparison, as does any other difference in the body.

Test `a_restamped_event_is_no_conflict_against_its_own_read`: a restamp-only answer sends the PATCH, a renamed one and a restamp under a `--if-match` refuse `precondition-failed` with no PATCH; it fails with `restamped_only` stubbed to `false`. Live: `graph_event_alarm_round_trip` passed 10 runs of 10, then the Microsoft suite 6 of 6.

Moved: backends.md (msgraph).
