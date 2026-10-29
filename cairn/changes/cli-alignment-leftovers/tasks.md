---
cairn: tasks
change: cli-alignment-leftovers
---

# Tasks

- [ ] C3: decide the wording of a pimdir write confirmation, which only queues locally (src/shared/event/create.rs:138), and apply it to every pimdir write
- [ ] C4: resolve `pimdir.account` as himalaya does (himalaya/src/pimdir/client.rs:225): derive it when the store holds one account, error naming the accounts when it holds several (src/pimdir/client.rs:64, src/pimdir/backend.rs:451-457)
- [ ] B3: add an `alpn` key to the `caldav` block, defaulting to `http/1.1` (src/caldav/client.rs:146); update config.sample.toml
- [ ] F1: rename `CalendulaCli` and `CalendulaCommand` to `Cli` and `Command` (src/cli.rs:58, :95, src/main.rs:162-211)
- [ ] F7: add `footer!` from pimalaya-cli to the clap command
- [ ] G2: build one `SecretResolver` per account where the runtime assembles its backends (src/shared/client.rs:533-540)
- [ ] A2: once released, drop the git pins for ical @caa1422, io-pimdir @3b4bed8 and io-webdav @62fad89 (Cargo.toml:70, :73, :74)
- [ ] H1, H2: once pimalaya-cli ships them, drop src/wizard/configure.rs:91, :220, :246 and src/cli.rs:141
- [ ] CHANGELOG under [Unreleased]
- [ ] Fold into the spec; log; land
