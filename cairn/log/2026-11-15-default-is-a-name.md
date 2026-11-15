---
cairn: log
change: default-is-a-name
landed: 2026-11-15
---

# Default is a name

`-a default` takes the account named `default`, following pimalaya-config d5a33cb (same change id there), and the three account resolution failures carry a code. Capabilities **config** and **commands** moved.

- **Cargo.toml**: pimalaya-config pinned to d5a33cb under `[patch.crates-io]`; Cargo.lock moved its source line alone.
- **src/error.rs**: `AccountMissing` (`account-missing`), `AccountDefaultMissing` (`account-default-missing`), `AccountDefaultSeveral` (`account-default-several`); `error::code` reads them off the chain through `account_code`, a downcast to `pimalaya_config::toml::AccountError`, before the protocol refusal readers, so every command's failure object carries them through the `with_code` call it already went through.
- **Evidence**: tests/account.rs runs the binary under `--json` for each code and for `-a default`, refused with pimalaya-config's hint, then taking an account named `default` while another is marked default. Each fails on the old code (no code, `-a default` taking the marked account). No doc, sample or test of calendula said `-a default` meant the default account.

CHANGELOG (Added codes, Changed `-a default`, **Breaking**), MIGRATION.md (the `-a default` line with its before/after, the coded account failures), README, the src/main.rs header. Spec: config ("Account resolution failures name what is missing"), commands ("A JSON error carries a stable code").
