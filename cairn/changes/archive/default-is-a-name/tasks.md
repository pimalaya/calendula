---
cairn: tasks
change: default-is-a-name
---

# Tasks

- [x] Pin pimalaya-config d5a33cb; Cargo.lock moves its source line alone.
- [x] `AccountMissing`, `AccountDefaultMissing`, `AccountDefaultSeveral` in src/error.rs, read by `account_code` in `error::code`.
- [x] Offline tests through the binary (tests/account.rs): each code under `--json`, `-a default` refused with the hint, then taking an account named `default`.
- [x] No doc, sample or test said `-a default` meant the default account; nothing else to update.
- [x] CHANGELOG (Added codes, Changed `-a default`, **Breaking**), MIGRATION.md, the src/main.rs header.
- [x] Fold the delta, log, set `landed`.
