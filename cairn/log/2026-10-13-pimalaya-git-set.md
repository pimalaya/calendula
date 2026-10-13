---
cairn: log
change: pimalaya-git-set
landed: 2026-10-13
---

# io-pimdir 0.8, io-msgraph 0.6, and the unreleased Pimalaya set

io-pimdir moves from 0.7 to 0.8 and io-msgraph from 0.5 to 0.6, and `[patch.crates-io]` pins ical-rs (`caa1422`), io-http (`e181530`), io-msgraph (`2ae2637`), io-pimdir (`189db40`) and io-webdav (`62fad89`), the pushed revisions the Pimalaya Android client builds, so calendula opens the same store schema as the neverest, himalaya and cardamum built beside it. The patches go once those revisions are released. `Cargo.lock` holds one ical-rs, one io-http and one io-pimdir.

What the revisions change for calendula: io-http bounds a response head at 64 KiB and a body at 256 MiB, failing with a typed error past either, and fails on a connection closed mid-body; io-webdav parses a multistatus without quadratic copies, nesting capped at 64; ical-rs caps component nesting at 16. io-pimdir 0.8 adds the file kind (Annex A.7), so `start_of` in `src/pimdir/backend.rs` answers `None` for a `PimdirSummary::File`, as for mail and contacts: a file has no date. The new `since` floor on the mail counts and pages and `PimdirMailSummary::invitation` touch no API calendula calls.

Verification: `cargo test` with all features, the default features, and `--no-default-features --features pimdir` as MOA builds it (179, 179 and 138 tests); `cargo clippy --all-targets -- -D warnings` on the same three sets; `cargo fmt --check`; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`; `caldav`, `gcal`, `msgraph` and `vdir` each alone with `rustls-ring`.

Capabilities moved: none.
