---
cairn: log
change: series-exceptions-follow-the-listed-range
landed: 2026-11-15
---

# Series exceptions follow the listed range

A msgraph series' exceptions are read within the range listed, or five years either side of today without one, in windows Graph accepts, fixing [calendula#31](https://github.com/pimalaya/calendula/issues/31): `event list` no longer fails whole on an open-ended series (Graph's 400 `Maximum number of years: 5`), and an open-ended series older than five years lists this week's exceptions. Capability **backends** moved.

- **Cargo.toml**: io-msgraph taken from ../io-msgraph under `[patch.crates-io]`, for `MsgraphPatternedRecurrence::instance_windows` (io-msgraph change instances-read-in-windows-graph-accepts), until io-msgraph 0.9.1 is released; Cargo.lock dropped the registry source and checksum of io-msgraph.
- **src/msgraph/backend.rs**: `exceptions` takes the days to read within and walks every window `instance_windows` cuts, then each window's pages. `list_items` reads within `query.range`, a day wider either side since a range's days are UTC and an event's those of its zone, a side left open reaching five years past today (`HORIZON_YEARS`); `get_item`, and `update_item` for its base, read within today ± five years, so the base matches what a read gave. `series_window` and `OPEN_SERIES_YEARS` are gone. A refused window fails the reread, which the listing already turns into an item error coded by the refusal.
- **Evidence**: three scripted Graph tests: an open-ended series started 320 weeks ago lists this week's occurrence moved from Monday to Tuesday, its instances read over a window holding this week; no `instances` request of an open-ended or a ten-year bounded series spans five years; a refused second window lists the lone event beside the series' `permission-denied` entry. Each fails against the old window.
- **Live**: the read-only credentialed suite of tests/microsoft.rs passes on the test tenant (3 tests, `event list` included). The check with an open-ended series older than five years is not done: the suite writes nothing, and seeding such a series would be a write no live test makes.
- **Not done**: that live check; the reply on #31.

Open question, not checked on Graph: whether `instances` returns an occurrence moved out of the range read; if it does not, the series shows it at its original slot within the range. Spec: backends ("Microsoft Graph backend").
