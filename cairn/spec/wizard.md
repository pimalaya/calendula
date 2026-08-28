---
cairn: spec
capability: wizard
status: current
---

# Wizard

`calendula configure` (alias `wizard`) runs the interactive wizard. It discovers one account, tests it, then saves it, appends it to the configuration already there, or prints it. Prompts render on stderr, so redirecting stdout into a configuration file works directly.

### Requirement: The wizard runs when asked, or when nothing is configured
The wizard SHALL run from `calendula configure`, and from an offer raised only where nothing can happen without a configuration: a bare `calendula` finding none, and a command needing an account finding none. A bare `calendula` finding one SHALL print the help instead, and so SHALL one carrying `--account`, which names an account to act on and reads as a half-typed command rather than a first run. A file that exists but fails to parse counts as a configuration, so the offer never proposes to write over a broken one.

The offer SHALL be skipped in JSON mode and whenever stdin is not a terminal, neither a script nor a JSON consumer being able to answer a prompt: both get the help or the ordinary failure. A command whose offer was declined or skipped SHALL fail naming the path it looked at and `calendula configure`.

The offer is a hook rather than a gate: the wizard may print the account instead of writing it, so having run it proves nothing, and the caller SHALL look the configuration up again before carrying on.

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
The wizard SHALL NOT prompt for an account name. It derives one from the input (the domain's first label, or the folder name) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key. A name the configuration already holds SHALL be suffixed until it is free, two `[accounts.<name>]` tables making the whole document fail to parse and taking the working accounts down with it.

The generated account SHALL claim the default only when no account already does, since two `default = true` would make the account every command picks depend on map ordering.

### Requirement: Connection tested before printing
The account SHALL be tested before the fragment is printed, so a bad credential or endpoint stops the wizard instead of yielding a configuration that cannot connect. The test is the same one `account check` runs. Its failure SHALL name each backend that failed and why.

### Requirement: Saved, appended, or printed
The generated account SHALL be printed as a TOML document on stdout in JSON mode and whenever stdout is redirected, so `calendula configure > config.toml` and any script keep working. `configure` SHALL refuse outright when stdin is not a terminal, there being no way to answer its prompts.

On a terminal the target is `--config` when one was given, else the platform configuration path. A target that does not exist SHALL be offered as a file to create; one that does SHALL be offered as a block to append, so the accounts already there survive. Appending SHALL be a plain text append rather than a re-serialization, so comments, ordering and hand-written formatting come out untouched. A declined offer SHALL fall back to printing, so the generated document is never lost.

What was written SHALL be reported: the path, the account name, and how to reach it when it did not claim the default, the name never having been asked for.

The rendered account is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped. Its groups read in a fixed order, the endpoint first within each, since alphabetical order would file a server under the credential authenticating against it.

### Requirement: Stop when nothing is discovered
When discovery yields no supported configuration for the given input, the wizard SHALL stop with a message saying so, inviting the user to pass their server URL directly or to write the account by hand from the documented sample (linked). It SHALL NOT prompt for a server field it could not discover, and SHALL NOT emit a partial account.

### Requirement: Local backend auto-detected
A typed folder path or `file://` URL SHALL configure a local backend, auto-detecting the kind from on-disk markers: a pimdir index file or blob directory means pimdir, a directory holding at least one collection (a subdirectory carrying an `.ics` file or a vdir metadata marker) means vdir. pimdir SHALL be tested first, since a store also holds subdirectories and testing vdir first would misread every store as a home. The wizard SHALL prompt vdir-against-pimdir only when both backends are compiled in and detection is inconclusive, which an empty directory is.

### Requirement: The wizard covers the discoverable backends only
The wizard SHALL configure CalDAV, vdir and pimdir. gcal is out: it needs no discovery and its token broker story is the user's to settle, so a Google account is written by hand from the sample configuration.

### Requirement: The welcome introduces the wizard only where it was not asked for
The offer SHALL be preceded by a welcome naming what calendula is, the configuration path that is missing, the sample documenting every field, and `calendula configure` for later. `configure` typed by name SHALL skip it, whoever typed it knowing what it does. It renders on stderr, so a redirected stdout holds the document alone.
