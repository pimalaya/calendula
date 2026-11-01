---
cairn: log
change: a-to-do-refusal-names-the-way-out
landed: 2026-10-30
---

# A to-do refused by an events-only calendar names the way out

Since the io-pimdir pins of 2026-10-29 (1df2b70, then 8907303; commits f3a7330 and 84aa00c), a VTODO created or updated into a calendar whose source states `x-vtodo` `none` there or source-wide is refused before anything is queued. That landed with no CHANGELOG line, spec requirement or log entry; this entry records it. The refusal now reads `Stage the pimdir action: this calendar takes no to-do, source <source> states x-vtodo none (<reason>); store it in another calendar with -k/--calendar` instead of the store's generic `Source caldav does not support x-vtodo: events only`.

Test: `a_to_do_into_an_events_only_calendar_names_the_way_out`, a create and an update, which failed on the generic message.

Capabilities moved: **backends** (pimdir refuses a to-do where its calendar takes none, added).
