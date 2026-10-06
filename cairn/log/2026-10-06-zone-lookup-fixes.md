---
cairn: log
change: zone-lookup-fixes
date: 2026-10-06
---

# Two zone lookups that misread an invitation

## What landed

- `named_zone` tries a `TZID`'s segments from the first, so `/Europe/Paris` resolves as `Europe/Paris`.
- `defined_zone`: a `VTIMEZONE` stating no observance defines nothing, the time read in the local zone with `zoneAssumed`.
- Tests `a_tzid_with_a_leading_slash_names_its_zone`, `a_vtimezone_without_observances_defines_nothing`.
- Capability moved: `commands` (zone resolution).

## Verification

`cargo test --all-features` and `cargo clippy --all-features --all-targets` clean.
