---
cairn: log
change: removed-keys-are-unknown-keys
landed: 2026-11-18
---

# Removed keys are unknown keys, and MIGRATION.md goes

Landed without a proposal: the owner's decision of 2026-11-14, restated 2026-11-16 (a key or a table calendula no longer reads fails through serde's `deny_unknown_fields` alone, no by-name refusal; no migration guide from 0.x to v1, users start over).

- src/config.rs: the two `removed_key!` blocks (`todo`, `journal`, `table`, `account`, `calendar.list`, `event.list.table`, `item.list.table`, `caldav.discover`, `vdir.home-dir`) and the fields carrying them are gone; each now fails as an unknown field. Test `a_key_an_earlier_calendula_read_is_an_unknown_one` replaces the five by-name tests; tests/account.rs `a_caldav_discover_key_is_an_unknown_one` and `a_vdir_home_dir_key_is_an_unknown_one_and_root_is_read` replace their by-name twins.
- MIGRATION.md is deleted, and the README link to it; the [Unreleased] CHANGELOG lines naming it say "fails as an unknown key", and the line announcing the family sentence is dropped, the sentence never having been released.
- Spec config: "Unknown fields" lists the old keys among the unknown ones, "A leftover todo or journal table is refused by name" is removed, "No colour or table is configured", "The vdir home is `vdir.root`" and "CalDAV locates its home-set two ways" read the old keys as unknown fields.

Capabilities moved: config.
