---
cairn: change
id: follows-storage-draft-17
---

# Journal

- 2026-11-23 17:00 0.1: the matrix at 82547a7 (HEAD moved from 794550c by the plan commit alone) gives inventory.md's counts on all 12 sets, clippy-clean.
- 2026-11-23 17:01 0.3 prep: HEAD's release binary (fat LTO) kept as ~/code/pimalaya/.lanes/calendula/bin/calendula-base before any edit.
- 2026-11-23 17:10 A1: pinned straight to io-pimdir drop 1 (cc3021b, lead's message), 7f16b27 never committed. Cargo's MSRV resolver also moves windows-sys 0.61.2 to 0.52.0 in five Windows-only entries of the lock.
- 2026-11-23 17:20 A3: cargo-deny failed on RUSTSEC-2026-0592 (`url` 2.5.8, `Url::origin` stack overflow), present at HEAD too, not the pin's; `cargo update -p url` to 2.5.9 (form_urlencoded 1.2.3) clears it. The first `nix build` (A alone) failed on B's breaks as expected, the git source vendoring fine.
- 2026-11-23 17:35 B3, C1, D1: the no-run matrix compiles on all 12 sets; the two tests fail before C1 and D1 (backend.rs:3232, index.rs:875) and pass after. CD gate: the matrix equals the baseline on all 12 sets, clippy-clean.
- 2026-11-23 17:40 0.2: the four stores seeded under ~/code/pimalaya/.lanes/calendula/stores/ (v19 10k in 122 s, 100k in 1,267 s; v20 10k in 65 s); `pimdir status` counts 10,000 and 100,000 plus 200 on the v19 ones.
- 2026-11-23 17:45 G1: `a_malformed_queue_row_fails_no_listing` (tests/pimdir_queue.rs) fails before the fix (`event list -k a` exits 1, "payload states no integer version", code null) and passes after. `queue list` exits 3 there, its partial contract, unchanged. Decision: only a `PimdirError::Action` (decode) is tolerated per calendar; any other store error still fails the listing, coded through `store_error`.
- 2026-11-23 17:50 E1, E2, E3: five tests added (three in tests/pimdir_search.rs, two in tests/pimdir_queue.rs), the MCP instructions test asks for `reserved-form`, `mo` and `label:`. The matrix: 566 (default, all-features), 400 (vdir,pimdir), 321 (pimdir), the others unchanged, clippy-clean.
