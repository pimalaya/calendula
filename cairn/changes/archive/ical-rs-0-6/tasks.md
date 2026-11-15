---
cairn: tasks
change: ical-rs-0-6
---

# Tasks

- [x] Bump ical-rs to 0.6, io-gcal to 0.2, io-msgraph to 0.5, io-pimdir to 0.7 from crates.io; one ical-rs and one io-pimdir in `Cargo.lock`.
- [x] Remove `is_spurious` (`src/shared/ical.rs`).
- [x] Remove `folded` and `FOLD_OCTETS` (`src/shared/event/fields.rs`), the encoder folding.
- [x] `--duration` through the validator; tests on `PT1H5S`, `PT1.5S`, lower case and a leading `+`.
- [x] Tests: a multibyte line folded between characters, a parameter and a duration the RFC refuses caught by the check.
- [x] Expansion on the zone-aware set (`of_component_in`, `with_override_in`, `IcalRecurOverride::of_component`), `localize_until` removed; tests on a UTC `UNTIL`, `EXDATE` and `RECURRENCE-ID` on a zoned series.
- [x] The four broken intra-doc links.
- [x] Fold into the spec, log, changelog.
