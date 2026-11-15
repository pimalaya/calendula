---
cairn: tasks
change: the-wizard-leaves
---

# Tasks

- [x] tests/first_run.rs: a bare invocation without configuration prints the help and no `configure`, and `account list`, `account check` and `calendar list` without one fail naming the path and the sample; both seen failing on the old code.
- [x] Delete src/wizard/, the `configure` command and its `wizard` alias.
- [x] src/main.rs: bare `calendula` prints the help; the header's configuration section no longer describes the wizard.
- [x] src/cli.rs: drop both `offer_configuration` variants and the offer in `load_config`; `load_config` and `resolve_account` lose the printer, with the client builders.
- [x] src/config.rs: one `NO_CONFIG_HINT`; drop `CONFIG_SAMPLE_URL`, `AccountConfig::render`, `RENDER_ORDER`, `ENDPOINT_KEYS`, `is_default`, `is_default_msgraph_user_id`, the `skip_serializing` attributes and the `Serialize` derives, after checking no `*Output` type carries a configuration type.
- [x] src/account/check.rs: drop `all_ok`, `check_account` private, the missing configuration through `load_config`; src/caldav/client.rs: `resolver` folded into `resolve_server`.
- [x] src/json_schema.rs: drop `calendula-configure`.
- [x] Cargo.toml: drop the `wizard` feature and its place in `default`, shellexpand; io-pim-discovery optional under `caldav` with `rfc6764` and `stream`, its TLS forwards weak; `cargo tree` with `rustls-ring,vdir` builds no io-pim-discovery.
- [x] README, CONTRIBUTING, config.sample.toml, CHANGELOG (one Removed entry), MIGRATION.md.
- [x] first-run-wizard-frame abandoned; wizard-discovery-first, wizard-runs-when-unconfigured and wizard-save-path moved to changes/archive/.
- [x] `cargo fmt`, `cargo clippy --all-targets -D warnings` (default, all features, `rustls-ring` with each backend), `cargo test --all-features`, `json-schema` diffed against the old binary.
- [x] Fold the delta into the spec (wizard.md removed; config, commands, packaging), log, set status landed; `sh cairn/verify.sh`.
