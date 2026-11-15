---
cairn: tasks
id: ical-composer-and-build
---

# Tasks

- [x] `shared/uuid.rs`: version 4 UUIDs, for the `UID` a minted item carries
- [x] `shared/ical.rs`: `read_source`, `check`, `blank_item`, and the family-to-kind mapping
- [x] `shared/composer.rs`: `IcalComposer` and `IcalDraft`
- [x] `shared/arg.rs`: `IcalComposerArgs`, the `-i` / `--composer` pair
- [x] `shared/build.rs`: the build pipeline and its output
- [x] `config.rs` + `account/context.rs`: `item.composer` and its resolver
- [x] `shared/client.rs`: open on first use, `disconnect` before a composer
- [x] `cli.rs`: `resolve_account` public, families dispatching per subcommand
- [x] four families: `build`, `-i` on `create` and `update`, named outputs
- [x] `json_schema.rs`: the twelve new entries
- [x] docs: `config.sample.toml`, `README.md`, `CHANGELOG.md`
- [x] fold the delta into the spec and write the log entry
