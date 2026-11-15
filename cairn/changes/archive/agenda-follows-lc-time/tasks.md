---
cairn: tasks
change: agenda-follows-lc-time
---

# Tasks

- [x] Resolve the time locale from `LC_ALL`, `LC_TIME` and `LANG` (`time_locale`, `parse_locale`), falling back to POSIX.
- [x] Replace `FULL_MONTH` and `WEEKDAYS` with `Names`, built from chrono's locale tables, and carry it in `CalControl`.
- [x] Stamp the events in the locale (`EventAgendaOutput::with_locale`).
- [x] Read a month argument in English or in the locale.
- [x] Centre and pad by terminal columns (`unicode-width`), not bytes or characters; cut the weekdays to two columns.
- [x] Tests: locale parsing and fallback, POSIX, Spanish and Japanese names, month parsing, centring an accented name, a Japanese month keeping its columns, cutting a weekday to two columns, a Spanish stamp.
- [x] Checked against a Google calendar with `es_ES.UTF-8`: one month, `-3`, `-y` and a month by name; and with `LANG=C`, unchanged.
- [x] `cargo test`, `cargo clippy --all-targets --all-features`, `cargo fmt --check`.
- [x] CHANGELOG: one `### Changed` entry.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
