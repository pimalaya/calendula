---
cairn: change
change: zone-lookup-fixes
---

# Delta

## MODIFIED Requirements

### Requirement: Times resolve to instants
A time SHALL resolve through its `TZID` as the time-zone database names it (a writer's path prefix or a leading slash tolerated), else through the calendar's `VTIMEZONE` of that `TZID` when it states at least one observance, else in the local zone, as a floating time does; a `Z` time is UTC. A local time a transition skips SHALL read with the offset before the gap, one it repeats as its first occurrence (RFC 5545 3.3.5).
