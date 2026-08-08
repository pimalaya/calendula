---
cairn: log
change: drop-io-calendar
landed: 2026-08-08
---

# Drop io-calendar, own the cross-protocol layer

Removed the io-calendar dependency and rebuilt the cross-protocol layer inside calendula, following the cardamum precedent. The shared types (`Calendar`, `CalendarDiff` beside the calendar commands; `CalendarItem`, `CalendarTimeRange` beside the item ones) and the dispatcher (`shared::client::CalendarClient`, an enum over exactly one backend) are now calendula's own, with one `src/<proto>/backend.rs` adapter per protocol.

This was forced rather than chosen: io-calendar is frozen and still pinned io-vdir 0.0.3 and io-webdav 0.0.1, so nothing below it could move while it stayed. The repository did not resolve against its own upstreams either, since the lockfile pinned old git commits while the patch entries followed HEAD.

Adopted ical-rs for iCalendar parsing. io-calendar's `as_ical` went with the crate and io-vdir 0.1 had removed its own parser feature, so both consumers of the old helper (the event listing and the agenda) now go through one `Event::project`, which is also where the malformed-item policy lives: an item whose bytes do not parse projects nothing instead of failing the listing.

Bumped everything else to its current release: io-vdir 0.1, io-http 0.3, pimalaya-cli 0.2, pimalaya-config 0.1.1, pimalaya-stream 0.1.2, and pimconf to its renamed successor io-pim-discovery 0.5. io-webdav, io-pimdir and io-replica stay on git patches while their needed changes ride unreleased, each with a note saying what it waits on. The direct comfy-table dependency is gone: the toolkit re-exports it, and `shared/table.rs` maps the v7 preset string a configuration carries onto the v8 typed style, so existing configurations stay valid.

Along the way, fixed pimalaya-cli: `wizard::keyring` gated every item on the mail features, so a caldav-only consumer compiled the module and found it empty. The module declaration is now the only gate.

Verified: every backend feature combination builds (vdir, pimdir, vdir+pimdir, caldav, and each pair with caldav), fmt and clippy clean across all of them, 43 tests pass.

Spec updated: backends (ADDED the shared operation set, the selection order, the CalDAV and vdir requirements, the CalDAV time-range push-down), packaging (ADDED released dependencies, no aggregator).
