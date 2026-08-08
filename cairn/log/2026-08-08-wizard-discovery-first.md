---
cairn: log
change: wizard-discovery-first
landed: 2026-08-08
---

# Wizard: discovery first, print rather than write

Rewrote the wizard on himalaya's model. One prompt now takes an email address, a server URL or a folder path, and its shape orients everything; the four questions that came before any work (confirm, account name, email, backend) are gone. An address runs io-pim-discovery's parallel discovery under a short deadline, so one unreachable endpoint cannot stall the prompt, and each distinct context root becomes one entry with the capabilities of every mechanism that named it folded together. The authentication scheme is a second prompt offering only what that service advertised, skipped when one qualifies, and credentials come from pimalaya-cli's keyring and token pickers, which is how an OAuth broker reaches the configuration without calendula running a grant.

The account is tested before anything is emitted, with the same check `account check` runs, and the failure names each backend that failed and why. What comes out is a TOML document on stdout, so `calendula > config.toml` works; saving is offered only when stdout is a terminal, refuses to clobber without confirmation, and falls back to printing so the document is never lost.

Kept one deliberate deviation from himalaya, specified rather than silent: a typed server URL is configured as given. himalaya refuses hand entry because mail providers are near-universally discoverable, but Radicale, Baikal and self-hosted Nextcloud routinely publish neither an SRV record nor a `.well-known` redirect, and a wizard that could not configure them would be one most self-hosters cannot use. `caldav` and `caldavs` are accepted as aliases for `http` and `https`; any other scheme is rejected by name.

A typed folder is detected as a vdir home or a pimdir store from its own markers, pimdir first because a store also holds subdirectories and testing vdir first would misread every store as a home. The prompt only appears when both backends are compiled in and the directory is genuinely ambiguous, which an empty one is.

Bare `calendula` now runs the wizard. A command finding no configuration fails pointing at it rather than running it: the wizard prints a document instead of writing one, so it cannot hand a configuration back to a command already underway. `account configure` was removed for the same reason, and `Config::write` and the direct toml dependency went with it.

Spec updated: wizard (ADDED, the whole capability), commands (MODIFIED, the account family lost `configure`).
