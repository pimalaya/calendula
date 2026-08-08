---
cairn: tasks
change: pimdir-cache-backend
---

# Tasks

- [x] Cargo `pimdir` feature pulling io-pimdir and io-replica; `Backend::Pimdir` and `allows_pimdir`.
- [x] `PimdirConfig { root, source }` and `AccountConfig.pimdir`.
- [x] `src/pimdir/`: client (open as source, auto-detect, shell-expand), hash (Neverest-matching digest), meta (the `text/calendar` convention), backend (reads plus staged writes), status command.
- [x] `shared/client.rs`: the `Pimdir` variant and every dispatch arm, local before network.
- [x] Availability-aware reads; a range filter answered from the summary when the body is not local.
- [x] `account check` and the wizard's local detection both cover pimdir.
- [x] The `text/calendar` summary convention contributed to the pimdir specification.
- [x] Tests: id parsing, the collection refusal, summary-stamp folding, summary-windowed range, digest shape, projection.
