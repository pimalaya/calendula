---
cairn: change
id: google-caldav-credentialed
status: landed
created: 2026-11-14
---

# Google CalDAV is tested credentialed

## Why

tests/google.rs covered Google CalDAV with a made-up token only: Google's CalDAV server refuses a token of the narrow scopes the test Workspace's delegation granted (`calendar.events`, `calendar.calendarlist.readonly`), answering 403. The owner granted the full `https://www.googleapis.com/auth/calendar` scope to that delegation on 2026-11-14, and checked that a delegated token for google@pimalaya.org gets 207 on the principal PROPFIND and on the primary `events/` collection.

## What

- tests/google.rs mints a second token per run, of the full `calendar` scope alone (`caldav_token`), the Calendar API tests keeping their narrow `SCOPES`; minting is one `mint(scopes)` both share.
- Three read-only `credentialed` tests on the sample's Google CalDAV section: `caldav_account_check`, `caldav_calendar_list` (the primary calendar listed) and `caldav_event_list` (five events of the primary calendar). The primary calendar's CalDAV id is `events`: it is the one collection directly under the home set Google gives the user, which io-webdav names by its last segment, the others by their path.
- The module header, CONTRIBUTING.md and the packaging spec ("Live provider suites").
