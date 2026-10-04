---
cairn: change
id: json-error-code
status: landed
created: 2026-10-04
---

# A JSON error names a stable code when the caller has something to do

## Why

A front-end (MOA) acts on a few failures: a body not downloaded yet, a stale `--if-match`, an occurrence the performer cannot act on alone. Matching the wording breaks at the first rewording. himalaya added a stable `code` beside pimalaya-cli's report; calendula follows it.

## What

`src/error.rs`, as in himalaya: `ErrorCode` and `CodedError`, found anywhere in the error chain, printed as `code` under `--json`. Codes: `body-pending`, `precondition-failed`, `occurrence-unsupported`. A failure without one prints as before.
