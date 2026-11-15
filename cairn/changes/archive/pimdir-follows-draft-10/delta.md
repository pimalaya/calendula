---
cairn: delta
change: pimdir-follows-draft-10
---

## ADDED Requirements

### Requirement: A pimdir write settles no conflict (backends)
An `update` and a `remove` calendula queues SHALL state no `resolves` (pimdir STORAGE §15.3): calendula shows no conflict and decides none. The store's owner parks, rather than applies, an edit or a delete meeting a conflict, the item's or that of the binding it runs through (§15.2), so the diverging body nobody read is neither replaced nor deleted, and the parked row SHALL show in `pimdir status` as every parked action does.

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions (backends)
The body is written through the producer's `enqueue_with_body`, which runs the store's gate once. The partial supports the enqueue answers SHALL each be logged as a warning, for a `remove` and an intent too; calendula SHALL NOT run the gate apart from the enqueue.

### Requirement: pimdir refuses a create whose UID the calendar holds (backends)
The refusal comes through io-pimdir's `enqueue_with_body`.

### Requirement: A pimdir store failure carries io-pimdir's code (backends)
`lock-replaced` joins the codes carried. The error codes table (commands) gains its row.

### Requirement: A queued pimdir edit names the body it replaced (backends)
A `remove` that `delete_item` queues SHALL state its `if_match` on the terms of an `update`: the item's body hash when the delete was gated on a version, none otherwise, a `remove` stating none deleting whatever the item holds. The owner parks a delete whose item holds another body by the time it drains. Scenario added: a sync lands before a delete.

### Requirement: A pimdir calendar search says what it did not cover (commands)
The coverage gains `missing`, `unparseable` and `zoneDiffers`, flat and camelCase. `complete` is io-pimdir's judgement of SEARCH §8.2, the lag included: `zoneDiffers` leaves an answer complete, and so does a `missing` or an `unparseable` count while no term of the query reads a calendar body. The plain line names those two counts only while a term reads one. Scenarios added: a body the index could not read, a query reading no body.

### Requirement: A pimdir search names an index read in another zone (commands)
The line prints when the coverage reports a differing zone (a query holding a date term), and the JSON coverage carries `zoneDiffers`, where the line printed for every query and the JSON said nothing. Scenario added: a query of no date term.

## REMOVED Requirements

None.
