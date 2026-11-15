---
cairn: delta
change: an-option-after-a-query-is-an-option
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: pimdir searches calendar items through the store's index
(commands) An option of `pimdir search` or `pimdir count` SHALL be read as the option wherever it stands, before the query or after it, `--json` included; a query word starting with `-`, a negated term, SHALL pass after `--` alone, a word starting with `-` before it being refused as an unknown option.

#### Scenario: An option after the query
- GIVEN a pimdir account
- WHEN `calendula pimdir search budget --page-size 3 --json` runs
- THEN it searches `budget`, three hits a page, and prints JSON
- AND `calendula pimdir search -- budget -lisbon` searches `budget -lisbon`

## REMOVED Requirements
