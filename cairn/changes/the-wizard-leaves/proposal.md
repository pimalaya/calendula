---
cairn: change
id: the-wizard-leaves
status: landed
created: 2026-11-12
---

# The wizard leaves

## Why

The owner decided on 2026-11-08 that the Pimalaya CLIs drop their configuration wizards; Ortie and Himalaya already did (himalaya cairn/log/2026-11-11-the-wizard-leaves.md). This is change 3 of the-family-standards. Keeping calendula's means owning discovery of providers, a keyring dialogue, a review UI and a TOML writer in a binary whose job is to drive calendars, for a flow the shared first-run welcome of pimalaya-cli replaces family-wide.

## What

- Delete src/wizard/ and the `configure` command (alias `wizard`), with the `wizard` cargo feature, its place in `default`, and the pimalaya-cli `wizard`, `caldav` and `spinner` features it enabled. pimalaya-cli `prompt` stays: the composer asks `Re-edit the item?` through it. shellexpand leaves, the wizard being its only user.
- Bare `calendula` prints the help, whether or not a configuration exists. A command needing one and finding none fails with `No configuration found at <path>, write one by hand from the sample: https://github.com/pimalaya/calendula/blob/master/config.sample.toml`, the one `NO_CONFIG_HINT` left, with no cfg split; `account check` fails with the same message. `load_config`, `resolve_account` and the client builders no longer take the printer, which only the offer needed.
- Drop the serializer the wizard needed: `AccountConfig::render`, `RENDER_ORDER`, `ENDPOINT_KEYS`, `is_default` and the other `skip_serializing_if` helpers, `CONFIG_SAMPLE_URL` and every `Serialize` derive on a configuration type, no JSON output carrying one. Drop `account::check::all_ok`, which the wizard alone called; `check_account` turns private, and `caldav::client::resolver` folds into its one caller.
- io-pim-discovery stays for `caldav.discover`, which resolves a bare domain through RFC 6764 on every run: an optional dependency of the `caldav` feature with `rfc6764` and `stream` only, `autoconfig` and `pacc` leaving with the wizard, the TLS forwards weak.
- Drop the `calendula-configure` JSON Schema.
- README, CONTRIBUTING, config.sample.toml, the src/main.rs header, CHANGELOG and MIGRATION.md stop pointing at the wizard.
- first-run-wizard-frame is abandoned; the landed wizard changes (wizard-discovery-first, wizard-runs-when-unconfigured, wizard-save-path) move to changes/archive/, beside the-wizard-picks-and-asks-once.

## Scope / non-goals

pimalaya-cli and pimalaya-config stay on 0.3: the first-run welcome replacing the missing-configuration error, `-c` taking one path per flag and typed account resolution are the-shared-kit's. secret-resolver-per-account stays where it is: the one resolver per account serves `account check` and the backends, not the wizard alone. `account check` keeps every backend check it runs.
