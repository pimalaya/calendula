---
cairn: log
date: 2026-08-28
change: duplicate-link-id-mints-an-item
---

# A UID names two items now, and nothing here was addressing by it

calendula is the last link of the chain pimdir, io-replica, io-pimdir, io-webdav and neverest carried today: a calendar collection may hold two calendar object resources whose bodies state one `UID`, which RFC 4791 4.1 forbids and servers do not always enforce, and the engine now keys the second apart as `dup:<hint>#<handle>` (pimdir SPEC 9) instead of freezing it. What was one row is two, and the invariant this crate never had to state, one item per `UID`, stops holding.

## What landed

**The audit came back clean.** Every read in src/pimdir/ addresses an item by its public `seq`: `item` parses the id and resolves it through the reader, `scan_items` pages on the sort key rather than on the link id, and `in_range` answers per item. Nothing re-derives a `UID` to look a row up, nothing read `bindings.ambiguous_handles` or `ReplicaStatus::Ambiguous`, so io-pimdir dropping them is not breaking here. `create_item` is unchanged on purpose: it stages the derived bare `UID` and reports it, and a staged add naming an identity the collection already holds parks in the store (pimdir SPEC 15.3) rather than minting, which is the opposite answer to the one a source gets and the right one for a producer.

**The dependencies point at the sibling checkouts.** Cargo.toml's `[patch.crates-io]` pointed io-pimdir, io-replica and io-webdav at git, which resolves to revisions predating the minted key. Nothing in the chain is published, so the three entries now carry `path = "../io-pimdir"`, `path = "../io-replica"` and `path = "../io-webdav"`, which is what makes the local suite testable end to end. They must go back to git or crates.io before a release, recorded as an open task on the change.

**One stale rationale, one new test.** The doc comment on `an_added_item_links_the_way_the_store_spells_it` in src/pimdir/backend.rs claimed a create "deduplicates against" a synced copy, which was never what the store did and now reads as the opposite of the rule: a colliding staged add parks. `two_items_sharing_one_uid_list_under_their_own_public_ids` is the regression guard, over two hand-built items carrying the Posteo pair "Pre demo woonies" and "Pre demo MINIS" under one `UID`: the identity both bodies derive is one string, and the two items still project two events under their own public ids.

**The CHANGELOG collapsed rather than gained a fix.** The pimdir backend has not shipped, v0.1.0 predating it, and `[Unreleased]` is the net diff against that release, so a "no longer disappears" bullet would announce a fix to a version nobody ran. The behaviour is folded into the backend's own `### Added` entry instead, as the shape it ships with.

## Capabilities moved

- backends: a `UID` is not an address, and the public id is what keeps two duplicated resources apart

## Verification

82 tests green, `cargo clippy --all-targets` clean, `cargo fmt` applied, all through the nix devshell with the default feature set (the one carrying the pimdir backend). No live store was read: the parking behaviour is the store owner's and is tested in io-pimdir, and this crate has no store fixture.
