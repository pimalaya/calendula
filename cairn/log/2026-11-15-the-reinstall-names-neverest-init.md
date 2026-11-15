---
cairn: log
change: the-reinstall-names-neverest-init
landed: 2026-11-15
---

# The store reinstall names `neverest init`

The message refusing a pimdir store an earlier neverest wrote said to reinstall it with `neverest sync --reset --force -a <account>`, which neverest cannot do: it refuses such a store before any account runs, and its reset drops an account's collections, never the store. The message, MIGRATION.md and the CHANGELOG now give neverest's own order (neverest cairn/spec/sync.md, "A store opens under its own format version alone"), as Cardamum does: sync with the neverest that wrote the store until nothing is queued, upgrade neverest, move the store directory aside, then `neverest init` and a sync. The tests in src/pimdir/client.rs and src/pimdir/backend.rs assert `neverest init` and fail on the old message.

Capabilities moved: backends (cairn/spec/backends.md, the pimdir store-version requirement).
