---
cairn: tasks
change: the-list-frame-is-shared
---

# Tasks

- [x] Cargo.toml: pimalaya-cli patched by path, its own commit.
- [x] `event list`, `event find`, `item list`, `gcal instances` on `Listing` and `ListingPage`; `calendar list` and `event agenda` on `ItemErrors`; src/shared/listing.rs and the local `ItemErrors` removed; the byte-exact tests pass with their expected text unchanged.
- [x] `account capability` on the shared output, `Support::level` typed `CapabilityLevel`, `resolve_backend` shared with `CalendarClient::new`.
- [x] tests/capability.rs (the text, the flat rows naming their backend, a `--backend` the account lacks) and tests/mcp.rs (`account_capability` rows and their schema), each failing on the old code.
- [x] CHANGELOG [Unreleased] Added entry amended, MIGRATION.md, README.md.
- [x] Fold the delta into commands.md, write the log entry, set `landed`.
