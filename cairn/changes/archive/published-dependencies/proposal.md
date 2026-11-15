---
cairn: change
id: published-dependencies
status: landed
created: 2026-11-12
---

# Build on the published dependencies

## Why

Every revision calendula pinned through `[patch.crates-io]` is now inside a release: ical-rs 99b9273 in 0.7.0, io-gcal 9053761 in 0.3.1, io-http e181530 in 0.6.0, io-msgraph 2ae2637 in 0.7.1, io-pimdir 5c18a02 in 0.10.0, pimalaya-cli f976e40 in 0.3.2. The packaging requirement "Released dependencies, patched only while unreleased" asks for them to go. It comes first in the-family-standards so any API move surfaces before the large edits.

## What

- Cargo.toml requires io-gcal 0.3.1, io-msgraph 0.7.1, io-pimdir 0.10 and pimalaya-cli 0.3.2 (build and normal), and drops `[patch.crates-io]`; ical-rs 0.7 and io-http 0.6 were already required.
- The releases carry two behaviour fixes: io-gcal 0.3.1 writes an event with neither `DTEND` nor `DURATION` as ending at its start instead of refusing it, and io-msgraph 0.7.1 restores a stashed alarm whole.
- It completes the release task of pimdir-adopts-the-io-pimdir-release-api and the A2 task of cli-alignment-leftovers.
