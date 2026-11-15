---
cairn: tasks
change: the-shared-kit
---

# Tasks

- [x] Cargo.toml: pimalaya-cli and pimalaya-config 0.4, pimalaya-cli patched to cli 0fbe5dd, clap without default features, crossterm out.
- [x] src/cli.rs: `Cli`, `Command`, `footer!`, `ConfigFlags`, `resolve_account` on `load` and `take_account`, `meet_bare_invocation`, `welcome()`.
- [x] src/main.rs: the welcome for a `ConfigMissingError` a person meets; the header's Configuration and Output sections.
- [x] src/config.rs: `sample_url`, `accounts`; the table and colour blocks gone, `table` and `account` refused by name; `caldav.alpn`.
- [x] Plain text: `Columns`, `Fields`, `Records` and `sanitize::text` at every text output, `-w` gone, ariadne and the agenda colourless.
- [x] `event list`, `event find`, `item list` and `gcal instances` between a header and a footer; the unified client reading one item past the page; `nextPage` in the JSON.
- [x] One `SecretResolver` per unified client; `CaldavBackend::new` takes it.
- [x] Tests, each seen failing against the old code: a missing configuration typed for the welcome (the old binary under `script`), an unknown account listing the held ones, several defaults refused, `-c` never split and repeated in order, a byte-exact `event list` page and last page (tests/event_list.rs), a colour key and a `table` or `account` table refused, `-w` refused, `caldav.alpn` read, a reopened backend running its secret command once; tests/first_run.rs for the welcome.
- [x] README, config.sample.toml header and its table and colour lines, CHANGELOG, MIGRATION.md.
- [x] `cargo fmt`, clippy over the feature matrix, `cargo test --all-features`, `json-schema` diffed against the old binary, the binary run under `script` and on a scratch vdir account.
- [x] Fold the delta into the spec (commands, config, packaging), log, tick the-family-standards, `sh cairn/verify.sh`.
