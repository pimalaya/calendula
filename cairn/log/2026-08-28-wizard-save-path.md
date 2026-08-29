---
cairn: log
date: 2026-08-28
change: wizard-save-path
---

# The wizard offered to hide the configuration it was run beside

Found while explaining why a ~/.calendularc holding four accounts had gone unnoticed during the pimdir run. calendula reads it correctly; the wizard is what does not, and the way it does not is worse than not reading it.

## What landed

**[src/wizard/discover.rs](../../src/wizard/discover.rs) seeds the save prompt from what is in play.** `--config` when one was given, else `TomlConfig::first_valid_default_path`, else the platform path.

Until now it proposed the platform path unconditionally, and since the loader takes the first existing default path and merges nothing, accepting that default on a machine configured through ~/.calendularc wrote a file that silently hid every account in the rc.

The clobber guard could not fire: the proposed path was the one that did not exist. Reproduced against a fake home, where `account list` with both files present reported only the XDG one's accounts.

**`--config` reaches the wizard at all.** `execute` handed `cli.config_paths` to every subcommand and nothing to the wizard, so the flag was silently dropped on the one path where a user is most likely to pass it.

## Capabilities moved

- wizard: the save prompt's default path
