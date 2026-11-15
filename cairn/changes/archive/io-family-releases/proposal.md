---
cairn: change
id: io-family-releases
status: landed
created: 2026-11-13
---

# Build on the io-* releases of 2026-11-13

## Why

The family libraries calendula builds on were released together on 2026-11-13, every one a breaking release: io-http 0.7, io-oauth 0.6, io-gcal 0.4, io-msgraph 0.8, io-webdav 0.8, io-vdir 0.3 and io-pim-discovery 0.12. They close defects a family audit found in the libraries, some of which only close once the application follows: an id encoded as one path segment, a stash that says what it leaves out, a paging link checked against its origin, a value outside an enum kept rather than failing a page. One io-http has to remain in the tree, so the seven move in one step.

## What

- Cargo.toml requires the seven releases; the pimalaya-cli pin stays as it is, and io-pimdir, pimalaya-config and pimalaya-stream, not released that day, stay where they were.
- gcal: the enums read from the server gained `Unknown(String)` and are no longer `Copy`; the listings print such a value as the API spells it. `GcalEvent::from_ical` can answer `StashFull`: the event is written without the lines left out, a warning naming them, as the projection spec already ruled for an oversized line. Ids go to io-gcal as they stand, which a scripted test pins.
- msgraph: the calendar listing follows its pages through `MsgraphClientStd::page_get`, which refuses a link off the Graph origin, where it built the request by hand.
- CalDAV, vdir, discovery and the OAuth of the live suites: no code moves, the new behaviours reaching the user through the errors already surfaced.
