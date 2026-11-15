---
cairn: change
id: with-code-is-shared
status: landed
created: 2026-11-15
---

# with_code is shared

> Item 13 of the 2026-11-15 owner round: pimalaya-cli 51ce8b1 carries the helper every app had its own copy of.

## Why

`error::with_code`, which rebuilds a chain around an innermost `CodedError` so the failure object reads a code a reader found deep in it, was calendula's own copy of a helper every Pimalaya CLI carried. pimalaya-cli 51ce8b1 ships it as `pimalaya_cli::error::with_code(err, code)`.

## What

- Pin pimalaya-cli to 51ce8b1 in `[patch.crates-io]`.
- Delete `error::with_code`; every former call site calls `pimalaya_cli::error::with_code(err, error::code)`. Its test of the helper's mechanics goes with it, now pimalaya-cli's; the tests asserting calendula's codes through it stay.
- No behaviour change: the shared helper also leaves alone a chain coded `usage-invalid` or holding a `PartialOutcome`, neither of which calendula's `code` ever coded, so every failure object is the same. No spec moves.
