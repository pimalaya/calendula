---
cairn: delta
change: pimdir-search-reads-local-days
---

## MODIFIED Requirements

### Requirement: pimdir searches calendar items through the store's index
`calendula pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (§8) from the store's search index, scoped to one component kind of the account: the query is read as its conjuncts plus `kind:event`, `kind:task` or `kind:journal` as `--kind event|todo|journal` names it (`event` by default), `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar, each a top-level conjunct. Without `-k` it SHALL search every calendar of the account, `calendar.default` not applied; a `-k` naming no calendar of the account SHALL be refused, naming those it holds. The words of the query are joined by spaces into one query. Calendula SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

A `date:` term SHALL compare the occurrences the index expanded (SEARCH §7), its days, `today` and `yesterday` read in the zone the tables read times in (`TZ`, then the system's).

Each hit SHALL render with its kind's columns (Columns follow the kind), from its summary when its body is not local, then its calendar, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`; its `id` is the public `seq`, which the family's `read` takes. `--sort date|name|relevance` orders the hits, `date` (latest first) by default, a series by its first occurrence, and `--page` and `--page-size` page them, the page size defaulting to the kind's `list` one. Under `--json` the page carries `hits`, `more` (whether hits follow it) and `coverage`, each hit the kind's projection plus `calendarId`, `matched` and `snippet`.

#### Scenario: A series found by a later occurrence
- GIVEN an indexed pimdir calendar holding a weekly event begun on 2019-01-07, a Monday
- WHEN `calendula pimdir search date:2026-11-09` runs
- THEN the event is listed

#### Scenario: A day is the local day
- GIVEN `TZ=Europe/Paris` and an event at 00:30 on 2026-11-07, its `DTSTART` in `Europe/Paris`
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the event is listed, and `date:2026-11-06` does not list it

#### Scenario: An all-day event stays on its day
- GIVEN `TZ=Europe/Paris`, an index reading floating times in `Europe/Paris`, and an all-day event on 2026-11-07
- WHEN `calendula pimdir search date:2026-11-08` runs
- THEN the event is not listed

#### Scenario: A queued event is not listed
- GIVEN an event created in a pimdir calendar and still in the queue
- WHEN `calendula pimdir search` runs a query its body matches
- THEN it is not listed until the owner applies the create and refreshes the index

## ADDED Requirements

### Requirement: A pimdir search names an index read in another zone
When the index reads floating times and dates in another zone than the one `pimdir search` reads days in (SEARCH §7, `floating_zone`), the plain output SHALL say so under its table, naming both zones.

#### Scenario: An index read in another zone
- GIVEN `TZ=Europe/Paris` and an index reading floating times in `UTC`
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the plain output says the index reads floating times in UTC and this search reads days in Europe/Paris
