---
cairn: log
change: account-check-fails-on-a-failed-backend
landed: 2026-10-30
---

# account check fails when a backend fails

`AccountCheckCommand::execute` fails with `Account <name> failed its check: <backend>: <reason>; ...` when any checked backend failed, instead of printing a FAIL row and exiting 0; a passing check prints the report as before. The `check_pimdir` doc no longer claims opening the store creates an index.

Test: `a_failed_backend_fails_the_check` over a vdir home that exists, then one that does not; the second returned `Ok` on the old code.

Capabilities moved: **commands** (account check fails when a backend fails, added).
