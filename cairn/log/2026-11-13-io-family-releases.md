---
cairn: log
change: io-family-releases
landed: 2026-11-13
---

# Build on the io-* releases of 2026-11-13

calendula requires io-gcal 0.4, io-http 0.7, io-msgraph 0.8, io-pim-discovery 0.12, io-vdir 0.3 and io-webdav 0.8, and io-oauth 0.6 for its live suites, all released on 2026-11-13. Cargo.lock moves those seven and gains psl and psl-types, which io-pim-discovery 0.12 pulls for its provider rules. One io-http (0.7.0) and one io-oauth (0.6.0) remain; the tree holds no io-sasl. The pimalaya-cli pin, io-pimdir 0.12, pimalaya-config 0.4 and pimalaya-stream 0.3 stay as they were; pimalaya-stream 0.3.1 still builds io-proxy 0.1.1, io-proxy 0.2.0 waiting on its own release.

- **gcal enums.** The 18 enums io-gcal reads from the server gained `Unknown(String)` and lost `Copy`. src/gcal/render.rs borrows them and answers an unknown value as the API spells it; the rows of `gcal calendars`, `gcal acl list` and `gcal instances` hold strings in place of `&'static str`, defused when printed since they can now carry a server string. The `acl` flags still refuse a value calendula does not know.
- **gcal stash.** `GcalEvent::from_ical` answers `StashFull { event, left_out }` once the unmanaged lines pass the 30,000 octets io-gcal keeps the two stashes within. The projection spec already ruled that an oversized line cannot fail the whole write, so `create_item` and `update_item` write the event it carries and log a warning made of the error's message, which names every line and component left out. `a_stash_over_what_google_stores_is_written_without_the_lines_left_out` failed before the change with that message as the create's error.
- **gcal ids.** calendula never encoded an id, and the spec now says so: io-gcal encodes each as one path segment. `an_id_holding_a_hash_or_an_at_names_its_own_resource` reads and deletes `2026#1@x` in `en.usa#holiday@group.v.calendar.google.com` over a scripted server and reads `%23` for each `#` in both request lines, `@` staying as it is.
- **gcal errors.** `GcalSendError::Api` carries a `GcalApiError`; calendula matches on no variant and still prints `HTTP <status>`. It backs off nowhere and keeps no sync token, so `retry_after` and `is_sync_token_expired` have no caller here.
- **msgraph pages.** `list_calendars` built the request of a next link by hand (`MsgraphSend::get` over a parsed URL), which no origin check covered. It now goes through `MsgraphClientStd::page_get`. `a_calendar_page_link_off_the_graph_origin_is_not_followed` failed before the change, the listing answering both pages. The request went over the connection already open to Graph, so no token left for another host.
- **msgraph projection.** No calendula code: an `RRULE` Graph would only approximate is refused by io-msgraph, a VEVENT with neither `DTEND` nor `DURATION` takes the RFC 5545 default, ids are encoded by the crate. `MsgraphSendError::Api` is boxed and the listings are `MsgraphPage<T>`, neither named here.
- **CalDAV.** No code: a listing that is not one whole multistatus and a delete naming failed members fail through the `?` already there, and `create_item` already returned the id io-webdav gives, now the percent-encoded segment. The name calendula mints holds unreserved characters and `@` alone, which the URL keeps as written.
- **vdir, discovery, OAuth.** No code. `VdirClient` kept its signatures. calendula calls `DiscoveryWebdavClientStd::resolve` alone and prints no discovered config, the wizard having left on 2026-11-12, so there is no `trust` to show. The live suites call `Oauth20ClientStd::connect` and the client credentials and JWT bearer grants, whose signatures did not move.
- A stream answering canned responses is written twice, in the tests of src/gcal/backend.rs and src/msgraph/backend.rs, each module compiling with its own feature alone.

Checked: `cargo fmt --check`; `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` over the eleven feature sets of .github/workflows/tests.yml (322 tests pass with the default features and with `--all-features`, 14 ignored, the live suites compiled and not run); `cargo-deny check` (advisories, bans, licenses and sources ok, the duplicates still getrandom and syn).

Capabilities moved: projection (the stash), backends (gcal ids, the CalDAV create and listing, the Graph pages), commands (the gcal family).
