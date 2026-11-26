---
cairn: delta
change: follows-storage-draft-17
---

## ADDED Requirements

### Requirement: A queue row that does not decode fails no listing
A pimdir listing (`event list`, `event agenda`, `event search`, `event read --uid`) SHALL NOT fail for a queue row whose payload does not decode, whichever collection holds it. The pending updates a listing reads SHALL be those of the account's calendars. When a calendar's queue does not decode, the listing SHALL read every hydrated item of that calendar from its body rather than rule items out by their summary, and SHALL log a warning naming the calendar. `pimdir queue list` keeps naming such a calendar in an `action-malformed` entry.

#### Scenario: A malformed row in another calendar
- GIVEN the calendars `a` and `b` of one account, and a queue row of `b` whose payload states no integer version
- WHEN `calendula --json event list -k a` runs, then `-k b`
- THEN both list their events and exit 0

## MODIFIED Requirements

### Requirement: An event search runs the pimdir query language
As in commands.md, but for its `--sort` sentence: `--sort newest-first|oldest-first|title|relevance` orders the events, `newest-first` by default (pimdir SEARCH §8.2). `date` and `name` SHALL be read as `newest-first` and `title`, and shown by no help, manual or completion. The query takes the forms and refusals "pimdir searches calendar items through the store's index" names.

### Requirement: pimdir searches calendar items through the store's index
As in commands.md, with three changes:
- The `--sort` sentence reads: `--sort newest-first|oldest-first|title|relevance` orders the hits, `newest-first` by default, a series by its first occurrence; `date` and `name` are read as `newest-first` and `title`, hidden.
- After the `date:` paragraph: a `date:` or `when:` term SHALL take an instant (`2026-11-23T10:00`, with seconds, `Z` or an offset, read in the zone above when it states none) and relative amounts in `min`, `h`, `d`, `w`, `mo` and `y` (SEARCH §8.1).
- After the instants: a word or a value beginning with `~`, a word or a value of two characters or more beginning with `/`, and a value beginning with `{` SHALL be refused `query-refused` naming `reserved-form` (SEARCH §8.3). `label:` and `property:` SHALL be answered from the store (SEARCH §8).

### Requirement: A pimdir listing streams the store
As in backends.md, plus, folded only if tasks.md H3 lands: a listing over a window SHALL take its candidates from the store's search index when the index answers the window completely (no lag, the window inside its horizon, no series cut, its floating zone the zone the window is read in, else a margin of a day each way). It SHALL then read the bodies of those candidates alone, plus the items the index has not folded, those with no local body (by their summary) and those a pending `update` repoints. Otherwise it SHALL walk the collection as above. Either way it SHALL list what the walk lists.

## REMOVED Requirements

None.
