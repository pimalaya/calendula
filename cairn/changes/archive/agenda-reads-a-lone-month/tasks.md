---
cairn: tasks
change: agenda-reads-a-lone-month
---

# Tasks

- [x] Read one non-numeric argument as a month of the current year, highlighting today as the no-argument arm does.
- [x] `parse_day`, `parse_month` and `parse_year` return an error naming the value instead of falling back.
- [x] Say it in the `DATE` help.
- [x] Tests: an unreadable day and a day outside its month are refused, an unknown month and a month out of range are refused, an unreadable year is refused.
- [x] Checked against a Google calendar with `es_ES.UTF-8`: `noviembre`, `nov`, `-3 diciembre`, `noviembre 2026`, `2027`, `1 2027`, and the errors for `novembre`, `13 2026`, `noviembre 2O26`; with `LANG=C`, `november`.
- [x] `cargo test`, `cargo clippy --all-targets --all-features`, `cargo fmt --check`.
- [x] CHANGELOG: one `### Fixed` entry.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
