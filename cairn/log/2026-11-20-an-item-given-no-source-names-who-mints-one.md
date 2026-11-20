---
cairn: log
change: an-item-given-no-source-names-who-mints-one
landed: 2026-11-20
---

# An item given no source names who mints one

Landed without a proposal: bug C-A of the pimdir backend tests of 2026-11-20.

`item add` and `item build` given no source bailed "Nothing to create; give an iCalendar, or -i to compose one" (and "Nothing to build; ..."), while `-i` with no source refused in turn, `item` minting nothing. Both now fall through to `blank_item`, whose refusal names `calendula event` for an event, `unfog` for a to-do and `journal` for a journal entry, as commands.md already required. `-i` stays on `item`: it opens the given source in the composer. `event build` keeps its "Nothing to build" refusal.

Test `an_item_given_no_source_names_who_mints_one` (tests/write.rs), which failed on the old message.

Moved: backends.md (a body composed by `item create -i` from the blank is no case: `item` composes from a source, queued as given, or from the stored item under `item update -i`).
