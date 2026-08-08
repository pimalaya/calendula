---
cairn: spec
capability: wizard
status: current
---

# Wizard

Bare `calendula` (no subcommand) runs the interactive configuration wizard. It discovers an account and prints it as a ready-to-save TOML fragment on stdout, writing nothing on its own. Prompts render on stderr, so redirecting stdout into a configuration file works directly.

### Requirement: Input orients the flow
A single prompt SHALL accept an email address (or a bare domain), a `scheme://` server URL, or a local folder path. An email or bare domain runs io-pim-discovery's parallel discovery; a server URL names the CalDAV context root outright; a folder is a local vdir home or pimdir store. The wizard SHALL NOT ask which backend to configure, and SHALL NOT prompt for an endpoint field it could derive.

### Requirement: A typed server URL is configured as given
A `http`, `https`, `caldav` or `caldavs` URL SHALL be taken as the context root and only its credentials prompted, with every authentication scheme offered since nothing was advertised. `caldav` and `caldavs` are accepted as aliases for `http` and `https`, since that is how a DAV endpoint is often written down; any other scheme SHALL be rejected by name.

This is calendula's one deliberate deviation from himalaya's wizard, which refuses hand entry entirely. Mail providers are near-universally discoverable; CalDAV servers are not. Radicale, Baikal and self-hosted Nextcloud routinely publish neither an SRV record nor a `.well-known` redirect, and refusing them would put the servers calendula's users most often run out of reach.

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline, so a single unreachable endpoint (a firewalled port, a black-hole host) cannot stall the interactive wizard. Each mechanism runs independently; any that has not reported by the deadline is abandoned, and only what completed in time is offered.

### Requirement: One entry per service, then auth
The discovery list SHALL show one entry per distinct context root, folding the capabilities of every mechanism that named it: SRV and PACC routinely agree on a root, and offering it twice is a choice with no difference. After an entry is picked, the authentication scheme SHALL be chosen in a second prompt offering only what that service advertised, skipped when only one qualifies. When nothing was advertised, every scheme SHALL be offered rather than none.

### Requirement: OAuth folds into the API token
calendula runs no OAuth 2.0 grant itself, so OAuth SHALL NOT be a standalone list entry. It folds into the API-token credential prompt, which offers the OS keyrings (for a token the user generated) and the OAuth token brokers (Ortie, pizauth, oama) together, the brokers appearing only when the service advertises OAuth.

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the input (the domain's first label, or the folder name) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key. The generated account SHALL be left non-default, so merging the fragment into a configuration that already has a default does not hijack it.

### Requirement: Connection tested before printing
The account SHALL be tested before the fragment is printed, so a bad credential or endpoint stops the wizard instead of yielding a configuration that cannot connect. The test is the same one `account check` runs. Its failure SHALL name each backend that failed and why.

### Requirement: Printed, and saved only on a terminal
The generated configuration SHALL be printed as a TOML document on stdout in JSON mode and whenever stdout is redirected, so `calendula > config.toml` and any script keep working. Only when writing to a terminal SHALL the wizard offer to save it to a file, defaulting to the platform configuration path, refusing to clobber an existing file without confirmation, and falling back to printing so the generated document is never lost.

The printed fragment is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped.

### Requirement: Stop when nothing is discovered
When discovery yields no supported configuration for the given input, the wizard SHALL stop with a message saying so, inviting the user to pass their server URL directly or to write the account by hand from the documented sample (linked). It SHALL NOT prompt for a server field it could not discover, and SHALL NOT emit a partial account.

### Requirement: Local backend auto-detected
A typed folder path or `file://` URL SHALL configure a local backend, auto-detecting the kind from on-disk markers: a pimdir index file or blob directory means pimdir, a directory holding at least one collection (a subdirectory carrying an `.ics` file or a vdir metadata marker) means vdir. pimdir SHALL be tested first, since a store also holds subdirectories and testing vdir first would misread every store as a home. The wizard SHALL prompt vdir-against-pimdir only when both backends are compiled in and detection is inconclusive, which an empty directory is.

### Requirement: The wizard does not serve a running command
A command finding no configuration SHALL fail with a message pointing at the wizard, not run it. The wizard prints a document rather than writing one, so it cannot hand a configuration back to a command already underway.
