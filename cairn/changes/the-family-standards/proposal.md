---
cairn: change
id: the-family-standards
status: active
created: 2026-11-12
---

# calendula takes the family standards

## Why

Ortie and Himalaya moved between 2026-11-08 and 2026-11-12 onto a standard the whole family follows (ortie and himalaya cairn/log from 2026-11-08, pimalaya-cli and pimalaya-config 0.4): no wizard; the pimalaya-cli 0.4 and pimalaya-config 0.4 kit (the first-run welcome, `-c` one path per flag, typed account resolution, plain text through `Columns`, `Fields`, `Records` and `Trees`, the shared REPL and MCP server); a tested sample shipped in share/doc; dependencies naming their features with `cargo deny` clean; every feature set clippy-clean; live provider suites; crates.io dependencies. No app releases alone: calendula has to reach the same level before the family releases.

calendula 0.2.0 (8ffae21) is entirely on the earlier model: a wizard offered from a bare invocation and from a missing configuration, pimalaya-cli and pimalaya-config 0.3 behind git pins, `CALENDULA_CONFIG` and a `:`-split `-c`, bordered and coloured comfy tables with about 25 colour keys and `-w`, no REPL or MCP server, a prose sample neither tested nor shipped, dependencies on their defaults, no live suite and no clippy matrix.

## What

Ten changes, landed in this order, each with its own proposal, tasks and delta, each its own commit:

1. published-dependencies: every git pin dropped for the released crate (ical-rs 0.7, io-gcal 0.3, io-http 0.6, io-msgraph 0.7, io-pimdir 0.10, pimalaya-cli 0.3.2); first, so the API moves surface before the large edits.
2. events-only (the 2026-10-27 draft, unblocked by the owner on 2026-11-12): `todo` and `journal` leave for unfog and the journal CLI; a leftover `todo` or `journal` table is refused by name, pointing at the CLI that took it over, as the family refuses removed keys.
3. the-wizard-leaves: the wizard, `configure`, the offer, the `wizard` feature, io-pim-discovery and the serializer it needed; first-run-wizard-frame abandoned.
4. the-shared-kit: pimalaya-cli and pimalaya-config 0.4 in one step, since neither half compiles alone: `Cli` and `Command`, `footer!`, `ConfigFlags`, `TomlConfig::load` and `take_account`, the welcome, and plain text output (no table, colour key or `-w`).
5. the-shared-repl-and-mcp: `calendula repl` and `calendula mcp` on the shared core, writes behind `--allow-write`.
6. the-sample-is-tested-and-shipped: config.sample.toml in Himalaya's shape, tested through `pimalaya_config::sample::variants`, installed as share/doc/calendula/config.sample.toml.
7. dependencies-trimmed: every dependency `default-features = false` naming its features, `idna_adapter`, `SimdCpu::install_detected`, deny.toml targets and licences, `cargo deny check` clean.
8. every-feature-set-builds-clean: clippy `-D warnings` over the feature matrix, checked in CI.
9. live-provider-tests: Google, Microsoft and Fastmail suites against the built binary, anonymous on every push, credentialed read-only on the test accounts.
10. docs: README, MIGRATION.md 0.2 to 0.3, CHANGELOG, the src/main.rs header and CONTRIBUTING; cli-alignment-leftovers closed.

`event list` prints the way Himalaya's newest `envelope list` does (owner, 2026-11-12): one aligned line per event with no header, the summary last, then a blank line and a footer naming the calendar and the page. Himalaya's design was still moving on 2026-11-12 (uncommitted), so the-shared-kit re-reads Himalaya's landed version before writing calendula's.

## Out

jmap-backend and event-alarm-flag stay independent. The version bump to 0.3.0 is the owner's call.
