---
cairn: tasks
change: gcal-vtimezone
---

# Tasks

- [x] Cargo.toml: tz-rs and tzdb behind the `gcal` feature.
- [x] src/gcal/timezone.rs: an IANA name to the VTIMEZONE it denotes around an anchor, the closing POSIX rule past the end of the record and the bracketing transitions before it, a settled zone as one observance, and the name test the stash decision reads.
- [x] src/gcal/project.rs: the undefined zones read off the finished document with folds resolved, one definition each spliced ahead of the VEVENT, and `calendar_remainder` stops stashing one that can be rebuilt.
- [x] Tests: the generated zone resolves through ical-rs to the offset the database puts in force, swept across twelve zones and six years spanning half a century; the spring gap and the autumn fold; the era either side of the 2007 rule change; a zone that gave daylight saving up; the half-hour and fixed shapes; an unknown name mints nothing.
- [x] Tests: a Google-shaped event arrives with its definition, ahead of the VEVENT and only once; a zone named by a stashed EXDATE or across a fold is defined too; a known zone leaves the stash and an unknown one stays in it.
- [x] The CHANGELOG entry, folded into the unreleased gcal backend rather than filed as a fix, since that backend has not shipped.
- [x] Fold the delta into [cairn/spec/projection.md](../../spec/projection.md) and [cairn/spec/packaging.md](../../spec/packaging.md); write [cairn/log/2026-08-10-gcal-vtimezone.md](../../log/2026-08-10-gcal-vtimezone.md).
