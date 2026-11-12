---
cairn: tasks
change: live-provider-tests
---

# Tasks

- [x] Cargo.toml: io-oauth 0.5.2 as a dev-dependency (`client`, `jwt-bearer`, `rustls-ring`), as Himalaya's and Cardamum's
- [x] tests/common.rs: the shared helpers, the token read through `["printenv", "CALENDULA_TEST_TOKEN"]`
- [x] tests/google.rs: `anonymous` (gcal and CalDAV, `account check` and `calendar list`), `credentialed` (gcal `account check`, `calendar list` holding the primary, `event list -s 5` on it)
- [x] Google CalDAV probed with a token of `calendar.events` and `calendar.calendarlist.readonly`, then `calendar.events` alone: both refused, so CalDAV stays anonymous
- [x] tests/microsoft.rs: `anonymous` (Graph `account check` and `calendar list`), `credentialed` (`account check`, `calendar list` holding `Calendar`, `event list -s 5` on it)
- [x] tests/fastmail.rs: `anonymous` (CalDAV `account check` and `calendar list`, a made-up Basic password for a reserved-domain user)
- [x] .github/workflows/tests.yml: the `live` job; actionlint
- [x] Run from this machine: anonymous and credentialed, then one debug run with each binary's stderr saved and grepped for the minted tokens, the client secret, `ya29.` and a JWT
- [x] `cargo deny check`
- [x] fmt; clippy `-D warnings` default and `--all-features`; `cargo test --all-features`
- [x] Fold the delta into cairn/spec/packaging.md; log; tick the-family-standards; status landed; `sh cairn/verify.sh`
