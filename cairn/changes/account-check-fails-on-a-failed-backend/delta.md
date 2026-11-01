---
cairn: change
change: account-check-fails-on-a-failed-backend
---

# Delta

## ADDED Requirements

### Requirement: account check fails when a backend fails
`account check` SHALL exercise every backend of the account `--backend` allows and print one verdict per backend when all of them pass. When any fails, it SHALL exit non-zero with one error naming each failed backend and why, so a script reads the outcome from the exit code alone.
