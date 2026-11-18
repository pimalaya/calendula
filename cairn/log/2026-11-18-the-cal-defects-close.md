---
cairn: log
change: the-cal-defects-close
landed: 2026-11-18
---

# The CAL defects close, first batch

Landed without a proposal: defects of the deep analysis of 2026-11-15 (~/.claude/plans/pimalaya-cli-family-analysis-2026-11-15/plan.md, section 4.4, verdicts of 2026-11-16), each checked on the head before its fix, plus one owner ruling of 2026-11-18.

- **CAL-7, ruled 2026-11-18 (a local day), and CAL-1** (7842252): `CalendarTimeRange::from_days` builds its bounds from the local midnights, `from_days_in` from those of a given zone, and `contains` tests a UTC time as an instant and a date or a local time by the local days of the bounds. The agenda fetched its grid's days as UTC midnights and filed the events by local days, so the evening of its last day west of UTC and the morning of its first day east of it were dropped with exit 0. Reproduced at fac7873 on a vdir calendar: with `TZ=America/New_York` an event at 20:00 on 2026-11-30 was absent from `event list --from 2026-11-30 --to 2026-11-30` and from `event agenda 11 2026`, with `TZ=Asia/Tokyo` one at 07:00 on 2026-11-01 from the November agenda; all three listed after. `caldav report query`, `gcal free-busy` and `gcal events instances` read their `--from` and `--to` the same way. Tests: `a_day_range_spans_the_local_days`, `containment_tells_a_utc_time_as_an_instant_and_a_local_one_by_its_day`.
- **CAL-2, the CalDAV half of TWN-1** (34771b5): `update_item` given no version sends `If-Match: *`, its 412 coded `item-not-found`. Test `an_update_of_a_missing_item_creates_nothing` (src/caldav/backend.rs).
- **CAL-4 and the guard half of CAL-13** (4130220): `calendar_delete` under `--allow-write` alone is refused while an item of the calendar names an attendee or cannot be read, and `invites` reads every VCALENDAR of a body through `IcalCst::parse_many`, a body holding none counting as naming one. Tests `a_calendar_holding_an_invitation_needs_allow_send_to_go` (tests/mcp.rs) and the second-VCALENDAR case of `an_item_invites_whoever_it_names_as_an_attendee`.
- **CAL-10** (da42cd2): `--end` without `--start` is held to the stored `DTSTART` (`stored_start`), a date against a time or an end before the start refused. Test `an_end_alone_is_checked_against_the_stored_start`.
- **CAL-11** (a2e4d69): on Google, a cleared colour is refused before the rename, and a failed paint after `calendar create` names the calendar created. Tests `a_cleared_colour_is_refused_before_the_rename`, `a_failed_paint_names_the_calendar_created`.

Left: CAL-13's composer half (the draft is written with `fs::write` in the shared temporary directory) waits on pimalaya-cli's `file::private_options` (374eddd, unreleased), the kit's private-create helper of KIT-4, rather than a copy in calendula. A partially applied Google create still fails with exit 1; the result with exit 3 of the COH-5 ruling of 2026-11-18 comes with the family's shape.

Capabilities moved: commands, mcp, backends.
