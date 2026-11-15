---
cairn: change
id: the-codes-follow-the-family
status: landed
created: 2026-11-15
---

# The codes follow the family

> Lines K13 and K16 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), findings F33 and F34; cardamum's C11 and C15 are the twins.

## Why

- A command naming no calendar, on an account with no `calendar.default`, failed with `code: null` ("Missing calendar id; pass -k/--calendar or set calendar.default"), where pimalaya-config codes the same miss for an account (`account-default-missing`).
- calendula relayed 18 of the pimdir codes Himalaya relays: a store a sync owns, one between two chunks, one of another sync version, a write a source does not support, an intent no source or several can perform, among others, failed uncoded, so a script reading every pimdir app met a code in Himalaya and `null` in calendula for the same store state.

## What

- `calendar-default-missing`: ``No default calendar found, name one with `-k <CALENDAR-ID>` or set `calendar.default` ``, pimalaya-config's sentence for an account, from the one resolver every unified and pimdir command goes through.
- The pimdir codes relayed are Himalaya's table, by io-pimdir's spelling: `store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `conflict-unread`, `unsupported`, `no-performer` and `ambiguous-performer` join the vocabulary. `tag-refused` stays out: only a mail tag raises it.
- The store's refusals calendula words itself carry their code too: an online meeting, a scheduled event or a to-do a source does not support `unsupported`; an intent no source syncs `no-performer`, several `ambiguous-performer`, as Himalaya codes them; reading the sources' capabilities goes through the relay.
- Spec: commands (the code table, "Calendar selection"), backends ("A pimdir store failure carries io-pimdir's code").
