---
cairn: delta
change: the-vdir-item-is-added
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: vdir items
(commands) `create <COLLECTION> [--kind ical|vcard] <INPUT>` becomes `add <COLLECTION> [--kind ical|vcard] <INPUT>` (visible alias `new`, `create` a hidden one), the rest unchanged.

#### Scenario: A raw item is added
- WHEN `calendula vdir item add cal --kind ical -` runs
- THEN it stores the item read from stdin in `cal` and prints `{id}`
- AND `vdir item create` and `vdir item new` run the same command, and `calendula vdir item --help` lists `add` (alias `new`) and not `create`

## REMOVED Requirements
