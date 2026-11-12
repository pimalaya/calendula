---
cairn: log
change: the-next-page-is-a-token
landed: 2026-11-12
---

# The next page is a token

From the cross-check of the-family-standards with cardamum and Himalaya: pimalaya-cli's output spec ("Paging resumes") asks a listing that can stop before its end to print `nextPage` and take it back through `--page-token`, which cardamum's `card list` and Himalaya's `envelope list` and `envelope search` do, `nextPage` being the next page number as a string.

- `event list` and `item list` flatten pimalaya-cli's `PageTokenArg` and print `nextPage` as `Option<String>`, the next page number. The token is read before anything else in the command, so one that does not parse as a `u32` is refused before the backend is reached (the error `Invalid page token` naming `x2` in backticks, exit 1, the parse error as its cause); `0` reads as page 1, as `-p 0` does. The token wins over a `-p` given beside it, as in cardamum's "card list" and Himalaya's "envelope list" requirements; a first cut refused the pair as a clap usage error, dropped to keep the family's one contract. A listing the command leaves unpaged (`-s 0`, `event list` over a window with no `-s`) ignores the token as it ignores `-p`.
- `Listing::write` (src/shared/listing.rs) takes the token as `Option<&str>`; the footer still prints `Next page:  --page <n>`, the token being the page number.
- `event find` shares `EventListOutput`, so its `nextPage` is typed a string too and stays `null`: it reads the calendar whole, so it has no page to resume and takes no token.
- JSON schemas: `json-schema -d` (debug build, every feature) before and after differ in `nextPage` of calendula-event-list, calendula-event-find and calendula-item-list alone, from an integer or `null` (`uint32`) to a string or `null`. Breaking for a consumer reading a number, which only an unreleased build printed: `nextPage` arrived with the-shared-kit (4865362) and 0.2.0 printed none, so the CHANGELOG amends its unreleased Added line rather than marking a break.
- tests/event_list.rs: `a_listing_resumes_at_the_next_page_it_names` (`"nextPage":"2"`, then `--page-token 2` listing `c` with `null`, on `event list` and `item list`), `a_page_token_that_is_no_page_number_is_refused`, `a_page_token_wins_over_a_page` (`-s 2 -p 1 --page-token 2` listing `c`, both families) and the text footer read through `--page-token 2` in `an_event_list_prints_a_header_one_line_per_event_and_a_footer`; the four were seen failing on the old code (`unexpected argument '--page-token'`, `nextPage` the number 2), and `a_page_token_wins_over_a_page` also on the first cut, which exited 2. The old `an_event_list_names_the_next_page_in_json` is folded into the first.

Capabilities moved: commands (cairn/spec/commands.md: "A page names the next one" added, with the scenario "A listing resumed from its token").

Verified: `cargo fmt`; `cargo clippy --all-targets -- -D warnings` with the default features and `--all-features`; `cargo test --all-features` (263 unit, tests/event_list.rs 4, tests/first_run.rs 5, tests/read.rs 2).

Left: `pimdir search` prints `more` and pages through `-p` alone, as Himalaya's and cardamum's `pimdir search` do, short of "Paging resumes"; a change for the three.
