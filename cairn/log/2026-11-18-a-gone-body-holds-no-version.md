---
cairn: log
change: a-gone-body-holds-no-version
landed: 2026-11-18
---

# A gone body holds no version

Landed without a proposal: the two twin gaps cardamum's session left after TRI-3 (calendula 2afc519), aligned on cardamum.

- `update_item` read the item's current body through `body`, which fails `body-pending` when the blob is gone, so an update given a source over such an item failed where cardamum queues it. It now reads `local_body`, `None` for a gone blob, the online-meeting check then treating any ask as new.
- `check_version` compared `--if-match` with the item's object hash even when its blob was gone, so the former hash passed. It now takes the version held, `None` without a local body, in `update_item` and `delete_item` alike, any `--if-match` then failing `precondition-failed`.

Test `a_write_over_a_missing_body_matches_no_version`. A read (`item read`, `event read`, an `event update` starting from the stored item, `pimdir reply`, `pimdir cancel`) still fails `body-pending`.

Capabilities moved: backends.
