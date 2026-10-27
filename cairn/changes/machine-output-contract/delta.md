---
cairn: delta
change: machine-output-contract
---

## ADDED Requirements

### Requirement: The JSON contract is described and versioned
`json-schema` SHALL describe the failure object under the key `calendula-error`, its `code` typed by the code vocabulary. Every generated schema SHALL carry an `$id` naming the contract version: the major version of calendula, or `0.<minor>` before 1.0. Every registered payload SHALL be a JSON object, checked by a test. Piped `--json` output SHALL print one value per line.

## MODIFIED Requirements

### Requirement: Every data command returns a named output type
A command answering with data SHALL hand the printer a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, named after its command path. `Message` carries a confirmation and never data.

A `create`, an `update` and a `build` answer data too: an abandoned edit is an outcome they have to report, so each SHALL return an untagged enum whose write arm serializes exactly as it would on its own, and whose abandoned arm carries nothing, as an empty object. A `build` writing to `-o` answers a `Message`, having handed the data to a file.

Every key of such a payload SHALL be camelCase, the spelling of the wire formats calendula sits over and the one a jq path addresses without quoting. The configuration schema stays kebab-case, being a TOML document a person writes rather than machine surface.
