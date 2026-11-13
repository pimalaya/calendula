---
cairn: tasks
change: io-family-releases
---

# Tasks

- [x] Cargo.toml: io-oauth 0.6 (tests), io-gcal 0.4, io-http 0.7, io-msgraph 0.8, io-pim-discovery 0.12, io-vdir 0.3, io-webdav 0.8; the pimalaya-cli pin untouched; Cargo.lock moves those seven and gains psl and psl-types
- [x] One io-http (0.7.0) and one io-oauth (0.6.0) in the tree; no io-sasl
- [x] gcal: the wire spellings borrow the enums and print an unknown value as the API spells it; the rows hold strings, defused when printed
- [x] gcal: a full stash writes the event without the lines left out, a warning naming them, on a create and on an update, with a test over a scripted server
- [x] gcal: a scripted test of an id holding `#` and `@`, sent as one encoded segment
- [x] msgraph: the calendar listing follows its pages through `page_get`, with a test of a link off the Graph origin
- [x] CalDAV, vdir, discovery, the live suites: read against the new releases, no code to move
- [x] `cargo fmt --check`, clippy `-D warnings` and `cargo test` over the eleven feature sets of tests.yml, `cargo-deny check`
- [x] Spec: projection (the stash), backends (gcal ids, CalDAV create and listing, Graph pages), commands (gcal family)
- [x] CHANGELOG, log; land
