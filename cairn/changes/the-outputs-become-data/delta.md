---
cairn: change
change: the-outputs-become-data
---

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Every data command returns a named output type
A command answering with data SHALL hand the printer a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, named after its command path. A write with no payload of its own SHALL print a receipt rather than a sentence: a shared write `{id, queued}` (`WriteReceipt`), the id of the item acted on and whether a sync engine applies the write later, and a write in a protocol namespace `{id}` (`IdReceipt`), the id as it is after the write (a renamed collection's new one). The sentence is the text form only.

A `create`, an `update` and a `build` answer data too: an abandoned edit is an outcome they have to report, so each SHALL return an untagged enum whose write arm serializes exactly as it would on its own, and whose abandoned arm carries nothing, as an empty object. A `build` writing to `-o` answers `{path, size}`, the file written and its size in bytes, having handed the data to it.

Every runnable command SHALL be described in the `json-schema` registry, a command printing one of two types through both (`OneOf`), which a test checks over the clap command tree.

Every key of such a payload SHALL be camelCase, the spelling of the wire formats calendula sits over and the one a jq path addresses without quoting. The configuration schema stays kebab-case, being a TOML document a person writes rather than machine surface.

## REMOVED Requirements
