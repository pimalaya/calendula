---
cairn: change
id: account-check-fails-on-a-failed-backend
status: landed
created: 2026-10-30
---

# account check fails when a backend fails

## Why

`account check` printed `pimdir: FAIL (...)` and exited 0 (src/account/check.rs:56-60, reproduced on a stale store 2026-10-30). The output contract makes the exit code the only success signal on stdout, so a script or CI step running `calendula account check` never saw the failure.

## What

- `account check` prints its report when every checked backend passed, and otherwise fails with one error naming each failed backend and why, printed (plain or JSON) as any failure is, with a non-zero exit. One value on stdout either way.
- The wizard, which runs the same check on the account it builds, is unchanged.
