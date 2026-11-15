---
cairn: change
id: a-backend-left-out-is-named
status: landed
created: 2026-11-15
---

# A backend left out is named

> Item 16 of the 2026-11-15 owner round, the same rule in every app with backend cargo features.

## Why

Since an-unknown-key-is-refused, the block of a backend left out of the build loads unread, so one file serves every build. But nothing said so afterwards: `auto` skipped the block as if absent and fell through to the next backend, or failed uncoded, saying no backend matching `auto` is configured for this account; `account list` and `account capability` showed the account without it; `--backend gcal` on a build without gcal said `Invalid backend`. A person sharing a configuration between a full and a reduced build could not tell a missing block from a missing feature.

## What

- `AccountConfig::blocks` lists every block the account carries, with whether the build serves it; `backends` and `not_built` split it.
- `resolve_backend` walks the blocks in priority order; under `auto` a block of a backend left out stops it, never falling through, so one configuration reaches the same backend in every build. The unified client fails coded `backend-not-built`: "This build leaves out the `gcal` backend, rebuild with `--features gcal`", plus "; this account's msgraph backend is built (`--backend msgraph`)" when one is.
- `account check` under `auto`: one failed verdict per left-out block, coded `backend-not-built` (exit 3).
- `account capability`: fails coded `backend-not-built` with the unified commands' message, as cardamum does (owner rule: a command using that account's backend fails coded).
- `account list`: `notBuilt` beside `backends` (unchanged, the built ones), the text cell `gcal (not built)`.
- `--backend gcal` in a build without gcal: clap's refusal (exit 2, `usage-invalid`) names the feature instead of `Invalid backend`.
- Typos stay refused: only the five backend names load unread.
- Spec: backends ("Backend selection order"), config ("Unknown fields"), commands (code table, account check, account capability, a new requirement for account list), packaging.
