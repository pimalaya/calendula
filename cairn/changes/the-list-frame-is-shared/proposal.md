---
cairn: change
id: the-list-frame-is-shared
status: landed
created: 2026-11-14
---

# The list frame is shared

> Cross-repo change, same id in pimalaya-cli (79c0d9e), Himalaya (984549dd) and Cardamum (40237d2). Order: pimalaya-cli, Himalaya, Cardamum, then Calendula (this side).

## Why

The owner decided on 2026-11-14 that the list frame, the item errors and the `account capability` output belong to pimalaya-cli, so every binary prints a list, its item errors and an account's capabilities the same way. Calendula carried copies: src/shared/listing.rs (the header, the paging and the label column of `event list`, `event find`, `item list` and `gcal instances`) and `error::ItemErrors` (a copy of pimalaya-cli 79c0d9e's, waiting for the pinned commit to carry it). Its `account capability` printed every configured backend as a block, `{account, backends: [{backend, capabilities: [{name, level, detail}]}]}`, under a `CAPABILITY  BACKEND  LEVEL  DETAIL` column header, and failed when `--backend` allowed no configured backend.

## What

- Cargo.toml patches pimalaya-cli by path (`../cli`) until its release, where it pinned 0be1b1f; the io-gcal pin and the io-webdav and io-pim-discovery path patches stay. Its own commit.
- `event list`, `event find`, `item list` and `gcal instances` print through `pimalaya_cli::listing::{Listing, ListingPage}`; src/shared/listing.rs is gone. `calendar list` and `event agenda` print their errors through `pimalaya_cli::error::ItemErrors`; the local copy is gone. The text of every one is byte for byte what it was. `error::with_code` stays (owner decision pending); the agenda's JSON `{starts, errors}` stays.
- `account capability` prints `pimalaya_cli::capability::AccountCapabilityOutput`: `{account, capabilities: [{name, backend, level, detail}]}`, one row per capability, `backend` the backend the unified commands resolve to, `null` with level `none` when the account configures no backend `--backend` allows, where the command failed. `--backend <name>` reads another backend of the account. `capability::Support::level` is the typed `CapabilityLevel`. The text is a list headed by `Account:`, one line per capability with no column header. `shared::client::resolve_backend` names the backend `CalendarClient::new` takes, so both read one priority order.
- The command was not in v0.2.0 (`git ls-tree v0.2.0 src/` holds no src/account/capability.rs), so the CHANGELOG amends its [Unreleased] Added entry rather than adding a breaking one, as Cardamum did.
- Spec: commands ("A command's errors print at its end", "Text output is plain text", "An account lists what it serves").
