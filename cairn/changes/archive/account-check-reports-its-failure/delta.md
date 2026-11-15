---
cairn: delta
change: account-check-reports-its-failure
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: account check fails when a backend fails
(commands) `account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend: under `--json`, `ok`, the stable `code` of a failure (the code vocabulary of "A JSON error carries a stable code", `null` when the backend passed or its failure carries none) and the error of a failed one; the text shows no code. When any fails, it SHALL still print the report and exit 2, the family's partial outcome (`pimalaya_cli::error::PartialOutcome`), so a script reads the outcome from the exit code alone and which backends passed from the report; when all pass, it SHALL exit 0. It mirrors Himalaya's `account check`.

#### Scenario: One backend of two fails
- GIVEN the account `test` whose vdir home exists and whose pimdir root does not
- WHEN `calendula account check` runs
- THEN it prints `Account:  test`, `vdir:     OK` and `pimdir:   FAIL (<why>)`, and exits 2
- AND `calendula --json account check` prints the same report as its result, the pimdir verdict carrying `"code":"store-uncreated"` and the vdir one `"code":null`, and exits 2, and `calendula --backend vdir account check` exits 0

## REMOVED Requirements

None.
