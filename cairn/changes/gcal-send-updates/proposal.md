---
cairn: change
id: gcal-send-updates
status: active
created: 2026-10-28
---

# A gcal write chooses who is mailed

## Why

On a gcal account, whether the attendees get an email is a side effect nobody chooses (#21). `update_item` and `delete_item` pass io-gcal's default params, so `sendUpdates` is omitted and Google decides: in the tests for #20 it mailed the guest on every update and on the delete ("Updated invitation" on a move, "Event cancelled" on removing the attendee or deleting the event). Fixing a typo in a title, or tidying up an old event, cannot be done without mailing everyone on it. `create_item` now decides on its own too, `sendUpdates=all` on the insert path (gcal-create-invites).

## What

- `event create`, `event update` and `event delete` take `--send-updates all|external-only|none`, passed to io-gcal as `GcalSendUpdates`.
- Absent, nothing changes: update and delete omit `sendUpdates`, and create keeps the choice gcal-create-invites makes. Existing scripts mail the same people as before.
- On `event create` the flag overrides that choice on the insert path, so `--send-updates none` stores an invitation the account organizes without mailing anyone yet. An event imported instead (one whose `ORGANIZER` is someone else) goes through `events.import`, which takes no `sendUpdates` and mails nobody: `none` is what it does already and passes, while `all` and `external-only` are refused naming the organizer, since Google would mail nobody whatever the flag said. An event with no attendee has nobody to mail, and the flag passes.
- On every backend but gcal the three commands refuse the flag by name, before reaching the backend, as `--online-meeting` is refused off pimdir. CalDAV leaves scheduling to the server (RFC 6638, `SCHEDULE-AGENT`), and msgraph has no per-request switch of this kind that I found; either can lift the refusal later if it gains a way to honour it.
- `event build` reaches no backend and does not take the flag.

## Out of scope

- `item delete`, `todo …` and `journal …`: Google stores events only, and `item` is the raw surface.
- The `if_match` on a gcal delete: io-gcal 0.2.0's `event_delete` takes an `if_match`, so the refusal in `delete_item` ("Google Calendar cannot gate a delete on an ETag") may be lifted, but that belongs to its own change.

## Open question

Threading: the flag reaches the backend as one more `Option` argument on the shared client's `create_item`, `update_item` and `delete_item`, every arm but gcal receiving `None` since the command refused it before. A small write-options struct carrying it together with `if_match` would keep the signatures from growing further; I lean to the plain argument unless you prefer the struct.
