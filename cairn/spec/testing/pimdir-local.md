---
cairn: spec
capability: testing/pimdir-local
status: current
---

# pimdir (local store): shared-command test report

- calendula: `--features rustls-ring,pimdir` (working tree; io-pimdir, io-replica and io-webdav at their current git revisions)
- account: `posteo-local`, the same store the machine's own ~/.calendularc configures, driven here through a throwaway copy of that block in a scratch config (`--config <scratch>/calendula.toml`) so the run could not depend on or disturb the real file
- date: 2026-08-28
- method: **read-only against a live store**, the first run of this backend against one a sync actually populated.

  The store holds `caldav/default` with 454 calendar items (380 VEVENT projections and 75 VTODOs), all hydrated at `Full`, beside `carddav/default` and twenty-odd `message/rfc822` collections.

  Nothing here writes: every item verb exercised is a read, the three `calendar` verbs refuse before touching anything, and the store's queue was confirmed empty before and after with `pimdir queue list`.

  The write paths are covered separately, against a **clone of the index alone** (pimdir.db copied into the scratch directory, no blob tree), so every staged action was appended to a throwaway store.

  The production store was confirmed untouched afterwards: its queue still empty and item 139 still pointing at its original object.

## Results

| Command | Variants tested | Result |
| --- | --- | --- |
| `account check` | base | ✅ `pimdir: OK` |
| `account list` | base | ✅ one account, backend `pimdir` |
| `pimdir status` | base, `--json` | ✅ 454/454 downloaded, 0 queued, the store's one grouped account named (**CD1 fixed**) |
| `calendar list` | base, `--json` | ✅ one calendar, `caldav/default`; the mail and contact collections are filtered out by kind |
| `calendar create` | base | ⛔ bails: the collection row is the sync's to write |
| `calendar update` | `-n` | ⛔ bails, same reason |
| `calendar delete` | `-k` | ⛔ bails, same reason |
| `item list` | base, `-s`, `-p` (first, middle, last, past the end), `--json` | ✅ 454 items in the store's calendar order, paging total across the boundaries with no repeat and no gap, the last page short, a page past the end empty |
| `item read` | `<id>`, absent id, non-numeric id | ✅ the stored iCalendar byte for byte; an absent id → "Item `99999` not found in calendar `caldav/default`"; a non-numeric one → "Invalid pimdir item id `abc`: expected the number a listing showed" |
| `item list` / `item read` | unknown `-k` | ⛔ "Calendar `nope` not found", the guard holding on both the read and the addressing path |
| `event list` | base, `-s`, `--from`/`--to` (hit, miss) | ⚠️ a window lists correctly and lifts the page cap; **the unwindowed listing came back empty** (**CD2**, open) |
| `event read` | `<id>` | ✅ the stored resource verbatim |
| `todo list` | `-s`, `--json` | ✅ summary, status, priority and completion rendered from the parsed bodies |
| `journal list` | `-s` | ✅ empty, the store holding no VJOURNAL |
| any command | `pimdir.root` naming no store | ⛔ "Open pimdir store `…`: unable to open database file", and nothing is created at the path |
| `calendar list`, `item list` | **run while another process holds owner.lock** | ✅ list normally, which is the point of the role change (**CD5 fixed**) |
| `item create` | file argument, twice | ✅ writes the body into the blob tree under the store's base32 name, appends one `add` row naming the bare `UID`, and reports the link id; an identical second body deduplicates onto one blob |
| `item update` | `<id>` | ✅ appends one `update` row addressing `seq 139`; **`item read` on it returns the staged body**, the pending overlay working end to end |
| `item delete` | `<id>` | ✅ appends one `remove` row addressing `seq 40`; the item leaves the listing (454 to 453) and `item read` on it reports not found, again through the overlay |
| `item list` | **item naming an object whose blob is gone** | ⚠️ lists blank, silently (**CD6 fixed**) |

## Findings

### Bugs / issues

- **CD1: `pimdir status` claimed the store grouped no account: FIXED.** The unset-`pimdir.account` arm printed "Reading account: every collection (none grouped)" while the same report's next line listed `posteo` as a grouped account.

  Reading every collection and the store grouping none are two different facts, and the message asserted the second from the first. It now reads "every account in the store", which is what the unset case actually does.

- **CD2: `event list` with no window returns an empty table on a real store: OPEN.** The shared listing paginates at the backend and filters by component afterwards (the command's own help points at `item list` for the unfiltered view), so a page holding no VEVENT renders empty.

  It never showed before because the scan order was the link id, which interleaves the kinds by accident.

  The store's calendar order sorts an item with no resolved start first, and this store's 75 VTODOs carry no `DUE`, so they fill the first page and a plain `calendula event list` reports nothing on a calendar holding 380 events.

  The order is right and the projection is right; what is wrong is doing them in that sequence.

  Fixing it means either projecting before paginating in the shared layer, which changes every backend and costs a full scan per listing, or paging the store descending. Left open: it is a shared-command design decision, not a pimdir one.

- **CD3: the link id and the summary were calendula's own derivations: FIXED.** src/pimdir/meta.rs derived them by hand in 533 lines.

  Checked against the live store: `io_pimdir::conventions::calendar::derive` reproduces item 139's stored `link_id` (`223134df-…`, the bare `UID`) and its stored `meta` byte for byte, field order included.

  That is what makes an item calendula stages and the same item arriving through a sync one item rather than two.

- **CD4: the body hash was a digest of calendula's own choosing: FIXED.** src/pimdir/hash.rs computed a 128-bit FNV-1a rendered as 32 hex chars.

  The live store names its bodies in lowercase base32 under the algorithm recorded in `store_meta.hash_algo`, which reads `Blake3`; item 139's body hashes through the store's handle to exactly the stored `erc6eo3d…` name. A body written under the old digest is a body no read ever finds.

- **CD6: an item whose blob is missing listed as a blank row: FIXED.** `item_from` read the body with `unwrap_or_default()`, so an item naming an object whose file is gone rendered with an empty size and vanished from `event list` and `todo list` (nothing to parse).

  `item read` on the same item correctly reported "Body blob missing": the listing hid a broken store the read reports. This is **not** the not-fetched case, which carries no object at all and is ordinary.

  The listing still renders the row, since the shared type is the raw bytes and there is no document of record to invent, but it now logs a warning naming the item. Same defect as cardamum's PD4.

- **CD5: calendula took the owner role: FIXED.** io-pimdir 0.3 makes the owner exclusive, so `calendula event list` and a sync could not run at the same time, either one failing with `PimdirError::Owned`.

  calendula is a reader plus a producer, and now holds those two handles. Verified by holding owner.lock from another process and listing anyway.

### Backend behaviour (not bugs)

- **The pending overlay works end to end.** A staged `update` reads back through `item read` and a staged `remove` drops the item from both the listing and the read, before any owner applied either. This is what makes a write feel like a write on a store calendula does not own.
- **A queued create has no public id.** `create_item` reports the item's link id instead of a `seq`, the `seq` not existing until the owner applies the action.
- **A queued action carries no sort key.** io-pimdir leaves the key to the sync that pushes the write, so an item staged here sorts at the head of the listing until the next sync resolves it.
- **Item ids are the store-assigned public `seq`,** which is why a non-numeric id gets its own parse error and why the ids in a listing are sparse (`19`, `40`, `66`, …): the `seq` is store-global, shared with the mail and contact collections.
- **`pimdir.account` is taken verbatim, never derived.** himalaya resolves the single account of a store and refuses a store holding several rather than guessing; calendula reads every collection when the key is unset, which on a multi-account store merges two accounts' calendars silently.

  Same shape as cardamum's. Worth aligning on one behaviour across the three.

- **The queue stayed empty throughout**, checked before and after, which is the proof that this run wrote nothing.

### Observations

- **The shared write confirmations say "created", "updated", "deleted" where pimdir staged.** `item create` reports "Item `<link id>` successfully created" for an item that does not exist yet and will not until a sync applies the queued action, and the other two read the same way.

  The messages are shared across backends and true on every other one; on pimdir they overstate.

  Not filed as a defect, since wording that names the queue would have to be either backend-specific text in the shared commands or an outcome the shared layer carries, both of which are decisions rather than fixes.

- `pimdir check` reports the store carrying 12 orphan blob files (369.8 KiB, reclaimable with `pimdir gc`) and 13 identities a source holds more than once, four of them in `caldav/default` and all four Google-originated UIDs.

  Both predate this run and neither is calendula's doing: the duplicated identities are the duplicate-link-id freeze, and the items behind them do not sync until the source holds each once again. No refcount drift and no dangling row.

- The scratch config and the store path were kept out of `~` throughout, and the only files this run created are the two throwaway configs under the scratch directory.

## Verdict

The read side is correct against a real 454-item calendar: listing, calendar ordering, paging totality, byte-faithful reads, window filtering, and every error path naming what is wrong.

Five defects, four fixed, and two of the four (**CD3**, **CD4**) were silent data-duplication bugs only a live store could reveal, since both are about agreeing with what a sync engine already wrote.

**CD5** is the structural one: the backend held the one handle the format says a frontend must not hold. **CD2** is left open on purpose, being a shared-command decision rather than a pimdir one.

The write side is verified too, on a cloned index: `add`, `update` and `remove` each append exactly one correctly addressed queue row under the producer name `calendula`.

The body reaches the blob tree under the store's own name before the row that pins it, and the pending overlay makes all three visible before a sync runs.

What no run here proves is the far half: an owner draining those rows and pushing them to Posteo. That needs a sync, and it is the outstanding work on this backend.
