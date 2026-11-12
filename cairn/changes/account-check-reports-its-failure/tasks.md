---
cairn: tasks
change: account-check-reports-its-failure
---

# Tasks

- [x] tests/account_check.rs: an account whose vdir home exists and whose pimdir root does not prints `vdir: OK` and `pimdir: FAIL (...)` and exits 2, in text and as the `--json` result, and `--backend vdir` exits 0; the two failing cases seen failing on the old code (exit 1).
- [x] `a_failed_backend_prints_the_report_and_fails_the_check` (src/account/check.rs) reads the report and `PartialOutcome`, seen failing on the old code.
- [x] `account check` prints its report, then `PartialOutcome` on a failed backend; its help.
- [x] `code` on each verdict, as Himalaya's; `a_failed_backend_is_the_json_result_with_its_code` (`store-uncreated` for the missing root, `null` for vdir) seen failing on the report without it; the d10d119 failure object checked to carry `"code":null`.
- [x] fmt; clippy `-D warnings` default and `--all-features`; `cargo test --all-features`; `json-schema -d` diffed (calendula-account-check alone, `code` added).
- [x] Fold the delta into the spec (commands); CHANGELOG; log; tick the-family-standards; `sh cairn/verify.sh`.
