---
cairn: tasks
change: secret-resolver-per-account
---

# Tasks

- [x] Bump pimalaya-config to 0.2
- [x] The wizard builds a `CommandConfig` instead of a `std::process::Command`
- [x] `check_account` builds one `SecretResolver` and threads it into the CalDAV and gcal checks
- [x] Keep a resolver-less entry point beside each threaded one, for a caller opening a single backend
- [x] Fold the delta, log
