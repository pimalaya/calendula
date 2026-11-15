---
cairn: log
change: a-backend-left-out-is-named
landed: 2026-11-15
---

# A backend left out is named

The block of a backend the build leaves out still loads unread, and a command reaching it now fails coded `backend-not-built`, naming the cargo feature, where it was skipped as absent (item 16 of the 2026-11-15 owner round). Capabilities **backends**, **config**, **commands** and **packaging** moved.

- **src/config.rs**: `AccountConfig::blocks` lists every block the account carries with whether the build serves it (the five fields exist in every build, typed or `IgnoredAny`); `backends` keeps the built ones, `not_built` the others.
- **src/backend.rs**: `ALL`, `not_built` ("This build leaves out the `gcal` backend, rebuild with `--features gcal`"); `FromStr` gives that message for a known name the build leaves out, so `--backend gcal` exits 2 naming the feature.
- **src/shared/client.rs**: `resolve_backend` walks the blocks in priority order and, under `auto`, stops on a left-out one, returned as its error; `CalendarClient::new` fails with `error::backend_not_built`, which adds the first built backend of the account and its `--backend`. Decided: `auto` never falls through to the next backend, so a configuration reaches the same backend, and the same ids, in every build.
- **src/account/check.rs**: under `auto`, one failed row per left-out block, coded `backend-not-built`, after the built ones (exit 3). **src/account/capability.rs**: fails coded `backend-not-built` with the unified commands' message, aligned with cardamum (a first draft listed every row `none` and succeeded; the coordinator ruled it a command using that backend). **src/account/list.rs**: `notBuilt` beside `backends`, which keeps its meaning (the built ones, safe to pass to `--backend`); the text cell appends `gcal (not built)`.
- **src/error.rs**: `BackendNotBuilt`, `backend_not_built`.
- **Evidence**: tests/account.rs, gated on `not(feature = "gcal")` so it runs in the no-backend build and every single-backend build but gcal's: a gcal-only account fails `calendar list` coded `backend-not-built` naming `--features gcal`, its `account check` row carries the code, `--backend gcal` exits 2 naming the feature, `account list` reads `notBuilt: ["gcal"]` and `gcal (not built)`, `account capability` fails coded `backend-not-built`; a typo beside the block stays refused; and, in the msgraph-only build, `calendar list` and `account capability` name `--backend msgraph`. All but the typo test fail on the old code (uncoded "No backend matching", no `notBuilt`, a `null` backend).

CHANGELOG (Added `backend-not-built`, Changed `auto`, **Breaking**), MIGRATION.md, README, the src/main.rs header, the `--backend` help. Spec: backends ("Backend selection order"), config ("Unknown fields"), commands (code table, account check, account capability, the new "An account names the backends left out of the build"), packaging.
