---
cairn: log
change: published-dependencies
landed: 2026-11-12
---

# Build on the published dependencies

calendula takes every Pimalaya crate from crates.io: Cargo.toml requires io-gcal 0.3.1, io-msgraph 0.7.1, io-pimdir 0.10 and pimalaya-cli 0.3.2 (build and normal), and `[patch.crates-io]` is gone. Each former pin is inside its release: ical-rs 99b9273 in 0.7.0, io-gcal 9053761 in 0.3.1, io-http e181530 in 0.6.0, io-msgraph 2ae2637 in 0.7.1, io-pimdir 5c18a02 in 0.10.0 (one commit apart, the version bump), pimalaya-cli f976e40 in 0.3.2. No calendula code changed.

- Cargo.lock moves io-gcal, io-msgraph, io-pimdir and pimalaya-cli to the registry, and drops what only the pins held: the second ical-rs (the 0.6.1 git revision io-gcal and io-msgraph built on), the second io-http (the 0.5.0 git revision), base64 0.23.1, and git2, libgit2-sys and libz-sys (pimalaya-cli 0.3.2 runs git for its build metadata).
- io-gcal 0.3.1 writes an event with neither `DTEND` nor `DURATION` as ending at its start, the next day after a date start (RFC 5545 3.6.1), where it refused it; io-msgraph 0.7.1 restores a stashed alarm whole rather than scattering its lines. Both are CHANGELOG Fixed entries. The CHANGELOG's clauses promising to drop the pins are gone.
- pimdir-adopts-the-io-pimdir-release-api lands with its release task; cli-alignment-leftovers has its A2 task done.

Verified: `cargo fmt --check`; `cargo clippy --all-targets -D warnings` with the default features and `--all-features`; `cargo test --all-features` (289 tests).

Capabilities moved: none.
