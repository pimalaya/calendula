---
cairn: delta
change: an-unknown-key-is-refused
---

# Delta

Every requirement lives in config.md.

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Unknown fields
Every block of the configuration, the top-level and account blocks included, SHALL set `deny_unknown_fields`, so a key calendula does not know fails the load naming it, a typo reported rather than ignored. The block of a backend the build leaves out SHALL load unread, so one file serves every build. A table calendula no longer reads SHALL be refused by name: `todo`, `journal`, `table`, the top-level `account`, `calendar.list`, `event.list.table` and `item.list.table`.

#### Scenario: A typo in an account
- GIVEN `[accounts.a]` holding `defualt = true`
- WHEN the configuration is loaded
- THEN it fails naming the unknown field `defualt`

#### Scenario: A backend left out of the build
- GIVEN an account holding a `gcal` block, and a build without the `gcal` feature
- WHEN the configuration is loaded
- THEN it loads, the `gcal` block unread

### Requirement: A leftover todo or journal table is refused by name
The last sentence reads: the table is refused by name, as the family refuses its removed keys, rather than as an unknown field.

### Requirement: No colour or table is configured
The configuration SHALL hold no colour and no table shape. A `table` table (`table.preset`, `table.arrangement`), at the top level or under an account, a top-level `account` table (`account.list.table`), and the `calendar.list`, `event.list.table` and `item.list.table` tables holding the `*-color` keys, at the top level or under an account, SHALL fail the load by name, saying the output is plain text and to remove it. `event.list.page-size` and `item.list.page-size` stay.

#### Scenario: A colour key
- GIVEN a configuration written for 0.2 with `event.list.table.id-color = "red"`
- WHEN any calendula command loads it
- THEN it fails naming the line and saying the `event.list.table` table is gone

### Requirement: One sample configuration, with a section per backend
The scenario "A build without a backend" reads: the test suite parses every section and its variants, the Google Calendar block loading unread.

## REMOVED Requirements
