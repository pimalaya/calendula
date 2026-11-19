---
cairn: tasks
change: cli-alignment-leftovers
---

# Tasks

- [x] C3: decide the wording of a pimdir write confirmation, which only queues locally (src/shared/event/create.rs:138), and apply it to every pimdir write (done by a-pimdir-write-says-it-is-queued, 2026-10-30)
- [x] C4: resolve `pimdir.account` as himalaya does (himalaya/src/pimdir/client.rs:225): derive it when the store holds one account, error naming the accounts when it holds several (src/pimdir/client.rs:64, src/pimdir/backend.rs:451-457) (done by pimdir-reads-one-account, 2026-10-30)
- [x] B3: add an `alpn` key to the `caldav` block, defaulting to `http/1.1` (src/caldav/client.rs:146); update config.sample.toml (done by the-shared-kit, 2026-11-12: `CaldavConfig::alpn`, `default_caldav_alpn`, config.sample.toml)
- [x] F1: rename `CalendulaCli` and `CalendulaCommand` to `Cli` and `Command` (src/cli.rs:58, :95, src/main.rs:162-211) (done by the-shared-kit, 2026-11-12)
- [x] F7: add `footer!` from pimalaya-cli to the clap command (done by the-shared-kit, 2026-11-12: src/cli.rs `after_help = footer!()`)
- [x] G2: build one `SecretResolver` per account where the runtime assembles its backends (src/shared/client.rs:533-540) (done by the-shared-kit, 2026-11-12: `CalendarClient` keeps one resolver)
- [x] A2: once released, drop the git pins (done by published-dependencies, 2026-11-12: every Pimalaya crate from crates.io)
- [x] H1, H2: once pimalaya-cli ships them, drop src/wizard/configure.rs:91, :220, :246 and src/cli.rs:141 (lapsed: src/wizard/ left with the-wizard-leaves, 2026-11-12, and `resolve_account` is a thin wrapper over pimalaya-config `take_account`, as in Himalaya and cardamum)
- [x] CHANGELOG under [Unreleased] (written by the-shared-kit)
- [x] Fold into the spec; log; land (folded and logged by the-shared-kit, 2026-11-12; folder archived 2026-11-19, status kept `abandoned`)
