---
cairn: tasks
change: agenda-expands-recurring-series
---

# Tasks

- [x] Extract the first drawn month from `monthly` into `first_month`, and derive the drawn days from it (`drawn_days`).
- [x] Fetch the events once the grid is settled, over the drawn days, and expand them with `Event::occurrences`.
- [x] Tests: one month, three months spanning a new year, a whole year.
- [x] Checked against a Google calendar: the yearly birthdays now mark their days in `event agenda -3` and `event agenda 2027`.
- [x] `cargo test`, `cargo clippy --all-targets`, `cargo fmt`.
- [x] CHANGELOG: one `### Fixed` entry.
- [x] Fold delta.md into cairn/spec/commands.md; append the log entry; mark `landed`.
