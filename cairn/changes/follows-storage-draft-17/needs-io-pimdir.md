---
cairn: change
id: follows-storage-draft-17
---

# What calendula needs from io-pimdir

- **N1, the pending actions per row.** `PimdirReader::pending_actions` and `list_pending_actions` fail whole on the first row that does not decode, and expose neither that row nor the valid ones. Why: calendula's B1 (inventory.md) is one malformed row from any producer failing every `event` listing, and `pimdir queue list` drops the valid rows of that calendar (the contract audit's handoff-pimdir.md item 8). The ask: a per-row result, or the undecoded rows with their ids beside the decoded ones. Evidence: inventory.md "Bug B1", reproduced on 0.19.1 and 7f16b27. App test: G1's regression test, then a `queue list` test listing the valid rows beside the entry. Workaround meanwhile: D5 (no listing fails, the summary rule-out is dropped for that calendar).
- **N2, a calendar profile in spec/generator.** The ask: events at a chosen count, with weekly, daily, monthly and yearly series, UNTIL, COUNT and open-ended ones, EXDATEs, overrides in the resource, IANA-zoned and Outlook-zoned VTIMEZONEs, all-day and floating times, and attendees. Why: the count profile files events with no recurrence (src/profile.rs:1165-1182), so no lane can bench expansion on a shared fixture. Workaround: bench/seed.rs here.
- **N3, a domain left out says so.** Under `search` without `calendar`, a calendar hit silently carries no snippet and `PimdirIndex::item` reads it from its summary (src/client/search.rs:2941, src/search/extract.rs:154-163). The ask: document it on the `search` feature and the two methods, or answer a typed "domain not built" so a consumer notices. Why: an `indexer` dev-dependency turns `calendar` on in every test build, so no test sees the loss. Workaround: D2's cargo tree gate.
