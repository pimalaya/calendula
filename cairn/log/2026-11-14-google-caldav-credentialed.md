---
cairn: log
change: google-caldav-credentialed
landed: 2026-11-14
---

# Google CalDAV is tested credentialed

tests/google.rs reads Google's CalDAV server with a real token, now that the test Workspace's delegation grants the full `https://www.googleapis.com/auth/calendar` scope (owner, 2026-11-14). Capability **packaging** moved.

- **tests/google.rs**: `mint(scopes)` signs the RFC 7523 assertion and requests the token, shared by `token` (the Calendar API's narrow `SCOPES`, unchanged) and `caldav_token` (`CALDAV_SCOPES`, the full `calendar` scope alone), each minted once per run. Three `credentialed` tests on the sample's Google CalDAV section, ignored like the others: `caldav_account_check`, `caldav_calendar_list` (the primary calendar listed) and `caldav_event_list` (`event list -s 5 -k events`). The primary calendar's CalDAV id is `events`: it is the one collection directly under the home set Google gives the user, which io-webdav names by its last segment, the two other calendars of the account being named by their path. The module header no longer says CalDAV is covered anonymously only.
- **Evidence**: `CALENDULA_GOOGLE_SERVICE_ACCOUNT_KEY_FILE=<key path> nix develop --command cargo test --all-features --test google -- --ignored` against the test Workspace, read-only: 10 passed (4 `anonymous`, 3 `gcal_*`, 3 `caldav_*`). A first run looking for the subject's address in the CalDAV ids found none among the 3 calendars, which is how the `events` id was learnt.

CONTRIBUTING.md and the packaging spec ("Live provider suites") say Google's CalDAV tests mint a token of the full `calendar` scope. The CI step running `credentialed::` picks the new tests up with no workflow change.

Verification, the three changes together: `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` over the eleven feature sets of the CI `features` job (359 tests under `--all-features`, 182 with no backend); `cargo deny check` (advisories, bans, licences, sources ok); `sh cairn/verify.sh` conformant.
