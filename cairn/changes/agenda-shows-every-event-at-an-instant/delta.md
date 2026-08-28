---
cairn: change
change: agenda-shows-every-event-at-an-instant
---

# Delta

## ADDED Requirements

### Requirement: An agenda shows every event at an instant
The agenda SHALL render every event whose start falls at a given instant, never one of them, and SHALL order two events at one instant totally and stably (by summary, then by public id) so the same calendar renders the same way twice.

Two events may legitimately start at the same moment: two unrelated meetings at 09:00, and, since a collection may hold two resources under one `UID` (pimdir SPEC §9), two copies of what a server considers one identity. A view keyed by the instant alone shows the last one written and reports nothing, which loses exactly the event the store went to the trouble of keeping.

The `--output json` payload SHALL carry the same multiplicity, so a consumer reading it is not told there was one event where there were two.

#### Scenario: Two events at one instant
- GIVEN a calendar holding two events starting at the same instant
- WHEN the agenda is rendered, in text and in JSON
- THEN both appear, in the stated order

## MODIFIED Requirements

## REMOVED Requirements
