---
cairn: change
id: cli-alignment-leftovers
status: active
created: 2026-10-18
---

# What calendula still owes the family alignment

## Why

The cross-repo alignment audit of 2026-10-18 (CLI_ALIGNMENT_PLAN.md, which kept the item ids of the 2026-08-28/29 plan) found these items still open in calendula. Each was re-checked against the code on 2026-10-18.

- **C3**: a write on the pimdir backend only queues the change locally, yet the shared confirmation says "Event `<id>` successfully created" (src/shared/event/create.rs:138), as if the server had it.
- **C4**: `pimdir.account` is taken as configured or left unset (src/pimdir/client.rs:64, src/pimdir/backend.rs:451-457). himalaya derives it from the store when one account synced it and errors by name when several did (himalaya/src/pimdir/client.rs:225).
- **B3**: the `caldav` block has no `alpn` key, the client pins `http/1.1` (src/caldav/client.rs:146). This reverses what config-and-output-alignment kept out on 2026-08-29.
- **F1**: `CalendulaCli` and `CalendulaCommand` (src/cli.rs:58, :95) are `Cli` and `Command` everywhere else.
- **F7**: no `footer!` on the clap command, which the rest of the family carries.
- **G2**: each backend built at runtime gets its own `SecretResolver` (src/shared/client.rs:533-540), so an account whose backends share a credential command still runs it once per backend outside `account check`.
- **A2**: the `[patch.crates-io]` git pins wait for releases.

F2 (list the account names when one is unknown) is already a task of first-run-wizard-frame and is not repeated here.

pimalaya-cli will take over two local copies once its changes land: wizard-save-layer replaces `ExistingConfig`, `append_or_print` and `print_saved` (src/wizard/configure.rs:91, :220, :246), shared-account-resolution replaces `resolve_account` and `load_config` (src/cli.rs:141).

## What

Close each item above, one task each. C3 and C4 are pimdir behaviour, so they fold into the pimdir spec; the rest are naming, config and dependency hygiene.
