---
cairn: log
change: pimdir-1-0-codes-relayed
landed: 2026-11-26
---

# calendula relays io-pimdir 1.0.0's `source-empty`

io-pimdir 1.0.0 added two codes: `source-empty` (`PimdirError::SourceEmpty`, a store bound to a source of an empty id) and `target-unsynced` (`PimdirMutateError::Unsynced`, a copy or a move into a collection no source syncs). calendula's relay table (src/pimdir/client.rs `PIMDIR_CODES`) is Himalaya's but `tag-refused`, and Himalaya relays `source-empty`, so calendula now does too, reversing the 2026-11-26-io-pimdir-1-0 entry's "`source-empty` is not relayed".

- **Code**: `ErrorCode::SourceEmpty`, serialised `source-empty`, in the `calendula-error` schema. No calendula path raises it today: it binds no source; the entry keeps the one vocabulary every pimdir program reads.
- **Not relayed**: `target-unsynced`. It is `PimdirMutateError::code`, raised only on the owner's run, which calendula never drives; a copy into an unsynced target reaches a producer as a parked queue row, not as an error. Himalaya leaves it out for the same reason.
- **Tests**: `a_relayed_pimdir_code_keeps_io_pimdirs_spelling` covers the spelling; `a_store_failure_carries_the_code_io_pimdir_names` gains `PimdirError::SourceEmpty`.
- **Docs**: CHANGELOG 1.0.0 (the pimdir writes codes), the code table of commands.md, the relayed list of backends.md.

Capabilities moved: commands (one code), backends (the relayed list).
