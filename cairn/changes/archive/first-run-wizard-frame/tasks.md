---
cairn: tasks
change: first-run-wizard-frame
---

# Tasks

- [ ] Add a `configure` command (alias `wizard`) running the wizard by name, with no welcome and a non-interactive bail.
- [ ] Move the welcome onto the offer, and give it the configuration path that was looked for.
- [ ] Add `offer_configuration`, raised by a bare `calendula` and by `load_config`, returning whether the wizard ran.
- [ ] Make a bare `calendula` show the help when a configuration exists, or when `--account` names one, instead of always running the wizard.
- [ ] Make `load_config` a hook: offer, re-read the configuration, then fail the ordinary way when nothing landed.
- [ ] Guard both entry points on `stdin().is_terminal()` and `printer.is_json()`.
- [ ] Resolve the target path from `Config::target_path` instead of prompting for it; append to an existing file rather than overwriting.
- [ ] Suffix the account name until free; claim `default` only when no other account does.
- [ ] Name all three resolution failures: the path read, the accounts held, the two ways to pick a default.
- [ ] Report where the account landed, under which name, and what to run next.
- [ ] Give `AccountConfig` the ordering-aware `render`, and drop the `GeneratedConfig` wrapper.
- [ ] Keep the wizard-less build compiling: no wizard means no offer, and the bare invocation keeps naming the missing backends.
- [ ] Tests: a generated account parses back, an appended one keeps the existing account and its comments, a taken name gets a suffix, a missing configuration constrains nothing.
- [ ] Build/test/fmt/clippy; run cairn/verify.sh.
- [ ] Fold into cairn/spec/wizard.md, superseding "The wizard does not serve a running command"; log; land.
