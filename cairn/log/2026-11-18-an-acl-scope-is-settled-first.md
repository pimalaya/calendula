---
cairn: log
change: an-acl-scope-is-settled-first
landed: 2026-11-18
---

# An ACL scope is settled first

Landed without a proposal: CAL-19 of the deep analysis, read then, fixed with a test now.

- `GcalAclRuleArgs::scope` refuses a `user`, `group` or `domain` scope with no `--value`, which posted a scope with no value, and a `default` scope given one, which dropped the value and granted everyone, as `acl update` settles both before any request. Test `a_scope_and_its_value_are_settled_before_any_request` (src/gcal/acl.rs).
- `gcal free-busy` refuses a missing `--from` or `--to` before any request, where it built a request with neither bound (the analysis's `[plausible]` half). The refusal runs inside `execute`, after the client opens; no unit test covers it, a test client needing a scripted Google stream the command does not take.

Capabilities moved: commands.
