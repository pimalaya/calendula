---
cairn: log
change: gcal-anchors-floating-times
landed: 2026-10-16
---

# A backend without floating times anchors the time flags in the local zone

`EventFieldsArgs::anchor_floating` in `src/shared/event/fields.rs` sets `--time-zone` to the local zone, as jiff resolves it (`TZ`, then the system's), when the account is served by gcal or msgraph, a time flag is given and no zone flag is. `event create` and `event update` call it next to `check_backend`, so the floating time becomes a `TZID` time with its `VTIMEZONE` through the path `--time-zone` already takes, and only the times naming no zone are affected.

Tested on both backends and on the four that keep a floating time, with the zone flag, a date, a `Z` time, no time flag and a zone without an IANA name; the local zone checked to resolve to `Europe/Madrid` from the system and to `America/New_York` from `TZ`.

Capabilities moved: **commands** (a backend without floating times anchors the time flags locally, added).
