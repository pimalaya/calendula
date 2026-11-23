---
cairn: log
change: an-update-icloud-refuses-on-any-version
landed: 2026-11-23
---

# An update iCloud refuses on any version

Landed without a proposal: a bug of the live test night of 2026-11-23 (lead seen by the neverest lane, reproduced by the calendula lane).

- On iCloud, `event update -k <calendar> <id>.ics <ICAL>` with no `--if-match` failed ``Event `<id>` not found``, caused by HTTP 412: `update_item` sends `If-Match: *` (CAL-2) and iCloud answers 412 to it over a resource it holds. An update given flags alone passed, as it reads the item and sends its ETag.
- `CaldavBackend::update_item` now follows a 412 to `If-Match: *` with a read of the item: a read answered 400, 404 or 410 refuses the update `item-not-found` as before, nothing written; a read serving an ETag sends the PUT again gated on it; a read serving none fails the update with the 412.
- Tests: `an_update_refused_on_any_version_is_sent_again_on_the_read_one` (red before the fix: `Item e1.ics not found in cal`), and `an_update_of_a_missing_item_creates_nothing` now scripts the read answering 404, 410 and 400 and checks one PUT only.
- Live: the iCloud scenario re-run passed (`event update` from a file, `item update`), an unknown `nope.ics` still refused and nothing created; Stalwart and Fastmail unchanged.

Capabilities moved: backends.
