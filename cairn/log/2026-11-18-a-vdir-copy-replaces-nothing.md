---
cairn: log
change: a-vdir-copy-replaces-nothing
landed: 2026-11-18
---

# A vdir copy replaces nothing

Landed without a proposal: CAR-1 of the deep analysis in calendula, as cardamum landed it (5b5c3ec, `check_item_absent`).

io-vdir's move and copy replace the target's file of the same id. `vdir item move` and `copy` now locate the id in the target first (`vdir::client::check_item_absent`) and fail `item-duplicate` when it is there, a `NotFound` letting them on. Reproduced at fac7873: `vdir item copy home work e1` over a `work/e1.ics` exited 0 and left the target holding the source's bytes. Test `an_item_moved_or_copied_onto_its_id_is_a_duplicate` (tests/calendar_duplicate.rs).

Capabilities moved: commands.
