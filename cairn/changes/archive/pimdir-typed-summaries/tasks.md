---
cairn: tasks
change: pimdir-typed-summaries
---

# Tasks

- [x] Cargo: `pimdir` drops `dep:io-replica`, io-pimdir patched to the local path, io-replica removed
- [x] Client and backend import the `Pimdir*` types from `io_pimdir::client::{reader, producer, blobs}` and the model modules
- [x] `add` and `update` carry the body alone; `enqueue` takes no timestamp
- [x] The listing scans with `list_summaries` and reads kind and date from the typed summary
- [x] The JSON summary reading, the stamp folding and the derivation wrapper are deleted
- [x] Tests build typed `PimdirItem` rows and cover the kind answer
- [x] The main.rs header and the module docs name io-pimdir alone
- [x] Build with `pimdir`, the default set and without it; tests, clippy, fmt
- [x] CHANGELOG net diff, spec fold, log
