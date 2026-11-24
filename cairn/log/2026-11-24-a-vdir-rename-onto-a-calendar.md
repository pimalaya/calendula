---
cairn: log
change: the-live-night-findings
landed: 2026-11-24
---

# A vdir rename onto a calendar

C3 of the-live-night-findings, cardamum D5's bug (cardamum 35b7d15).

- `calendula --json vdir rename work full`, `full` holding an event, failed `{"code":null,"error":"Directory not empty (os error 39)"}`; `vdir rename work empty`, `empty` an empty directory, exited 0 and replaced it.
- `vdir rename` now refuses a new id that is one directory name the home holds (any entry) before anything moves, coded `calendar-duplicate`, ``Cannot rename calendar `<id>` to `<new-id>`: it exists already`` (src/vdir/client.rs `check_rename_target_absent`). A new id that is not one directory name is still io-vdir's to refuse.
- Pinned by tests/calendar_duplicate.rs `a_calendar_renamed_onto_another_is_a_duplicate`, failing before the fix with `"code":null`, `"error":"Directory not empty (os error 39)"`.

Capabilities moved: backends ("A calendar created twice is a duplicate", new scenario "A vdir calendar renamed onto another", the `calendar-duplicate` row), commands (the `calendar-duplicate` row).
