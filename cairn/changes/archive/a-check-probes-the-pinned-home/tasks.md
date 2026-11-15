---
cairn: tasks
change: a-check-probes-the-pinned-home
---

# Tasks

- [x] Test (`account::check::caldav_tests`): a local HTTP server answering 401 fails the `home` row with the `server` row's code and error, on one PROPFIND Depth 0 on the home; a home answering 207 passes. The first failed on the old check (`ok: true`), the second too (no request sent).
- [x] `check_caldav` sends the PROPFIND on `home`, `connect` unchanged.
- [x] The `account check` help.
- [x] CHANGELOG Fixed line.
- [x] Fold the delta into [backends](../../../spec/backends.md); log.
