---
cairn: log
change: the-family-standards
landed: 2026-11-12
---

# calendula takes the family standards

calendula is on the standard Ortie, Himalaya and cardamum share, in local commits on 2026-11-12 after 8ffae21, each change with its own proposal, delta and log:

1. 321b246 and d09b4d1: the plan, then the cross-review with cardamum and comodoro folded into it.
2. c7e451e published-dependencies: every Pimalaya crate from crates.io.
3. 2b3678f events-only: `todo` and `journal` leave for unfog and journal, a leftover table refused by name.
4. f653c93 the-api-is-unified: the shared API renamed the unified API.
5. ff007c9 the-wizard-leaves: the wizard, `configure` and the `wizard` feature gone, a bare `calendula` printing the help.
6. 4865362 the-shared-kit: pimalaya-cli and pimalaya-config 0.4, the welcome, `-c` one path per flag, typed account resolution, plain text, `event list` paged as Himalaya's `envelope list`.
7. b6875e0 the-sample-is-tested-and-shipped: config.sample.toml in Himalaya's shape, parsed variant by variant, installed in share/doc/calendula.
8. 5da454f dependencies-trimmed: every dependency naming its features, ASCII hosts, `cargo deny` clean.
9. eac0546 a-read-is-printable, d10d119 the-next-page-is-a-token and 5564131 account-check-reports-its-failure: the cross-check with cardamum and Himalaya (a read defused, `nextPage` a string resumed through `--page-token`, a failed `account check` exiting 2 with its report).
10. 4c002b1 every-feature-set-builds-clean: clippy `-D warnings` and the tests over eleven feature sets, in CI.
11. 5017c37 live-provider-tests: Google, Microsoft and Fastmail suites against the built binary.
12. 2f30e35 capability-shared-api: the unified API as the union of the backends, `capability-missing`, `account capability`, the `Backends:` help footer.
13. 21fb50f the-shared-repl-and-mcp: `repl` and `mcp` on pimalaya-cli's shared core, writes behind `--allow-write`, whatever reaches an attendee behind `--allow-send`.
14. This change, release-polish: README, MIGRATION.md, the CHANGELOG netted, CONTRIBUTING.md, the src/main.rs header, the release-on-demand `os` default.

Owner decisions of 2026-11-12: events-only unblocked, unfog and journal being unreleased accepted; the shared API renamed the unified API as Himalaya's, landed between events-only and the-wizard-leaves; Himalaya's capability union for that API; lists printed as Himalaya's `envelope list` (envelopes-print-one-line-each); the MCP server's `openWorldHint` on what reaches people beyond the user, the family rule. The wizards leaving was decided on 2026-11-08.

Left for the owner:

- The `event agenda` grid marks no day since the escapes left; a plain-text marker needs a layout change.
- Google CalDAV refuses the narrowed delegation's scopes (403 insufficientPermissions), so its live suite is anonymous only; a credentialed one needs the full `calendar` scope granted.
- Microsoft Graph answered 502 and 504 on most requests during the live run of 2026-11-12, the calendar list included: rerun before relying on the credentialed step. calendula's Graph listing reads the whole calendar at `$top=500` even for `-s 5`, a candidate for a pushdown.
- `account capability` lists every configured backend, where cardamum lists the resolved one and Himalaya a flat list.
- The removed top-level `table` and `account` tables are refused by name in calendula, ignored in cardamum and Himalaya.
- The pimalaya-cli `[patch.crates-io]` git pin (0fbe5dd, for `records`) waits on the family release publishing pimalaya-cli.
- The version bump to 0.3.0, with the headings and SECURITY.md line it moves.

Still active beside it: jmap-backend, and event-alarm-flag, whose code landed in 4319651 and whose live check is pending.

Capabilities moved: none of its own; each change logged its own.
