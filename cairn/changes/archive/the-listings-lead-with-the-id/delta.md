---
cairn: delta
change: the-listings-lead-with-the-id
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Text output is plain text
(commands) `item list` SHALL print the same header above one line per item: its id first, as every listing of the family, then its size and its entity tag. `calendar list` SHALL print as Himalaya's `mailbox list` does: `Account:` alone above one line per calendar, its id, `default` for the one the server names so, its color, then its name, last; its description prints under `--json` alone. `calendar list` leaves the lists printed as records.

#### Scenario: A calendar list
- GIVEN the vdir account `perso` holding the calendars `personal`, the default, colored and described, and `work`
- WHEN `calendula calendar list` prints them
- THEN it prints `Account:    perso (vdir)`, a blank line, then `personal  default  #ff0000  Personal` and `work      -        -        Work`, aligned, the description left out

## REMOVED Requirements
