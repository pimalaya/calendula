---
cairn: delta
id: a-family-page-holds-that-family
---

# Delta

## ADDED Requirements

### Requirement: A page of a family holds that family
A listing SHALL narrow by component kind before it paginates. Filtering after paginating makes a page mean "the components of this kind among the first N items of any kind", which on a calendar dominated by another kind is empty however many matches the calendar holds.

The kind SHALL travel with the page and the window as one query, so no backend can serve one without the others. A backend SHALL push the kind down where its protocol defines such a filter: CalDAV as an RFC 4791 `comp-filter`, with the `time-range` nested inside it as 9.7.1 requires. A backend modelling one kind only SHALL answer any other kind without a round-trip.

The raw `item` family names no kind and SHALL keep every one, which is what makes it the unfiltered view.

A pimdir item whose body is not local SHALL answer its kind from the stored `v: 1` summary, which names the component a reader renders the resource as, so narrowing by kind keeps the listing availability-aware.
