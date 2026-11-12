---
cairn: delta
change: live-provider-tests
---

# Delta

## ADDED Requirements

### Requirement: Live provider suites
(packaging) The binary SHALL carry one ignored integration suite per provider under tests/ (google.rs for the Google Calendar API and Google CalDAV, microsoft.rs for Microsoft Graph, fastmail.rs for Fastmail CalDAV), run with `--ignored` and sharing tests/common.rs. Each SHALL hold an `anonymous` module that runs every backend the provider's sample sections offer on a made-up credential and touches no account, its accounts taken from the provider sections of config.sample.toml so a provider drifting from the sample fails it, and asserting the exit code of the output contract (2 with the `account check` report, 1 with the failure object) and a message naming the refusal in calendula's words (`HTTP 401`). It MAY hold a `credentialed` module that mints its own token from `CALENDULA_`-prefixed variables and only reads (`account check`, `calendar list`, `event list`), failing naming the variables it misses, which CI runs in the upstream repository only and once its secrets exist; Google's SHALL ask `calendar.events` and `calendar.calendarlist.readonly` alone, and so covers the Google Calendar API only. A token or password SHALL reach the binary through a command reading its environment, never argv nor the configuration, and no assertion SHALL print a token, a secret or an event's value. CI SHALL run each suite in its own job at `RUST_LOG=debug` at most.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
