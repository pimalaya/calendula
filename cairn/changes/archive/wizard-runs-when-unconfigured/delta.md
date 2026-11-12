---
cairn: change
change: wizard-runs-when-unconfigured
---

# Delta

## ADDED Requirements

### Requirement: The wizard runs when asked, or when nothing is configured
The wizard SHALL run from `calendula configure`, and from an offer raised only where nothing can happen without a configuration: a bare `calendula` finding none, and a command needing an account finding none.

A bare `calendula` finding one SHALL print the help instead, and so SHALL one carrying `--account`, which names an account to act on and reads as a half-typed command rather than a first run.

A file that exists but fails to parse counts as a configuration, so the offer never proposes to write over a broken one.

The offer SHALL be skipped in JSON mode and whenever stdin is not a terminal, neither a script nor a JSON consumer being able to answer a prompt: both get the help or the ordinary failure.

A command whose offer was declined or skipped SHALL fail naming the path it looked at and `calendula configure`.

The offer is a hook rather than a gate: the wizard may print the account instead of writing it, so having run it proves nothing, and the caller SHALL look the configuration up again before carrying on.

### Requirement: The welcome introduces the wizard only where it was not asked for
The offer SHALL be preceded by a welcome naming what calendula is, the configuration path that is missing, the sample documenting every field, and `calendula configure` for later.

`configure` typed by name SHALL skip it, whoever typed it knowing what it does. It renders on stderr, so a redirected stdout holds the document alone.

## MODIFIED Requirements

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the input (the domain's first label, or the folder name) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key.

A name the configuration already holds SHALL be suffixed until it is free, two `[accounts.<name>]` tables making the whole document fail to parse and taking the working accounts down with it.

The generated account SHALL claim the default only when no account already does, since two `default = true` would make the account every command picks depend on map ordering.

### Requirement: Saved, appended, or printed
The generated account SHALL be printed as a TOML document on stdout in JSON mode and whenever stdout is redirected, so `calendula configure > config.toml` and any script keep working.

`configure` SHALL refuse outright when stdin is not a terminal, there being no way to answer its prompts.

On a terminal the target is `--config` when one was given, else the platform configuration path. A target that does not exist SHALL be offered as a file to create; one that does SHALL be offered as a block to append, so the accounts already there survive.

Appending SHALL be a plain text append rather than a re-serialization, so comments, ordering and hand-written formatting come out untouched. A declined offer SHALL fall back to printing, so the generated document is never lost.

What was written SHALL be reported: the path, the account name, and how to reach it when it did not claim the default, the name never having been asked for.

The rendered account is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped.

Its groups read in a fixed order, the endpoint first within each, since alphabetical order would file a server under the credential authenticating against it.

### Requirement: The wizard covers the discoverable backends only
The wizard SHALL configure CalDAV, vdir and pimdir. gcal is out: it needs no discovery and its token broker story is the user's to settle, so a Google account is written by hand from the sample configuration.

## REMOVED Requirements

### Requirement: The wizard does not serve a running command
Removed: a command finding no configuration now raises the offer instead of failing at it. The wizard writes the account when the user accepts, so it can serve the command that raised it; the lookup is repeated afterwards because it may also have printed it instead.

### Requirement: Printed, and saved only on a terminal
Removed: replaced by the save-append-or-print requirement above. The save prompt no longer asks for a path, the target being `--config` or the platform path, so the seeding rule goes with it.
