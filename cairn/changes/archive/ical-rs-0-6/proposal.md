---
cairn: change
id: ical-rs-0-6
status: landed
created: 2026-10-09
---

# ical-rs 0.6, and the workarounds it retires

## Why

The field flags (`event-field-flags`) found four places where ical-rs 0.5.3 was narrower or laxer than RFC 5545, and calendula worked around each: `is_spurious` dropped the validator's findings on a `DATE` or a `TZID` on the time properties and on the `ORGANIZER` parameters, `folded` re-folded every encoded line at 75 octets and read it back through `push_raw`, and `duration` held `--duration` to the 3.3.6 grammar by hand. ical-rs 0.6 (its `rfc5545-contract-fixes`) fixes all four upstream, so the workarounds go.

io-gcal 0.2 and io-msgraph 0.5 are the releases on ical-rs 0.6, needed for one ical-rs in the build. io-pimdir 0.7 is the git revision calendula pinned, released on crates.io.

## What

- ical-rs 0.6, io-gcal 0.2, io-msgraph 0.5, io-pimdir 0.7 (crates.io, no git pin).
- `check` reports every validator finding; `is_spurious` is removed.
- Encoded lines are written as `IcalItem::Prop` and folded by ical-rs's encoder; `folded` is removed.
- `--duration` is checked through ical-rs's validator (`IcalValidateError::Duration`) after upper-casing and dropping a leading `+`; a negative one stays refused, by calendula.
- Expansion tells every time on the series' clock through ical-rs's zone-aware set, with a zone per `TZID` as calendula resolves it; `localize_until`, which did it for `UNTIL` alone, is removed, and a UTC `EXDATE` or `RECURRENCE-ID` on a zoned series now names its instance.
