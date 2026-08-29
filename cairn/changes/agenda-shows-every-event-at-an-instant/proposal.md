---
cairn: change
id: agenda-shows-every-event-at-an-instant
status: landed
created: 2026-08-28
---

# The agenda drops an event that starts at the same instant as another

## Why

The agenda collects its labels into a map keyed by the start instant, one label per key, so two events starting at the same moment print as one: the second overwrites the first and nothing says it did.

Two unrelated meetings at 09:00 are enough to trigger it, which makes this a defect the calendar has today.

What raises it from latent to systematic is the change in flight beside it (`duplicate-link-id-mints-an-item`). A calendar collection may hold two resources under one `UID`, and the store used to keep one of them; it will now keep both.

The verified case is a Posteo calendar holding "Pre demo woonies" and "Pre demo MINIS" under one `UID`. Both will list, both start at the same instant, and the agenda will show one of them.

A change whose whole purpose is to stop losing the second copy would deliver it into a view that drops it again.

It is separated from that change deliberately: it is a pre-existing defect with its own cause, and fixing it changes a user-facing output shape, which is a decision to take on its own terms rather than inside a storage change.

## What

- The agenda holds every event at an instant, not the last one written, and renders all of them in a stable order.
- The `--output json` payload changes shape accordingly, from one label per instant to a list. That is the breaking part, and the reason this is proposed rather than done.
- The ordering between two events at one instant is stated rather than incidental, so the same calendar renders the same way twice.

## Scope / non-goals

- **No dedup.** Two events at one instant are two events, including when they share a `UID`. Deciding they are the same thing is the user's, against their server.
- **No change to the range filter or to how a day is chosen**, only to what a chosen instant may hold.
- **No new output flag.** A second shape kept for compatibility would leave the dropping behaviour reachable, which is the whole point of the change.
