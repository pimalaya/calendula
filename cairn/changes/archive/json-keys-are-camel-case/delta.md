---
cairn: change
change: json-keys-are-camel-case
---

# Delta

## ADDED Requirements

(none)

## MODIFIED Requirements

### Requirement: Every data command returns a named output type
A command answering with data SHALL hand the printer a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, named after its command path. `Message` carries a confirmation and never data.

Every key of such a payload SHALL be camelCase, the spelling of the wire formats calendula sits over and the one a jq path addresses without quoting. The configuration schema stays kebab-case, being a TOML document a person writes rather than machine surface.

## REMOVED Requirements

(none)
