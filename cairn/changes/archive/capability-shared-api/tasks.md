---
cairn: tasks
change: capability-shared-api
---

# Tasks

- [x] src/capability.rs: the fifteen names, the levels, the five tables, `require`, the help footer, unit tests
- [x] `capability-missing` in src/error.rs
- [x] Every unified command requires its capabilities before connecting; `SendUpdatesArg::check_backend` and the online-meeting check of `EventFieldsArgs::check_backend` go
- [x] `account capability` and its schema
- [x] The `Backends:` footer on every unified leaf but `build`
- [x] tests/capability.rs, seen failing on the old code
- [x] The src/main.rs, src/shared/mod.rs and src/shared/client.rs headers, the `--backend` help, README, CHANGELOG
- [x] Clippy over the feature matrix, `cargo test --all-features`, `json-schema -d` compared, the anonymous live suites
- [x] Fold into the spec, log, land
