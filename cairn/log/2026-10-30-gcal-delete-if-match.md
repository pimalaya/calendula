---
cairn: log
change: gcal-delete-if-match
landed: 2026-10-30
---

# A gcal delete honours `--if-match`

`GcalBackend::delete_item` in src/gcal/backend.rs passes `if_match` to io-gcal's `event_delete`, which sends it as `If-Match`, instead of refusing it. A stale version fails with Google's 412 and the event stays; vdir still refuses the flag.

Verification: `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` with all features; checked by hand against a Google calendar.

Capabilities moved: **backends** (gcal honours `if_match` on a delete, modified), **commands** (a delete can be gated on a version, modified).
