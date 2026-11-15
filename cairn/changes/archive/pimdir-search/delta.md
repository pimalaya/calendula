---
cairn: delta
change: pimdir-search
---

## ADDED Requirements

### Requirement: pimdir searches calendar items through the store's index
`calendula pimdir search <query>...`, gated on the `pimdir` feature, SHALL answer the pimdir SEARCH query language (§8) from the store's search index, scoped to one component kind of the account: the query is read as its conjuncts plus `kind:event`, `kind:task` or `kind:journal` as `--kind event|todo|journal` names it (`event` by default), `account:<account>` when the store groups the account under a name, and `collection:<id>` when `-k/--calendar` names a calendar, each a top-level conjunct. Without `-k` it SHALL search every calendar of the account, `calendar.default` not applied; a `-k` naming no calendar of the account SHALL be refused, naming those it holds. The words of the query are joined by spaces into one query. Calendula SHALL open the index read-only (io-pimdir `PimdirIndex::open_query`) and SHALL NOT create, rebuild nor refresh it: the store's owner, its sync engine, does. The answer SHALL be the store as of the owner's last refresh: a queued write is not listed nor folded in.

A `date:` term SHALL compare the occurrences the index expanded (SEARCH §7). Its days, `today` and `yesterday` are read in UTC until the store's index and this search can state the user's zone, and the command's help SHALL say so.

Each hit SHALL render with its kind's columns (Columns follow the kind), from its summary when its body is not local, then its calendar, what a text term matched (`body`, `summary`, or `null` when only field terms did) and a `snippet`; its `id` is the public `seq`, which the family's `read` takes. `--sort date|name|relevance` orders the hits, `date` (latest first) by default, a series by its first occurrence, and `--page` and `--page-size` page them, the page size defaulting to the kind's `list` one. Under `--json` the page carries `hits`, `more` (whether hits follow it) and `coverage`, each hit the kind's projection plus `calendarId`, `matched` and `snippet`.

#### Scenario: A series found by a later occurrence
- GIVEN an indexed pimdir calendar holding a weekly event begun on 2019-01-07, a Monday
- WHEN `calendula pimdir search date:2026-11-09` runs
- THEN the event is listed

#### Scenario: A day is a UTC day
- GIVEN an indexed event at 00:30 UTC on 2026-11-07
- WHEN `calendula pimdir search date:2026-11-07` runs
- THEN the event is listed, and `date:2026-11-06` does not list it

#### Scenario: A queued event is not listed
- GIVEN an event created in a pimdir calendar and still in the queue
- WHEN `calendula pimdir search` runs a query its body matches
- THEN it is not listed until the owner applies the create and refreshes the index

### Requirement: pimdir counts calendar items from the index
`calendula pimdir count <query>...` SHALL answer how many items the scoped query matches, with its coverage, under the same scope as `pimdir search`.

### Requirement: A pimdir calendar search says what it did not cover
The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the items in scope whose body is not local, matched on their summary alone; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `since` and `until`, the range below or above which the store is not known to hold every item; `filling`, whether a first round is still listing a calendar in scope; `horizonLeft`, whether the date range reaches outside the occurrences the index holds; `cut`, how many series stop their occurrences before the range's end; and `complete`, true only when none of these holds. The plain output SHALL print one line under its table naming what did not hold, and nothing when the answer is complete.

#### Scenario: A range past the horizon
- GIVEN an index holding occurrences two years forward
- WHEN `calendula pimdir search date:2030` runs
- THEN the coverage reports `horizonLeft` true and `complete` false

### Requirement: A pimdir index or query failure is coded
A store with no search index SHALL fail both commands with the code `index-missing`, an index another version built with `index-outdated`, one of another draft or shape with `index-version`, one reading floating times in a zone the time zone database does not know with `index-zone-unknown`, and a SQLite lacking FTS5 with `index-fts5-missing`, each saying the store's sync engine builds or rebuilds the index and none advising to delete anything. A query the language refuses SHALL fail with `query-refused`: in plain output the SEARCH §8.4 fault drawn over the query, under `--json` the §8.4 code, span and expectations in `error`. An accepted query the index cannot answer SHALL fail with `query-unanswerable`.

## MODIFIED Requirements

### Requirement: Protocol-specific families
`configure` (alias `wizard`) SHALL run the account wizard.

`caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the shared API has no home for.

pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, how many creations are queued for the next sync, and every action the store's owner parked in those calendars, by its queue row (ROW), its calendar, its kind and the error that parked it; `reply` and `cancel`, the scheduling intents only its sync engine can perform; and `search` and `count`, the store's search index queried in the pimdir SEARCH language.

A backend MAY exist without a family where its protocol adds nothing the shared surface lacks; a family SHALL NOT push protocol-specific concepts into the shared API instead.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording.

| Code | Raised when |
|---|---|
| `body-pending` | a listed item's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the item no longer has (pimdir) |
| `occurrence-unsupported` | `pimdir reply` or `pimdir cancel` names an occurrence its performer cannot act on alone |
| `item-duplicate` | a pimdir create names a `UID` its calendar holds or has queued |
| `store-uncreated` | the pimdir store has no schema yet: its sync engine creates it |
| `store-version` | the pimdir store is of another format version than calendula's: an earlier sync engine's is reinstalled, a newer one's needs calendula upgraded |
| `store-foreign` | the pimdir database is not a pimdir store: another program wrote it |
| `store-corrupt` | the pimdir store's two version stamps disagree, or its schema is not the canonical one |
| `store-hash-algo` | the pimdir store names its bodies by a hash this calendula does not know: upgrade calendula |
| `store-busy` | another process holds the pimdir store; retry |
| `store-exposed` | a path of the pimdir store is a link or another user can reach it: restrict it |
| `flag-invalid` | a pimdir write carries a flag the store cannot hold |
| `action-malformed` | a pimdir queue action's payload is one its schema refuses |
| `index-missing` | the pimdir store has no search index yet: its sync engine builds it |
| `index-outdated` | the pimdir search index was built otherwise or from another store: its sync engine rebuilds it |
| `index-version` | the pimdir search index is of another draft or shape: its sync engine rebuilds an earlier draft's, and leaves any other |
| `index-zone-unknown` | the pimdir search index reads floating times in a zone the time zone database does not know |
| `index-fts5-missing` | the SQLite calendula links lacks FTS5 or is older than 3.43 |
| `query-refused` | the pimdir search query is malformed or names an unknown field or value |
| `query-unanswerable` | the pimdir search index could not answer an accepted query |

#### Scenario: A stale version
- GIVEN a pimdir event whose body changed since it was read
- WHEN `calendula --json event delete --if-match <old etag>` names it
- THEN the command exits 1, queues nothing and prints `{"code":"precondition-failed","error":"Precondition failed: …","sources":[],"backtrace":null}`
