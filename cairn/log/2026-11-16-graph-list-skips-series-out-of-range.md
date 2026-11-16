---
cairn: log
change: graph-list-skips-series-out-of-range
landed: 2026-11-16
---

# Graph listing skips series out of the range

A Graph `event list` over a range re-read every series master the calendar ever held (`GET /me/events/{id}`, for its `cancelledOccurrences`) before checking whether the series reached the range, so its cost grew with the calendar's history: a one-day listing on a Microsoft 365 calendar holding 135 series took about 56 s, 117 of its 155 requests spent on series ended before the day ([#32](https://github.com/pimalaya/calendula/issues/32)). Capability **backends** moved.

- **src/msgraph/backend.rs**: `list_items` drops, before `reread`, a series master whose recurrence `bounds()` (from the listing) end before the range or start on or after it, the range a day wider either side as `read_within` reads it (`range_days`, split out of it). A side the range leaves open, no range at all, or bounds that do not read skip nothing, so `item list` and an open-ended `event list` still read every series whole. A numbered range's end is io-msgraph's bound past its last occurrence, never short. The dropped series could list no occurrence: the re-read only adds EXDATEs, and its instances were already read only within its bounds. The reporter's listing goes from 155 requests to 38.
- **Evidence**: `a_series_out_of_the_range_is_not_read_again` lists a series ended in 2024, one starting in 2030 and a current one over 2026-11-16, and checks three requests are sent (the listing, the current series' re-read and its instances); with the skip disabled it fails on the extra re-reads.
- **Left**: the remaining re-reads and first `instances` pages could go through Graph's JSON `$batch` (io-msgraph `client.batch`, 20 requests a call), about 4 requests for the reporter's week; not started. `$select=cancelledOccurrences` on the listing is no way out, Graph returns it only on a read by id.

CHANGELOG (Fixed). Spec: backends ("Microsoft Graph backend", scenario "A calendar holding years of ended series").
