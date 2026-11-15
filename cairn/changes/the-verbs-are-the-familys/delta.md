---
cairn: delta
change: the-verbs-are-the-familys
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: One family per stored component kind
(commands) `event` SHALL cover `list`, `read`, `search`, `add`, `update` and `delete` over the item API, and `item` `list`, `read`, `add`, `update` and `delete`: the verbs of Himalaya, `add` storing an item as `message add` does and `search` looking events up as `envelope search` does, `create` and `find` kept as their hidden aliases.

### Requirement: An event is found by its UID
(commands) `event search <UID>` SHALL list, unexpanded, every VEVENT of the calendar carrying that `UID`, with the id the other `event` commands take. `find` SHALL stay its hidden alias.

### Requirement: A UID hint narrows a search
(commands) Renamed from "A UID hint narrows a find": the listing query MAY carry a `UID`, which `event search` sets.

### Requirement: A page names the next one
(commands) `event search` SHALL page the same way: it reads the calendar whole, then cuts its matches by `-s` and `-p` or the token, a page counting resources as `event list` does; with no `-s` every match is one page, `nextPage` `null`.

### Requirement: Tools are the classified unified API
(mcp) The tools are named after the leaves: `event_search`, `event_add` and `item_add`, where `event_find`, `event_create` and `item_create` were.

Every other requirement naming `event create`, `item create` or `event find` (commands, mcp, repl, backends) names `event add`, `item add` or `event search` instead, unchanged otherwise.

## REMOVED Requirements
