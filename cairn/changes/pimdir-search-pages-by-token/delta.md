---
cairn: delta
change: pimdir-search-pages-by-token
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: pimdir searches calendar items through the store's index
(commands) The output paragraph SHALL read: each hit SHALL print on one line through pimalaya-cli's `Listing`: its id, the public `seq`, which `event read` takes; its start and end as `event list` shows them (Columns follow the kind), from its summary when its body is not local; its calendar; what a text term matched (`body` or `summary`), a column present when any hit of the page says; then its summary. The header SHALL name the account (`<name> (pimdir)`), the calendar (`-k`, else `all`), the query, the page and the next one (`Next page: --page <n>`), then the `Incomplete:` and `Zones:` fields when they apply. `--sort date|name|relevance` orders the hits, `date` (latest first) by default, a series by its first occurrence, and `--page` and `--page-size` page them, the page size defaulting to `event list`'s; pimalaya-cli's `--page-token`, a page number from 1, SHALL win over `--page`, a token that is not one refused, naming it. Under `--json` the page carries `hits`, `nextPage` (the next page number as a string, `null` on the last page), `coverage` and `errors`, each hit the event projection plus `calendarId`, `matched` (`null` when only field terms matched) and `snippet`. A hit the store cannot read (its row, its body, a hit no calendar holds) SHALL be an `errors` entry, its id the `seq`, coded by the store's refusal, else `item-unreadable`, ``Read item `<seq>` ``, the other hits listed, and the command SHALL exit 3.

#### Scenario: A search resumed from its token
- GIVEN an indexed pimdir calendar holding three events naming the budget
- WHEN `calendula --json pimdir search -s 2 budget` runs, then `calendula --json pimdir search -s 2 --page-token 2 budget`
- THEN the first prints two hits and `"nextPage":"2"`, the second one hit and `"nextPage":null`, neither printing `more`
- AND `--page-token 0` or `--page-token two` fails naming the token

### Requirement: A pimdir calendar search says what it did not cover
(commands) The last sentence SHALL read: the plain output of `pimdir search` SHALL print an `Incomplete:` field above its hits, and `pimdir count` one `Incomplete:` line after its count, a blank line apart, naming what left the answer incomplete, a `bodiless`, a `missing` or an `unparseable` count only while io-pimdir counts it against the answer (`PimdirSearchReads::kinds` and, for `bodiless`, `counts_bodiless`), and nothing when the answer is complete.

### Requirement: A pimdir search names an index read in another zone
(commands) The plain output SHALL say so, naming both zones, in a `Zones:` field above the hits of `pimdir search` and in a line after the count of `pimdir count`.

### Requirement: A listing names the items it could not read
(commands) `pimdir search` SHALL join the listings printing `errors`.

## REMOVED Requirements
