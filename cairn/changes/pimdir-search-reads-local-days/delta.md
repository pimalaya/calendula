---
cairn: delta
change: pimdir-search-reads-local-days
---

## MODIFIED Requirements

### Requirement: pimdir searches calendar items through the store's index
`calendula pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (§8) from the store's search index, scoped to one component kind of the account: the query is read as its conjuncts plus `kind:event`, `kind:task` or `kind:journal` as `--kind event|todo|journal` names it (`event` by default), `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar, each a top-level conjunct. Without `-k` it SHALL search every calendar of the account, `calendar.default` not applied; a `-k` naming no calendar of the account SHALL be refused, naming those it holds. The words of the query are joined by spaces into one query. Calendula SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

A `date:` term SHALL compare the occurrences the index expanded (SEARCH §7), its days, `today` and `yesterday` read in the zone the tables read times in (`TZ`, then the system's), stated to the index as an IANA name (SEARCH §8.1); a zone with no IANA name SHALL read them in UTC, with a warning.

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

### Requirement: A pimdir index or query failure is coded
A store with no search index SHALL fail both commands with the code `index-missing`, an index another version built with `index-outdated`, one of another draft or shape with `index-version`, one reading floating times, or a search reading days, in a zone the time zone database does not know with `index-zone-unknown`, and a SQLite lacking FTS5 with `index-fts5-missing`, each saying the store's sync engine builds or rebuilds the index and none advising to delete anything. A query the language refuses SHALL fail with `query-refused`: in plain output the SEARCH §8.4 fault drawn over the query, under `--json` the §8.4 code, span and expectations in `error`. An accepted query the index cannot answer SHALL fail with `query-unanswerable`.

The `index-zone-unknown` row of the code table (A JSON error carries a stable code) reads: the pimdir search index reads floating times, or the search reads days, in a zone the time zone database does not know.

## ADDED Requirements

### Requirement: A pimdir search names an index read in another zone
When the index reads floating times and dates in another zone than the one `pimdir search` and `pimdir count` read days in (SEARCH §7, `floating_zone`), their plain output SHALL say so under the table or the count, naming both zones; the JSON output SHALL stay unchanged.

#### Scenario: An index read in another zone
- GIVEN `TZ=Europe/Paris` and an index reading floating times in `UTC`
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the plain output says the index reads floating times and dates in UTC and this search reads days in Europe/Paris

## REMOVED Requirements

None.
