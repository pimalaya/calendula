---
cairn: delta
id: pimdir-streamed-listing
---

# Delta

## ADDED Requirements

### Requirement: A pimdir listing streams the store
A pimdir listing SHALL walk the collection in keyset batches of the store's calendar order and hold one batch of rows, the body it is reading and the page it returns, never the collection. It SHALL stop reading once the page is full.

It SHALL read a body only to answer a kind or a window, or to return the item. Before the page of a listing naming neither, an item SHALL be counted off its row. An item whose stored summary sits in another component's table SHALL NOT be read for a kind, a resource holding one component type (pimdir STORAGE Annex A.3), and under a `UID` hint an item whose summary states another `UID` SHALL NOT be read, every component of a resource sharing one (RFC 4791 4.1); the summary keeps the value verbatim, so one holding an escape or padding SHALL be read rather than compared.

A summary SHALL NOT rule an item out by its dates: an override moves an occurrence anywhere and the summary describes the master alone. An item a pending `update` repoints SHALL NOT be ruled out by its summary, which describes the body it replaces.

### Requirement: A UID hint narrows a find
The listing query MAY carry a `UID`, which `event find` sets. A backend MAY leave out an item it can tell carries another `UID` without reading it, and SHALL ignore the hint on a listing naming a page size, which leaving items out would shift. The caller SHALL still compare the `UID` of every event it gets.

## MODIFIED Requirements

## REMOVED Requirements
