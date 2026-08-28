---
cairn: change
change: wizard-save-path
---

# Delta

## ADDED Requirements

(none)

## MODIFIED Requirements

### Requirement: Printed, and saved only on a terminal
The generated configuration SHALL be printed as a TOML document on stdout in JSON mode and whenever stdout is redirected, so `calendula > config.toml` and any script keep working. Only when writing to a terminal SHALL the wizard offer to save it to a file, refusing to clobber an existing file without confirmation, and falling back to printing so the generated document is never lost.

The save prompt SHALL be seeded with `--config` when one was given, else with the configuration already in effect, else with the platform configuration path. The loader takes the first default path that exists and merges nothing, so seeding the platform path at a user whose configuration is `$HOME/.calendularc` would offer to create a second file that silently hides every account in the first, with no overwrite prompt to warn them, the proposed path not existing yet.

## REMOVED Requirements

(none)
