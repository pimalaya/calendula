---
cairn: log
change: a-read-is-printable
landed: 2026-11-12
---

# A read is printable

From the cross-check of the-family-standards with cardamum and Himalaya (cardamum every-text-output-is-printable, eb79dff; Himalaya's `sanitize_body`, himalaya/src/shared/message/read.rs).

- `event read` and `item read` printed the stored iCalendar raw in text mode, as the spec then asked, so an event carrying an escape sequence or a bidi override drove the terminal of whoever read it. Their text view now goes through `printable` (src/shared/item/read.rs): CRLF read as LF, every other control character but tab and line feed (C0, DEL, C1) and the bidi embeddings, overrides and isolates (U+202A to U+202E, U+2066 to U+2069) replaced with U+FFFD, as Himalaya and cardamum do. `--json` keeps the stored text (lossily decoded as UTF-8, as before). tests/read.rs stores a vdir event with CRLF line ends, an OSC 52 clipboard write in its summary, U+202E in its location and a description folded on a tab: `a_read_prints_the_stored_body_defused` was seen failing on the raw view (`event read` in the test, `item read` by hand on a scratch vdir with the old binary printing ESC, BEL, U+202E and CRLF), and `a_read_keeps_the_stored_bytes_in_json` passes on both. The help of both reads says so.
- A `build` stays byte-exact, on stdout as in the `-o` file: its iCalendar is the caller's source or composition, printed to be piped into `event create -` or `item create -`. calendula has no `--raw` flag.
- The sweep of the other text outputs: the descriptions (`calendar list`, `caldav list`, `gcal calendars`, each line through `sanitize::text`), `gcal settings` (one-line values), `pimdir status` (account, calendars, parked errors), `pimdir search`, `gcal free-busy`, `gcal instances`, `gcal acl list`, `gcal colors`, `vdir list`, `account list`, `account check`, the event and item lists, the agenda labels and every receipt were already defused. Two lines were not: the zone the index reads floating times in, under `pimdir search` and `pimdir count`, and the days of their coverage line (`items before <day> are not synced`), both read from the store; they go through `sanitize::text` now, `the_lines_under_a_table_defuse_what_the_store_holds` seen failing first. The composer's validation report prints what the person's own editor wrote, and stays as it is.

Capabilities moved: commands (cairn/spec/commands.md: "Text output is plain text", its first paragraph and the scenario "A read of a body carrying controls").

Verified: `cargo fmt`; `cargo clippy --all-targets -- -D warnings` with the default features and `--all-features`; `cargo test --all-features` (263 unit, tests/event_list.rs 2, tests/first_run.rs 5, tests/read.rs 2). No output type moved, and the 48 schemas `json-schema -d` writes carry none of the changed help text.

Left: the failure a failed command prints (pimalaya-cli `ErrorReport`) carries its message and sources as written, a server's error text included, which the spec's "an error" now names only for an error a record carries; defusing it is pimalaya-cli's. A multi-line `sanitize` in pimalaya-cli would replace the three copies (Himalaya's `sanitize_body`, cardamum's `card read`, calendula's `printable`). Not marked breaking, as in cardamum: a script saving the text of `item read` now gets LF line ends and U+FFFD for a control character, `--json` being the byte-exact path.
