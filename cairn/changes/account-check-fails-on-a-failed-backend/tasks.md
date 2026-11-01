---
cairn: tasks
change: account-check-fails-on-a-failed-backend
---

# Tasks

- [x] Test: `account check` over a backend that fails fails with an error naming it and why; fails on the old exit 0.
- [x] Return the error from `AccountCheckCommand::execute`.
- [x] Fix the `check_pimdir` doc, which says opening creates the index.
- [x] CHANGELOG Fixed entry.
- [x] Fold the delta into [commands](../../spec/commands.md); log; land.
