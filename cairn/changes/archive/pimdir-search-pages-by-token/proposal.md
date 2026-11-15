---
cairn: change
id: pimdir-search-pages-by-token
status: landed
created: 2026-11-15
---

# pimdir search pages by token

> Line K9 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F12, owner ruling D9; the shape Himalaya defined in c7016650 (its log pimdir-search-pages-by-token), cardamum's C9 the twin.

## Why

`pimdir search` paged by `more`, a boolean, where every other listing of calendula and of the family names its next page in `nextPage` and takes it back through `--page-token`; a hit the store could not read failed the whole search; and its hits printed as records, where the family's searches print one line each through pimalaya-cli's `Listing`.

## What

- `--page-token` (pimalaya-cli `PageTokenArg`), a page number from 1, winning over `--page`; a token that is not one fails ``Invalid page token `<token>` ``. `-p/--page` and `-s/--page-size` stay.
- JSON `{hits, nextPage, coverage, errors}`: `nextPage` the next page number as a string, `null` on the last page, `more` gone.
- A hit the store cannot read (its row, its body) is an `errors` entry `{id, code, error, sources}`, its id the `seq`, coded by the store's refusal else `item-unreadable`, ``Read item `<seq>` ``; the other hits list and the command exits 3. A hit no calendar holds is one too, where it was dropped.
- Text through `Listing`: `Account:` (the account, `pimdir`), `Calendar:` (`-k`, else `all`), `Query:`, `Page:`, `Next page:`, then `Incomplete:` and `Zones:` when they apply, a blank line, one line per hit (id, start, end, calendar, what matched when any hit says, summary), then the `Error:` lines. The snippet prints under `--json` alone. `pimdir count` keeps its lines.
- Spec: commands ("pimdir searches calendar items through the store's index", "A pimdir calendar search says what it did not cover", "A pimdir search names an index read in another zone", "A listing names the items it could not read").
