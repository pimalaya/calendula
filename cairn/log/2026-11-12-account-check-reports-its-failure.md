---
cairn: log
change: account-check-reports-its-failure
landed: 2026-11-12
---

# A failed account check prints its report

From the cross-check of the-family-standards with cardamum (account-check-reports-its-failure, 57e30dd) and Himalaya (himalaya/src/account/check.rs): the family output contract prints a result whose outcome is not a full success and exits 2, a failed `account check` being its example.

- `AccountCheckCommand::execute` prints the report whatever the outcome, then returns pimalaya-cli's `PartialOutcome` when a backend failed, which `ErrorReport::eval_coded` turns into exit 2 with nothing printed after the report. It replaces the 2026-10-30 behaviour (account-check-fails-on-a-failed-backend): exit 1 with `Account <name> failed its check: <backend>: <reason>; ...` as the failure object, which hid from a `--json` caller which backends passed. 0.2.0 printed the report and exited 0. An account with no backend `--backend` allows still fails with exit 1. The report is the JSON result in both outcomes; the help says so.
- `BackendCheck` gains `code`, as Himalaya's: the stable code of the failure (pimalaya-cli `code_of`), `null` when the backend passed or its failure carries none, so `{"account", "backends": [{"backend", "ok", "code", "error"}]}`. A failed pimdir check now names `store-uncreated`, `store-busy` and the other store codes to a script; the text shows no code, as Himalaya's. The 2026-10-30 failure object never carried one: a d10d119 build (exported with `git archive` to a scratch directory) printed `"code":null` over the account below, its message folding each failure into a new error, so the code is new to a script, not restored. cardamum's `BackendCheck` has no `code`.
- Over an account whose vdir home exists and whose pimdir root does not, the text is `Account:  test`, `vdir:     OK`, ``pimdir:   FAIL (Open pimdir store `<root>`: Pimdir store has no schema yet: its owner has to create it first)``, exit 2; the check created nothing under the root.
- tests/account_check.rs runs the built binary over that account: `a_failed_backend_prints_the_report_and_exits_2` and `a_failed_backend_is_the_json_result_with_its_code` were seen failing on the old code (exit 1), the second also on the report before `code` (the vdir verdict lacking `"code":null`); it reads `"code":"store-uncreated"` on the pimdir verdict. `a_passing_check_exits_0` (`--backend vdir`) passes on both as the guard. The unit test, renamed `a_failed_backend_prints_the_report_and_fails_the_check`, reads the `vdir: FAIL (` line and `PartialOutcome`, seen failing on the old code too. No network: the failing backend is a local store.

Capabilities moved: commands (cairn/spec/commands.md: "account check fails when a backend fails" replaced, with the scenario "One backend of two fails").

Verified: `cargo fmt`; `cargo clippy --all-targets -- -D warnings` with the default features and `--all-features`; `cargo test --all-features` (263 unit, tests/account_check.rs 3, tests/event_list.rs 4, tests/first_run.rs 5, tests/read.rs 2); `json-schema -d` against d10d119's schemas differs in calendula-account-check alone, `code` added to each verdict (required, the `ErrorCode` vocabulary or `null`).
