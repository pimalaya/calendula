---
cairn: change
change: agenda-expands-recurring-series
---

# Delta

## ADDED Requirements

### Requirement: An agenda expands recurring series
The agenda SHALL read the events over the days its grid draws and SHALL expand a recurring series over them, as a windowed `event list` does: each occurrence marks its own day and lists at its own time, overrides applied and excluded dates dropped.

A series placed once at its first start leaves a yearly event out of every year but its first, so the agenda and a listing of the same days would disagree.

#### Scenario: A yearly event created years ago
- GIVEN a calendar holding a yearly event whose first start is in 2019
- WHEN `calendula event agenda 2027` draws that year
- THEN the day it falls on in 2027 is marked and the event lists under the grid at that day
