---
cairn: tasks
change: io-webdav-0-11
---

# Tasks

- [x] io-webdav 0.11.0 in Cargo.lock (that crate alone) and Cargo.toml (`0.11.0`); no `href` literal to adapt.
- [x] src/caldav/report/raw.rs: the REPORT through `client.run`, `run_report` gone.
- [x] src/account/check.rs: `check_caldav`'s PROPFIND through `client.run`.
- [x] Tests: `a_raw_report_is_sent_again_on_a_fresh_connection` (src/caldav/report/raw.rs, the scripted server closed before the REPORT, a fresh one answering), `a_refused_raw_report_is_coded` (a 403 coded `permission-denied`), `a_home_propfind_is_sent_again_on_a_fresh_connection` (src/account/check.rs, a local server closing its first connection unanswered). All three failed on the hand-run code over io-webdav 0.11 (`HTTP/1.1 send failed: reached unexpected EOF` twice, the 403 uncoded).
- [x] CHANGELOG (Changed: the dependency line; Fixed).
- [x] Fold the delta into backends.md, write the log entry, set `landed`.
