---
cairn: tasks
change: a-backend-left-out-is-named
---

# Tasks

- [x] `AccountConfig::blocks`, `backends`, `not_built`; the `IgnoredAny` fields lose `allow(dead_code)`.
- [x] `backend::ALL`, `backend::not_built`, `FromStr` naming the feature.
- [x] `resolve_backend` returns the left-out backend as its error; `CalendarClient::new` maps it to `error::backend_not_built`.
- [x] `account check`, `account capability`, `account list`.
- [x] `ErrorCode::BackendNotBuilt`.
- [x] Offline tests through the binary (tests/account.rs), run in a reduced build: a gcal block in a build without gcal (`calendar list`, `account check`, `--backend gcal`, a typo beside it, `account list`, `account capability`), and `auto` naming `--backend msgraph` in a msgraph-only build, `account capability` included.
- [x] CHANGELOG, MIGRATION.md, the src/main.rs header, the `--backend` help.
- [x] Fold the delta, log, set `landed`.
