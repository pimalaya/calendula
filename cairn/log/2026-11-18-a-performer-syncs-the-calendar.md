---
cairn: log
change: a-performer-syncs-the-calendar
landed: 2026-11-18
---

# A performer syncs the calendar

Landed without a proposal: CAL-17 of the deep analysis, its queueing half and its text receipt.

- In a store whose sources declare no capability, `pimdir reply` and `pimdir cancel` took any `--source` as the performer, and neverest skips a row naming an unknown source for ever, neither sent nor parked (neverest/src/offline/driver.rs, as the analysis read it). The chosen source must now be one syncing the calendar, else `no-performer`, naming those that do. Test `a_chosen_source_syncing_nothing_of_the_calendar_is_refused`.
- The text receipt reads ``Reply <PARTSTAT> to event `<id>` queued as row <row>; the next sync sends it``, the row `pimdir queue show` and `cancel` take, as the JSON already carried.

Left: `pimdir queue list` and `show` still print no performer, answer nor occurrence for an intent, so two queued replies to one event read alike. The row is the family's shared `PimdirQueueRow` (rulings D19 and D21, held by pimalaya-cli's spec), so a column for them is a kit change first.

Capabilities moved: backends, commands.
