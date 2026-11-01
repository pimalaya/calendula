---
cairn: log
change: pimdir-follows-storage-draft-08
landed: 2026-11-01
---

# The pimdir backend follows storage-draft-08

calendula builds against io-pimdir bff5737 (pimdir d671bbd: storage-draft-08, sync-draft-07, search-draft-08), through the `path = "../io-pimdir"` patch while porting.

- The compile port: `calendar.component.vtodo` replaces `x-vtodo` (`CALENDAR_COMPONENT_VTODO`), `PimdirInvitation::source` is a `String`, `PimdirError::Stale` and `HashAlgo::declared` are gone, `PimdirError::code()` maps the new vocabulary, `PimdirBlobs::path` answers `io::Result<PathBuf>`. Cargo.lock moves io-pimdir and its own dependencies alone (rusqlite 0.41, libsqlite3-sys 0.39, encoding_rs and two crates of its own new; io-pimdir now also depends on jiff and rustix, and no longer on sha2).
- The to-do gate reads `calendar.component.vtodo` only where a row states it: `none` on the calendar, or source-wide without a calendar row overriding it, refuses; a declaration stating nothing refuses nothing. The refusal names the capability.
- Every reply and cancel names its performer, the payload requiring `source`: in a calendar whose sources declare nothing, `--source`, else the one source syncing it, several or none refused pointing at `--source`. `PimdirQueued::source` and the JSON `source` of `pimdir reply` and `pimdir cancel` are never `null`. Payloads are built and validated by `PimdirInvitation::to_action`, a refusal there or at the enqueue carrying `action-malformed`.
- Error vocabulary: `store-newer` and `store-stale` out, `store-version`, `store-exposed`, `flag-invalid` and `action-malformed` in. A store of an earlier draft, or naming no format version, is refused with the reinstall in the order that keeps its queued writes (sync with the neverest that wrote it until nothing is queued, then `neverest sync --reset --force -a <account>`); a newer one with the upgrade of calendula. Neither says to delete the store.
- A blob entry that is no body (a directory, a link, bytes not hashing to its name) now reads as a missing one, which a listing logs and passes; the test poisoning bodies now revokes the file's permissions instead of turning it into a directory.
- `--send-updates` is refused by name on pimdir, as on every non-gcal backend, before anything is queued.

Tests failing against the old behaviour first, checked by putting the old code back in place (the old `store_error` arms and io-pimdir's own `store-version` message, an undeclared store passing `--source` through unnamed, the refusal matched on `x-vtodo`) and running `cargo test pimdir`: `a_store_failure_carries_the_code_io_pimdir_names`, `a_store_of_another_version_names_the_reinstall_or_the_upgrade`, `a_store_calendula_cannot_open_carries_its_code`, `a_to_do_into_an_events_only_calendar_names_the_way_out`, `an_undeclared_calendar_several_sources_sync_needs_a_source`, `a_queued_reply_and_cancel_read_back_under_their_schema`, `an_undeclared_store_refuses_one_occurrence`, and the two payload tests now expecting `source`. `a_to_do_is_gated_only_where_a_row_states_it`, `a_reply_outside_the_three_participations_queues_nothing` and `send_updates_is_refused_by_name_and_queues_nothing` pin behaviour that already held.

Not done here: the end-to-end run on a scratch store a neverest following sync-draft-07 synced, and the git pin replacing the path patch.

Capabilities moved: **backends** (pimdir is an availability-aware cache, pimdir refuses a to-do where its calendar takes none, a pimdir store failure carries io-pimdir's code, pimdir queues scheduling intents, modified), **commands** (a JSON error carries a stable code, modified).

## Supervisor follow-up (2026-11-01)

- Pinned to io-pimdir bff5737 by git rev, the path patch dropped; `cargo test --all-features` green on the pin (252 passed).
- End to end on a scratch store neverest synced from local Stalwart IMAP and Radicale DAV (not a provider test account): event and to-do list, an event create and update, a to-do create and delete; the writes queued here, then pushed by `neverest sync`, each write found on its server.
