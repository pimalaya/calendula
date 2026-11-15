---
cairn: change
id: live-provider-tests
status: landed
created: 2026-11-12
---

# Live provider suites

## Why

Change 8 of the-family-standards. Himalaya (2026-11-11-live-provider-tests) and Cardamum (2026-11-12-live-provider-tests) run their built binary against the real providers, whose APIs and refusals drift without notice; calendula has no live suite, so a provider moving under the gcal, Graph or CalDAV backends, or a sample section going stale, shows only in a user's report. The suites come before capability-shared-api so they guard that a served verb prints what it printed.

## What

- tests/common.rs, Himalaya's and Cardamum's helpers renamed for calendula: `write_config`, `calendula` (the binary with `CALENDULA_TEST_TOKEN` in its environment, its home a TempDir, stdin closed), `failure`, `json`, `json_failure`, `json_check_failure`, `assert_check`, `assert_calendars`, `assert_events` (a failure naming the faulty member, never its value, a calendar and an event being personal data), `sample_section`, `env_var`, `require_env`, `env_content`. Every account reads its secret through `["printenv", "CALENDULA_TEST_TOKEN"]`, the accounts written from the sample's provider sections.
- tests/google.rs, tests/microsoft.rs and tests/fastmail.rs, every test ignored, run with `--ignored`:
  - `anonymous` (a made-up credential, no account touched): `account check` exits 2 with every backend failed and `calendar list` exits 1 with the failure object, the message naming `HTTP 401`. Google Calendar and Google CalDAV on their sample sections; Microsoft Graph on its section; Fastmail CalDAV on its section with a made-up Basic password for a user on a reserved domain, so no real Fastmail account is tried.
  - `credentialed` (read-only): `account check`, `calendar list` and `event list -s 5` on the primary or default calendar, one token minted per run with io-oauth (dev-dependency, `client`, `jwt-bearer`, `rustls-ring`, as Himalaya). Google Calendar with a service-account token acting as google@pimalaya.org for `calendar.events` and `calendar.calendarlist.readonly`, the narrowest scopes the delegation grants for a read (one scope it refuses fails the whole request, and full `calendar` is refused). Microsoft Graph with a client credentials token for `https://graph.microsoft.com/.default`, `msgraph.user-id` set to microsoft@pimalaya.onmicrosoft.com.
  - Google CalDAV credentialed only if Google accepts a token of those scopes, probed first; otherwise anonymous only, the refusal reported to the owner.
- .github/workflows/tests.yml: a `live` matrix job (google, microsoft, fastmail, `fail-fast: false`, `RUST_LOG: debug`), Himalaya's: the anonymous step on every push, the credentialed steps in pimalaya/calendula only and once their secrets exist, mapping `GOOGLE_SERVICE_ACCOUNT_KEY`, `MICROSOFT_TENANT_ID`, `MICROSOFT_CLIENT_ID` and `MICROSOFT_CLIENT_SECRET` to `CALENDULA_*`.
- Spec: "Live provider suites" (packaging), Himalaya's wording adapted.

## Scope / non-goals

No behaviour changes: the binary, its outputs and its configuration are untouched. No write of any kind reaches an account. Fastmail stays anonymous: only a browser consent issues a Fastmail token, and its CalDAV server takes an app password, which no test account holds. No CHANGELOG entry, nothing user-visible moving.
