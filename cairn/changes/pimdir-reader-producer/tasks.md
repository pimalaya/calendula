---
cairn: tasks
change: pimdir-reader-producer
---

# Tasks

- [x] The client holds a `PimdirReader` with the pending overlay, and opens a `PimdirProducer` per write
- [x] The three item writes enqueue `add`, `update` and `remove` instead of staging replica mutations
- [x] `pimdir.source` leaves the config, the wizard, the sample and the README; `pimdir.account` takes its place
- [x] The derivations delegate to `io_pimdir::conventions::calendar`, and the body hash comes from the store
- [x] src/pimdir/meta.rs and src/pimdir/hash.rs are deleted
- [x] An unknown calendar id fails rather than reading as empty
- [x] `pimdir status` reports the account, the grouped accounts and the queued creations
- [x] caldav: `CaldavCalendar::supported_reports` after the io-webdav bump
- [x] cargo fmt, clippy, tests, feature matrix
- [x] Read-only run against the live Posteo store, reported under spec/testing
- [x] Fold the delta and log
