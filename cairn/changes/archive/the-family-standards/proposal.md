---
cairn: change
id: the-family-standards
status: landed
created: 2026-11-12
---

# calendula takes the family standards

## Why

Ortie and Himalaya moved between 2026-11-08 and 2026-11-12 onto a standard the whole family follows (ortie and himalaya cairn/log from 2026-11-08, pimalaya-cli and pimalaya-config 0.4): no wizard; the pimalaya-cli 0.4 and pimalaya-config 0.4 kit (the first-run welcome, `-c` one path per flag, typed account resolution, plain text through `Columns`, `Fields` and `Records`, the shared REPL and MCP server); a tested sample shipped in share/doc; dependencies naming their features with `cargo deny` clean; every feature set clippy-clean; live provider suites; crates.io dependencies; Himalaya's capability union in place of the least-common-denominator rule. No app releases alone: calendula has to reach the same level before the family releases.

calendula 0.2.0 (8ffae21) is entirely on the earlier model: a wizard offered from a bare invocation and from a missing configuration, pimalaya-cli and pimalaya-config 0.3 behind git pins, `CALENDULA_CONFIG` and a `:`-split `-c`, bordered and coloured comfy tables with about 25 colour keys and `-w`, no REPL or MCP server, a prose sample neither tested nor shipped, dependencies on their defaults, no live suite, no clippy matrix, and a least-common-denominator shared API whose partial verbs each backend refuses ad hoc.

Cardamum (cardamum/cairn/changes/the-family-standards) and comodoro run the same move; the three plans were reviewed against each other on 2026-11-12 and agree on the shapes below.

## What

Eleven changes, landed in this order, each with its own proposal, tasks and delta, each its own commit:

1. published-dependencies (landed c7e451e): every git pin dropped for the released crate (ical-rs 0.7, io-gcal 0.3, io-http 0.6, io-msgraph 0.7, io-pimdir 0.10, pimalaya-cli 0.3.2).
2. events-only (the 2026-10-27 draft, unblocked by the owner on 2026-11-12): `todo` and `journal` leave for unfog and the journal CLI; a leftover `todo` or `journal` table is refused by name, pointing at the CLI that took it over, as the family refuses removed keys.
3. the-wizard-leaves: the wizard, `configure`, the offer, the `wizard` feature and the serializer it needed; first-run-wizard-frame abandoned. io-pim-discovery stays behind `caldav`: `caldav.discover` resolves a bare domain through RFC 6764 on every run (src/caldav/client.rs), so only its wizard features (`autoconfig`, `pacc`) leave.
4. the-shared-kit: pimalaya-cli and pimalaya-config 0.4 in one step, since 0.4 no longer carries the table module and neither half compiles alone: `Cli` and `Command`, `footer!`, `ConfigFlags`, `TomlConfig::load` and `take_account`, the welcome, plain text output (no table, colour key or `-w`). It carries the open items of cli-alignment-leftovers (F1, F7, B3 `caldav.alpn`, G2 one `SecretResolver` per account), which is abandoned for it. pimalaya-cli comes from a git revision while `records::Records` is unreleased.
5. the-sample-is-tested-and-shipped: config.sample.toml in Ortie's and Himalaya's shape, tested through `pimalaya_config::sample::variants`, installed as share/doc/calendula/config.sample.toml.
6. dependencies-trimmed: every dependency `default-features = false` naming its features, `idna_adapter`, `SimdCpu::install_detected`, deny.toml targets and licences, `cargo deny check` clean. The packaging requirement on patches takes cardamum's wording: the cairn change adding a patch says what it waits on, Cargo.toml carrying no comment in its dependency or patch sections.
7. every-feature-set-builds-clean: clippy `-D warnings` over the feature matrix, checked in CI as Himalaya's `features` job.
8. live-provider-tests: Google (gcal, CalDAV), Microsoft (Graph) and Fastmail (CalDAV) suites against the built binary, anonymous on every push, credentialed read-only on the test accounts. They come before the capability union so they guard that a served verb prints what it printed.
9. capability-shared-api (owner, 2026-11-12): Himalaya's capability union, as cardamum adopts it, for the API the-api-is-unified renamed the "unified API" (owner, 2026-11-12, landed between events-only and the-wizard-leaves at the owner's request); the change id stays for cross-reference and src/shared keeps its name. One table per backend declares each shared capability at `full`, `partial`, `server` or `none`, seeded from today's per-backend refusals; names are pimdir STORAGE Annex B's with the domain word dropped, `collection` read as `calendar` and `item` as calendula's stored noun `item` (`calendar.create`, `item.add`, `item.update.if-match`), intents bare (`scheduling`, `online-meeting`, `reply`, `cancel`); an unserved verb fails with `capability-missing` before connecting; `account capability`.
10. the-shared-repl-and-mcp (last, as in cardamum, so its tests run on the final outputs): `calendula repl` and `calendula mcp` on the shared core, Himalaya's rule for both. The REPL runs the full `Command` tree, refusing `repl`, `mcp` and the composer (`-i`, `--composer`) by name, an editor fighting the claimed stdin. The MCP server builds tools through `Grammar` (copied from himalaya/src/mcp.rs), lists a tool only when the account's capabilities serve it, gates writes behind `--allow-write` and writes that reach attendees (`--send-updates`, attendee flags, `pimdir reply` and `cancel`) behind `--allow-send` as Himalaya gates sending, marks deletes `destructiveHint`, and sets `openWorldHint` on what reaches people beyond the user (owner, 2026-11-12: the family rule).
11. release-polish: README, MIGRATION.md 0.2 to 0.3, CHANGELOG, SECURITY.md, the src/main.rs header and CONTRIBUTING.

Lists print as Himalaya's `envelope list` does since envelopes-print-one-line-each (owner, 2026-11-12): a header (`Account: <name> (<backend>)`, `Calendar:`), headerless `Columns` rows with the free text last and `-` for an empty cell, then a footer (`Page: <n>, events <a> to <b>[, the last]`, `empty`, or `all` when unpaged; `Next page: --page <n>`), the values in one column. `, the last` and `Next page` are exact: no backend cursor reaches the shared listing, CalDAV, Graph and vdir paging the whole collection locally (`paginate`, src/shared/client.rs) and gcal and pimdir stopping at the page, so those two read one item past it rather than guess from a full page. JSON loses the `Paginated` wrapper, its keys unchanged. Wide lists other than these print as `Records`, records as `Fields`. Cardamum proposes lifting the header and footer into pimalaya-cli before either the-shared-kit lands, one session on ../cli at a time; calendula copies whichever lands first.

Times: a true instant prints RFC 3339 UTC; a calendar time keeps its own form (an event's `start` and `end` may be floating or a date, which RFC 3339 cannot say). pimalaya-cli's output spec says only that a timestamp read from content keeps its offset; the wording is proposed to the owner for it, not edited from here.

## Out

jmap-backend and event-alarm-flag stay independent. The version bump to 0.3.0 is the owner's call.
