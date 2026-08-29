---
cairn: log
date: 2026-08-28
change: wizard-runs-when-unconfigured
---

# The wizard runs when asked, or when nothing is configured

Bare `calendula` ran the wizard whether or not a configuration existed, and the wizard had no command of its own, so a user with four accounts in ~/.calendularc was dropped into a first-run flow every time they typed the binary's name.

himalaya and cardamum had already settled this the other way, together; this is calendula joining them.

## What landed

**[src/wizard/configure.rs](../../src/wizard/configure.rs), the command.** `calendula configure` (alias `wizard`) runs the wizard on demand.

It reads the configuration already there, suffixes the derived account name until it is free, claims the default only when no account does, and then saves to a target that does not exist, appends to one that does, or prints.

Appending is a plain text append, so comments and hand formatting survive.

**[src/cli.rs](../../src/cli.rs) raises the offer, [src/main.rs](../../src/main.rs) gates it.** `offer_configuration` welcomes and offers; `meet_bare_invocation` raises it only when no configuration is found, `--account` is absent, the output is not JSON and stdin is a terminal, and prints the help otherwise.

`load_config` raises the same offer for a command that needs an account, then re-reads, since the wizard may have printed the account rather than written it.

**[src/wizard/discover.rs](../../src/wizard/discover.rs) generates and nothing more.** `run` returns the name and the account it built; the welcome, the path prompt, the save and the document type left with it.

**[src/config.rs](../../src/config.rs) renders an account.** `AccountConfig::render` emits one `[accounts.<name>]` block with the groups in reading order and the endpoint first inside each, so a generated `caldav.server` does not sit under the credential authenticating against it.

`CONFIG_SAMPLE_URL` moves here too, naming the configuration rather than the wizard, which is also what lets a build with no wizard-capable backend say so.

**One duplicate went.** `account list` carried a second `load_config` of its own; it uses the shared one now, so a missing configuration raises the offer there as well.

## Capabilities moved

- wizard: the trigger, the welcome, the account naming, and what happens to the generated account
- commands: `configure`
