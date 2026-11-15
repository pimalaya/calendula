---
cairn: change
id: the-next-page-is-a-token
status: landed
created: 2026-11-12
---

# The next page is a token

## Why

The cross-check of the-family-standards with cardamum and Himalaya on 2026-11-12 found calendula's listings printing `nextPage` as a number and resuming through `-p/--page N` only. pimalaya-cli's output spec ("Paging resumes") asks a command that can stop before the end of a listing to print `nextPage` and take that value back through `--page-token` (pimalaya-cli `PageTokenArg`); cardamum's `card list` and Himalaya's `envelope list` and `envelope search` print it as a string, the next page number, and take it back so. A caller written against one of them cannot resume a calendula listing.

## What

- `event list` and `item list` print `nextPage` as a string, the next page number, and take it back through `--page-token`. A token that does not read as a page number is refused, naming it, before the backend is reached; the token wins over a `-p` given beside it, as in cardamum's "card list" and Himalaya's "envelope list" requirements.
- The footer is unchanged: `Next page: --page <n>`, the token being the page number.
- `event find` keeps printing `nextPage` as `null`, now typed as a string: it reads the calendar whole and never stops before the end, so it takes no token.
- JSON schema: `nextPage` of `event list`, `event find` and `item list` becomes a string or `null`. It breaks a consumer reading a number, which only an unreleased build printed: 0.2.0 printed no `nextPage`.
- Spec (commands), CHANGELOG (the unreleased Added line amended), log.

## Scope / non-goals

`pimdir search` prints `more` and pages through `-p` alone, as Himalaya's and cardamum's `pimdir search` do; moving it onto `nextPage` is a family change for the three, not this one.
