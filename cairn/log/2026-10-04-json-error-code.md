---
cairn: log
change: json-error-code
landed: 2026-10-04
---

# A JSON error names a stable code

`main` prints failures through `error::eval` (src/error.rs, as in himalaya), adding `code` under `--json` when the chain carries a `CodedError`. The pimdir backend raises `body-pending` for an unhydrated read, `precondition-failed` for a stale `if_match` and `occurrence-unsupported` for an occurrence its performer cannot act on alone.

Capabilities moved: **commands** (a JSON error carries a stable code).
