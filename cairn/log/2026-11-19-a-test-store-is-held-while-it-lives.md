---
cairn: log
change: a-test-store-is-held-while-it-lives
landed: 2026-11-19
---

# A test store is held while it lives

Landed without a proposal: test-only, no behaviour moved. `pimdir::index::tests::a_weekly_series_begun_years_ago_is_found_on_a_later_week` once panicked in `sync_from` (src/pimdir/client.rs) on `PimdirStore::open(..).unwrap()` under a full parallel `cargo test`.

Evidence, on 2026-11-19: 60 runs of the unit binary filtered on `pimdir::`, 40 full runs, 96 and 96 runs of an A/B under 8 parallel loops, all passed; 80 full runs under 8 parallel loops failed once, `pimdir::backend::tests::an_online_meeting_the_event_already_asked_for_is_no_new_ask` at its `PimdirStore::open(dir.path()).unwrap()`: `Owned("/tmp/.tmpcVPGeL")`, a fresh temporary store found owned. A throwaway probe (not kept) opened, dropped and reopened 2000 fresh stores while another thread spawned `true` in a loop: 231 reopens failed `Owned`, none with the spawner off.

Cause: io-pimdir's owner lock is a `flock` on the open file description of `owner.lock`, released when the last handle drops. A child spawned by another test thread holds a copy of every descriptor from its clone until it executes (`O_CLOEXEC` closes them only then), so for that moment the dropped lock is still held, and a reopen in the same process takes a new description and finds the store owned. The fixtures open a store, drop it, then reopen it (`sync_from`, a declaration, the backend's own writes); `a_reopened_backend_runs_its_secret_command_once` (src/shared/client.rs) spawns its secret command in the same binary.

Fix, in the tests alone: `StoreDir` and `store_dir()` (src/pimdir/client.rs, `cfg(test)`) create the store and hold one owner handle for as long as the temporary directory lives, so io-pimdir's per-process registry keeps the lock and every later open shares it, never taking it anew. The store fixtures of src/pimdir/backend.rs, src/pimdir/index.rs, src/pimdir/queue.rs, src/shared/event/update.rs and src/account/check.rs take it in place of `tempfile::tempdir()`. `a_store_calendula_cannot_open_carries_its_code` rewrites the database under a store no handle holds, as its open is a reader taking no owner lock. The same probe over `store_dir()`: 0 of 2000 reopens failed beside 62307 spawns.

Left: the staging lock (`objects.lock`) is open to the same window, unobserved; production calendula reopens no store while it spawns in another thread.
