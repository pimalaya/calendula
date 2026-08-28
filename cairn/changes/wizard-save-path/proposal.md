---
cairn: change
id: wizard-save-path
status: landed
created: 2026-08-28
---

# The wizard offered to hide the configuration it was run beside

Bare `calendula` runs the wizard whether or not a configuration exists, which is deliberate: it reads none, generates one, and offers to save it. What it offered was the platform path, `$XDG_CONFIG_HOME/calendula/config.toml`, unconditionally.

The loader takes the **first** default path that exists, in the order `$XDG_CONFIG_HOME/calendula/config.toml`, `$HOME/.config/calendula/config.toml`, `$HOME/.calendularc`, and merges nothing. So for a user whose configuration is `~/.calendularc`, accepting the wizard's default writes a second file that takes precedence and hides every account in the first. The clobber guard cannot fire, since the path it proposes is exactly the one that does not exist yet. Verified against a fake home: with both files present, `account list` shows only the XDG one's accounts.

`--config` was dropped on the same path. `execute` passes `cli.config_paths` to every subcommand and passed nothing to the wizard, so `calendula --config <path>` with no subcommand neither proposed nor wrote that path: a documented global flag silently ignored.

## What changes

The save prompt is seeded with `--config` when one was given, else with the configuration already in effect (`TomlConfig::first_valid_default_path`), else with the platform path as before. A user with an rc file is offered their rc file, which turns a silent shadowing into the ordinary overwrite prompt; a user with no configuration at all sees exactly what they saw before.

## What does not change

Everything else about the wizard: it still reads no configuration, still prints to stdout in JSON mode and whenever stdout is redirected, still refuses to clobber without confirmation, and still falls back to printing.
