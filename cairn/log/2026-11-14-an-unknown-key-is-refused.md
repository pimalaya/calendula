---
cairn: log
change: an-unknown-key-is-refused
landed: 2026-11-14
---

# An unknown key is refused

A configuration key calendula does not know fails the load at every level (owner decision, 2026-11-14), the root and the accounts included, which tolerated any key so a reduced build would load a file naming a backend it leaves out. Following Cardamum (8f25abf). Capability **config** moved.

- **src/config.rs**: `deny_unknown_fields` on `Config` and `AccountConfig`, every other table carrying it already; no table flattens another, so none needed restructuring. The block of a backend the build leaves out is an `IgnoredAny` field of `AccountConfig` under `cfg(not(feature))` (`vdir`, `pimdir`, `caldav`, `gcal`, `msgraph`; calendula has no `proxy` key), so a file serving a full build still loads in a reduced one, its block unread. The colour tables 0.2 read, refused until now as unknown fields of their block, are refused by name like `todo`, `journal`, `table` and the top-level `account`: `CalendarListRemovedConfig` (`calendar.list`), `EventListTableRemovedConfig` (`event.list.table`), `ItemListTableRemovedConfig` (`item.list.table`), at the root and in an account. No backend key changed since v0.2.0, so no other removed key needed a name.
- **Evidence**: `an_unknown_root_key_is_refused`, `an_unknown_account_key_is_refused` and `a_colour_table_is_refused_by_name` (which replaces `a_colour_key_is_refused`, the generic refusal it asserted being gone) fail on the old code; `an_unknown_backend_or_nested_key_is_refused` (`vdir.home-dri`, `caldav.tls.certificate`, `caldav.auth.basic.usernme`, the last reported by toml as "unexpected keys in table", `event.list.page-sise`) passes on it, those tables refusing already. `a_backend_left_out_of_the_build_still_loads` (a block of every backend) and `the_sample_configuration_parses_with_each_variant`, which now loads every section in every build where it skipped those of a backend left out, run in every build of the CI matrix.

CHANGELOG (**Breaking:** the strict root and accounts; the Removed entry naming the colour tables refused by name), MIGRATION.md (the colour tables, the unknown keys), the src/main.rs header. Spec: config ("Unknown fields", "A leftover todo or journal table is refused by name", "No colour or table is configured", the sample's "A build without a backend" scenario).

Verification, the three changes together: `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` over the eleven feature sets of the CI `features` job (359 tests under `--all-features`, 182 with no backend); `cargo deny check` (advisories, bans, licences, sources ok); `sh cairn/verify.sh` conformant.
