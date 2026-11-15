---
cairn: tasks
change: the-next-page-is-a-token
---

# Tasks

- [x] tests/event_list.rs: `nextPage` is `"2"` and `--page-token 2` lists the last page on `event list` and `item list`, a token that is no page number is refused naming it, the token wins over a `-p` beside it on both families, and the text footer reads the same through the token; all four seen failing on the old code, the third also on a build where `-p` conflicted with the token.
- [x] `event list` and `item list`: `PageTokenArg`, winning over `-p`, `nextPage` a string; `Listing::write` takes the token.
- [x] fmt; clippy `-D warnings` default and `--all-features`; `cargo test --all-features`; `json-schema -d` before and after diffed (`nextPage` of `event list`, `event find` and `item list` alone).
- [x] Fold the delta into the spec (commands); CHANGELOG; log; tick the-family-standards; `sh cairn/verify.sh`.
