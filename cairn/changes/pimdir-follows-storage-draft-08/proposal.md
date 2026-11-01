---
cairn: change
id: pimdir-follows-storage-draft-08
status: landed
created: 2026-11-01
---

# The pimdir backend follows storage-draft-08

## Why

io-pimdir bff5737 (pushed, master, still 0.8.0) follows pimdir d671bbd: storage-draft-08, sync-draft-07, search-draft-08 (io-pimdir cairn/changes/the-crate-follows-storage-draft-08). calendula reads and queues behind neverest.

Built on 2026-11-01 against ../io-pimdir at bff5737 (`cargo check --all-targets`, over the uncommitted work of 2026-10-30), 12 errors:

- `capability::X_VTODO` is `CALENDAR_COMPONENT_VTODO` (`calendar.component.vtodo`) (src/pimdir/backend.rs:33, 2126).
- `PimdirInvitation` and `PimdirCollectionCreate` take `source: String` (src/pimdir/backend.rs:671, 714, 753, 755, 802).
- `PimdirError::Stale` is gone, `HashAlgo` has no `declared`, and the code mapping expects `Option<String>` (src/pimdir/client.rs:178, 180, 187).
- `PimdirBlobs::path` answers `io::Result<PathBuf>` (src/pimdir/backend.rs:2923, 2924).

## What the compiler does not see

- **The to-do gate's meaning moved**: `calendar.component.vtodo` is read only where a row states it, a source stating `none` on the calendar or source-wide refusing a VTODO, a source stating nothing refusing nothing (pimdir STORAGE §15.6, Annex B). The refusal text of a-to-do-refusal-names-the-way-out names the new capability; neverest's events-only calendars state `none` (neverest b2de005, ported in neverest-follows-sync-draft-07).
- **Strict payloads**: an invitation reply's `partstat` outside `ACCEPTED`, `TENTATIVE`, `DECLINED`, an item named by both or neither of `seq` and `link_id`, and a cancel stating `partstat` are refused as `Invalid`; calendula's reply and cancel verbs refuse them before the enqueue.
- **Error codes**: io-pimdir no longer raises `store-stale` or `store-newer`; it adds `store-version`, `store-exposed`, `flag-invalid`, `action-malformed`. A store an earlier neverest wrote is refused as `store-version`: the message points at neverest's reinstall, never at deleting the store.
- **Scheduling**: `summary::calendar::unschedule` now writes `SCHEDULE-AGENT=NONE` when the user accepts that nobody is notified; adopting it is a separate change, not this port.

## Pin

Built against `path = "../io-pimdir"` while porting, then `[patch.crates-io]` by git rev bff5737 (replacing 84aa00c's 8907303); the requirement stays `0.8` until the user releases io-pimdir.
