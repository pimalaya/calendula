---
cairn: log
change: agenda-reads-a-lone-month
landed: 2026-11-01
---

# The agenda reads a lone month name, and refuses a date it cannot read

`EventAgendaCommand::execute` in src/shared/event/agenda.rs reads one non-numeric argument as a month of the current year, as cal does, today highlighted, and `parse_day`, `parse_month` and `parse_year` return an error naming the value where they used to fall back to the 1st, to January and to the current year.

Verification: `cargo test`, `cargo clippy --all-targets --all-features` and `cargo fmt --check`; checked by hand against a Google calendar in `es_ES.UTF-8` and `C`.

Capabilities moved: **commands** (an agenda reads its date as cal does, added).
