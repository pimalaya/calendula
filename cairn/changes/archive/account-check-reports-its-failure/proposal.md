---
cairn: change
id: account-check-reports-its-failure
status: landed
created: 2026-11-12
---

# A failed account check prints its report

## Why

account-check-fails-on-a-failed-backend (2026-10-30) made a failed `account check` exit 1 with one failure object naming each failed backend, where 0.2.0 printed the report and exited 0. The family output contract since (pimalaya-cli cairn/spec/output.md, every-output-is-data) prints a result whose outcome is not a full success and exits 2, a failed `account check` being its example: Himalaya's `account check` (himalaya/src/account/check.rs) and cardamum's (its change account-check-reports-its-failure, 57e30dd) do. calendula's failure object hid which backends passed from a `--json` caller. Found by the cross-check of the-family-standards with cardamum and Himalaya on 2026-11-12.

## What

- `account check` prints its report whatever the outcome (`Account:`, then one `<backend>:` line, `OK` or `FAIL (<why>)`; under `--json` the report as the result) and ends with pimalaya-cli's `PartialOutcome` (exit 2, nothing printed after the report) when a backend failed, 0 when all passed. An account with no backend `--backend` allows still fails with exit 1.
- Each verdict carries `code` under `--json`, the stable code of the failure (`code_of`), `null` when the backend passed or its failure carries none, as Himalaya's `BackendCheck` does (owner, 2026-11-12): a script reads `store-busy` or `store-uncreated` from a failed pimdir check. The text shows none, as Himalaya's. The 2026-10-30 failure object carried no code (`"code":null`, its message folding each failure into a new error), so this is new to a script, not restored.
- The command's help says so; the spec requirement the 2026-10-30 change wrote is replaced; CHANGELOG's unreleased Fixed line is reworded; log.

## Scope / non-goals

cardamum's `BackendCheck` carries no `code`; the family shape is Himalaya's.
