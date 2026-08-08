---
cairn: change
id: drop-io-calendar
status: landed
created: 2026-08-08
---

# Drop io-calendar, own the cross-protocol layer

## Why

calendula's shared client was io-calendar's `CalendarClientStd`. io-calendar is frozen: the org retired the per-domain aggregator crates on 2026-07-09, and it still pins io-vdir 0.0.3 and io-webdav 0.0.1 while both are at 0.1.0. Every other dependency was equally far behind, several by two major versions, and the repository could not even resolve against its own upstreams: the lockfile pinned old git commits while the patch entries followed HEAD, whose versions no longer satisfied the declared requirements.

So the bump is not a bump. Keeping io-calendar means keeping every dependency below it frozen too, and the fixes waiting upstream are not cosmetic: io-webdav's verbatim resource ids repair item read, update and delete addressing the wrong resource whenever an id does not end in `.ics`, its create returns a usable id when the server names the resource itself, and a calendar's time zone stopped being silently dropped on write.

## What

calendula owns its own least-common-denominator types and its own dispatcher, following the cardamum precedent: aggregation in the product, protocols as libraries. `Calendar` and `CalendarDiff` live beside the calendar commands, `CalendarItem` and `CalendarTimeRange` beside the item ones, and `CalendarClient` is an enum over exactly one backend, each adapted in its own `src/<proto>/backend.rs`.

iCalendar parsing moves to ical-rs, the org's own library: io-calendar's calcard-backed helper goes with it, and io-vdir 0.1 removed its equivalent, so there was no fallback either way. The VEVENT projection lands in one place instead of being repeated in the listing and the agenda.

Every remaining dependency moves to its current release: io-vdir 0.1, io-webdav (unreleased, git), io-http 0.3, pimalaya-cli 0.2, pimalaya-config 0.1.1, pimalaya-stream 0.1.2, and pimconf becomes io-pim-discovery 0.5 (the crate was renamed and every public type gained a `Discovery` prefix). comfy-table v8 arrives with pimalaya-cli and drops the positional preset string, so the config option keeps its v7 spelling through a mapper.
