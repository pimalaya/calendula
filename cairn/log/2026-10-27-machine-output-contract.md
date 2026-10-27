---
cairn: log
change: machine-output-contract
landed: 2026-10-27
---

# Adopted the shared machine output contract

The coded-error mechanism moved to pimalaya-cli (`55b6693`, pinned until 0.3.0); src/error.rs keeps only the `ErrorCode` vocabulary, and main.rs evaluates through `ErrorReport::<ErrorCode>::eval_coded`. The JSON error object prints exactly as before.

`json-schema` gained `calendula-error`, and each schema now carries `$id` = `https://pimalaya.org/schemas/calendula/v0.2/<key>.json`. A test asserts every payload root is an object. Eight were not: the `create` and `update` outputs of `event`, `todo`, `journal` and `item`, whose untagged abandoned arm serialized as `null`. That arm is now an empty struct variant, printing `{}`; it is reachable through `-i` alone, which `--json` refuses, so no JSON a caller can get moved. Piped `--json` is now one value per line, from the pimalaya-cli printer.

Verified: 200 binary tests pass with all features, clippy clean on all features, `--no-default-features` builds (28 pre-existing warnings, one fewer than before).

Spec updated: commands (ADDED: The JSON contract is described and versioned; MODIFIED: Every data command returns a named output type).
