---
cairn: log
change: the-releases-land
landed: 2026-11-18
---

# The releases of 2026-11-18 land

calendula moved onto io-jmap 0.10.1, io-pimdir 0.17.0 (pimdir draft-14), pimalaya-cli 0.6.1 and pimalaya-config 0.6.0, all from crates.io, and took up what they carry (cairn/changes/archive/the-releases-land).

- **jmap**: the calendar writes go through io-jmap's `Calendar/set`, its per-object errors coded as before (`calendarHasEvent` now named by io-jmap), and the read conversion is `JmapCalendarEvent::to_ical`; src/jmap/calendar_set.rs and `project::to_ical` are gone. The tick of jmap-backend's io-jmap line.
- **C1** (pimdir audit-2026-11-12): `store-version` reads in io-pimdir's `PimdirFormatRefusal`, naming calendula. calendula's own wording parsed `storage-draft-N` on both sides, so a draft store under a `storage-v1` calendula read as a newer engine's; test `a_store_of_another_version_names_the_drain_or_the_upgrade` takes that case.
- **C2**: an iCalendar a raw `item create` or `item update` is given is queued without its `X-PIMDIR-` properties (io-pimdir's stripping `enqueue_with_body`) and no longer gated for an online meeting; the `event` commands and an item composed with `-i` from the blank or the stored item queue through `enqueue_user_body`. The stated departure from STORAGE §18 is gone. Test `a_foreign_body_is_queued_without_its_instructions`, which fails with the foreign body queued whole.
- **C4**: `pimdir reply` and `pimdir cancel` print `handle` (`<store id>.<incarnation>.<row>`), and `pimdir queue show` and `pimdir queue cancel` take it or a bare row id; a handle of another store or incarnation reads `reset`, refused `store-reset` by `cancel`, nothing dropped. Test `a_handle_of_another_store_is_refused_as_a_reset`.
- **CAL-13, composer half**: the draft is created through `file::private_options` with `create_new`, 0600 on Unix, a planted file or link failing the command. Test `a_draft_is_private_and_never_written_through_a_planted_path`.

Moved: backends.md (jmap, pimdir online meeting, store failure codes, intents, queue), commands.md (`store-version` row, composer), projection.md.

Left: `pimdir queue list` and `pimdir status` print bare row ids, io-pimdir 0.17 exposing no store identity but in an enqueue's answer; ical-rs's `UNTIL` shift (jmap-backend) is unreleased.
