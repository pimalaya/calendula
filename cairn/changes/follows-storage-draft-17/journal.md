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
