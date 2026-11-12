---
cairn: log
change: release-polish
landed: 2026-11-12
---

# Release polish

Change 11, the last of the-family-standards: the documents a release reader meets say what the code does. No behaviour moved.

- README.md: eight Features of one line each (readme-008: the unified API as the union, the remote backends, the local stores with pimdir, the tested sample, REPL, MCP server, JSON output, Full client), where sixteen bullets and a nested TLS list stood; the provider subsections nested in the table of contents; the RFC rows in order, without code spans, RFC 7986 added for the `CONFERENCE` an online meeting link is read from; Configuration splitting Ortie's paragraph out, naming the three CalDAV routes, the paths bare, `account capability`; Usage pointing at `--help`, a REPL and an MCP subsection of a sentence and a block each. The Apple and Posteo snippets set `caldav.discover` beside `caldav.server`, which `connect` (src/caldav/client.rs) ignores, `server` winning: they keep `server`. The usage line `item read --calendar personal event-1.ics` failed on vdir, whose ids drop the extension: ids are placeholders now. Every command line was run on a scratch vdir account with a fresh debug build (the gcal and pimdir ones parsing, then failing on the missing block), the help of each command named read.
- MIGRATION.md: one section for a 0.2 user, Himalaya's shape (events only, configuration and first run, the pimdir store, text output with jq recipes, JSON output, commands that now fail earlier, what is new). Three sections left: "Error codes" (0.2.0 printed no `code` at all, so `store-newer` and `store-stale` never shipped), "pimdir reply and cancel" and "To-dos" (`x-vtodo`), both about commands or refusals 0.2.0 did not have, each checked with `git grep` on v0.2.0. Added: the default paths read as layers, which 0.2.0 did not (pimalaya-config 0.2.0 `from_default_paths` loads the first one found; a home holding config.toml and .calendularc now lists both files' accounts). The jq recipes ran against `--json` of a scratch account. Each refusal named was run: `table`, `account`, `todo`, `journal` by name, the colour keys as unknown fields, `-w` as a clap error.
- CHANGELOG.md `[Unreleased]`: 26 Added, 34 Changed, 4 Removed, 21 Fixed, from 31, 39, 4 and 24. Folded into their final state: the error codes into one entry, `nextPage` with `--page-token`, the `pimdir reply --recurrence-id` entry, the missing-body fix and the source rule into `pimdir reply`; the empty-value, composer, local-zone and `--duration` entries into the field flags; `onlineMeetingUrl` into the event JSON entry; the `event list` table times and the agenda escapes into the plain text entry; the store refusal wording into storage-draft-09; the dependency bumps (pimalaya-cli and pimalaya-config 0.3, io-pim-discovery 0.9, ical-rs 0.6, io-msgraph 0.6, io-pimdir 5c18a02, pimalaya-cli 55b6693) into one entry at the final versions, their behaviours kept without them. Gone as fixes to what 0.2.0 never shipped: the `pimdir search` lines and the `pimdir reply` body check. Added: the layered default paths (**Breaking.**) and the rustls 0.23.46 bump of b203fdf (RUSTSEC-2026-0346 to 0349). `-w` is removed from the three lists 0.2.0 had it on. Released sections untouched.
- CONTRIBUTING.md: the feature table with `gcal`, `msgraph` and `vendored` rows and the real default set (every backend, `rustls-ring`, `vendored`); a network backend needing one of `rustls-ring`, `rustls-aws` or `native-tls`, several allowed, where it said exactly one; the eleven sets of the `features` job as the loop to run; the live suites, their modules, variables and the `RUST_LOG` ceiling; the patch rule as packaging words it.
- SECURITY.md kept (0.2.x supported, 0.1.x not): it is right for the released line. Cardamum's reads `0.x.x`, Himalaya's and Ortie's `3.x.x`.
- src/main.rs header: io-msgraph in the stack, the five backends, both projected backends, the session refusals and the MCP tool listing and ATTENDEE guard, the tested and shipped sample and the refused tables, the failure codes, exit 2 and receipts, the paging and the defused read.
- .github/workflows/release-on-demand.yml: the `os` input defaults to `ubuntu-24.04`, an option it lists and the runner pimalaya/nix's releases.yml builds Linux on; actionlint flagged `ubuntu-latest` on the old file and passes on every workflow now. Ortie's and Himalaya's same file carry the same `ubuntu-latest` default, as do cardamum's, comodoro's and the other apps', not edited.
- event-alarm-flag stays active, its tasks noting the code landed (4319651) and the live check pending.

Capabilities moved: none.

Verified: `cargo fmt --check`; `cargo clippy --all-targets -- -D warnings` with the default features and `--all-features`; `cargo test --all-features` (268 unit tests, 41 integration tests, the 14 live tests ignored); actionlint with shellcheck on .github/workflows; no em dash and no hard-wrapped paragraph in the markdown touched; `cargo doc --no-deps` without a warning on the header's links; `nix build .#default` succeeded, its binary printing `calendula v0.2.0` and its share/doc/calendula/config.sample.toml identical to the repository's; `sh cairn/verify.sh` conformant.

Left:

- The release renames MIGRATION's `0.2.x -> unreleased` and the CHANGELOG's `[Unreleased]`, and moves SECURITY.md's supported line.
- cairn/spec/config.md calls `caldav.discover`, `server` and `home` mutually exclusive, but a block setting several is accepted, `home` then `server` winning.
- `event agenda` stamps an event at `09:00` for `DTSTART:...T090000Z` under CET, where `event list` prints the local `10:00`.
- `calendula vdir --help` still says "the shared calendar API", a word the-api-is-unified left.
- `item list --json` prints `contents` as an array of byte numbers.
- cairn/spec/projection.md names gcal alone; Graph's projection lives in backends.md.
- release-on-demand.yml offers `macos-13` and `macos-14`, where pimalaya/nix's releases.yml builds on `macos-15` and `macos-15-intel`.
