---
cairn: log
change: the-listings-lead-with-the-id
landed: 2026-11-15
---

# The listings lead with the id

`calendar list` prints Himalaya's listing frame and `item list` its id first, line K15 of the CLI alignment round. Capability **commands** moved.

- **src/shared/calendar/list.rs**: `Account:`, then `<id>  default|-  <color>|-  <name>` per calendar, through pimalaya-cli's `Listing`; the records, the `Name:` and `Color:` fields and the indented description go from the text.
- **src/shared/item/list.rs**: `<id>  <size>  <etag>`, where it was `<etag>  <size>  <id>`.
- **Evidence**: both unit tests read the old layouts on the code before. The JSON is unchanged.

CHANGELOG (Changed), MIGRATION.md (text). Spec: commands ("Text output is plain text").
