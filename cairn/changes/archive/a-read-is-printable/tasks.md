---
cairn: tasks
change: a-read-is-printable
---

# Tasks

- [x] tests/read.rs: a vdir event stored with an OSC 52 write, a bidi override, a tab fold and CRLF, read by `event read` and `item read`: defused in text (seen failing on the raw view, both reads), the stored text under `--json`.
- [x] `printable` (src/shared/item/read.rs) in both reads' text view; their help.
- [x] Sweep of the other text outputs; the index zone and coverage days under `pimdir search` and `pimdir count` defused, `the_lines_under_a_table_defuse_what_the_store_holds` seen failing first.
- [x] fmt; clippy `-D warnings` default and `--all-features`; `cargo test --all-features`; the schemas carry no changed text.
- [x] Fold the delta into the spec (commands); CHANGELOG; log; tick the-family-standards; `sh cairn/verify.sh`.
