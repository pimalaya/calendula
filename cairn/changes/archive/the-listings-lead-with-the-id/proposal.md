---
cairn: change
id: the-listings-lead-with-the-id
status: landed
created: 2026-11-15
---

# The listings lead with the id

> Line K15 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F18: three collection list layouts, and `item list` printing its id last.

## Why

Himalaya's `mailbox list` prints `Account:` above one headerless line per mailbox, its id first and its name last; calendula's `calendar list` printed records. `item list` printed the entity tag, the size, then the id, where every other listing of the family leads with the id.

## What

- `calendar list` prints through pimalaya-cli's `Listing`: `Account:` (the account and its backend), a blank line, then one line per calendar, its id, `default` or `-`, its color or `-`, then its name. The description prints under `--json` alone.
- `item list` prints the id, the size, then the entity tag.
- The JSON is unchanged. Text only, so not breaking for a script reading `--json`; MIGRATION.md notes the text.
- Spec: commands ("Text output is plain text", its calendar list scenario).
