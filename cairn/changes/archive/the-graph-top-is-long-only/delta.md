---
cairn: delta
change: the-graph-top-is-long-only
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: msgraph family
(commands) Appended: `calendars list` and `events list` SHALL take `--top` long-only, as Himalaya's Graph listings do, the former `-s` read as `--top` and named by no help, manual or completion.

#### Scenario: The Graph listings take `--top` long-only
- WHEN `calendula msgraph events list --help` prints
- THEN it lists `--top <N>` with no short letter
- AND `calendula msgraph events list -s 5` reads as `--top 5`

## REMOVED Requirements
