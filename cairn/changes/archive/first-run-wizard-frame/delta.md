---
cairn: change
change: first-run-wizard-frame
---

# Delta

## ADDED Requirements

### Requirement: A named command runs the wizard
A `configure` command (alias `wizard`) SHALL run the wizard by name, without the welcome, since whoever typed it knows what it does. It refuses to run when stdin is not a terminal, naming the sample configuration to write by hand instead.

### Requirement: The welcome names the missing path
The welcome SHALL name the configuration path that was looked for, which is the one `-c` or `CALENDULA_CONFIG` gave or the default location, so a mistyped path shows up as itself rather than as a generic first run.

It frames the product, points at the documented sample, and names the command that runs the wizard again later.

### Requirement: Generating never rewrites what a human wrote
The wizard SHALL write a configuration file that does not exist and append a plain text block to one that does, never parsing and re-serializing the document, so comments, ordering and formatting survive.

The account name is suffixed until the configuration does not already hold it, since a second `[accounts.<name>]` table makes the whole document fail to parse, and the generated account claims `default` only when no other account does.

The target path is not prompted: it is where `-c` or `CALENDULA_CONFIG` pointed, or the default location.

### Requirement: Account resolution failures name what is missing
Each of the three ways account resolution fails SHALL name what is missing and what to do about it: a missing configuration names the path it looked for, a missing named account lists the accounts the configuration does hold, and a missing default names both ways of picking one.

### Requirement: A generated account reads in a deliberate order
The serializer SHALL decide what a generated account holds, so a defaulted field is omitted and no field is enumerated twice.

The rendering SHALL nonetheless order what it emits: the groups run most-defining first, an unrecognised group renders after them rather than being dropped, a group's `server` key reads before the credentials qualifying it, and a blank line separates groups.

## MODIFIED Requirements

### Requirement: The wizard serves a running command through a re-read
A command finding no configuration SHALL offer the wizard rather than only pointing at it, and SHALL carry on afterwards either way. The offer never ends the process: accepting gives the command a chance to work, declining leaves it to fail on the configuration it still has not got.

Because the wizard may print a document instead of writing one, the configuration SHALL be read again after the offer rather than assumed, so a command whose wizard wrote nothing fails the ordinary way.

A bare invocation has nothing to carry on to, so a declined offer falls back to the help, which is also what someone already configured gets. Nothing is offered when stdin is not a terminal or `--json` is set.

This supersedes the wizard not serving a running command, whose premise was a wizard that only ever printed.

### Requirement: Saved where the configuration lives, printed when redirected
The generated configuration SHALL be printed as a TOML document on stdout in JSON mode and whenever stdout is redirected, so `calendula configure > config.toml` and any script keep working.

Only when writing to a terminal SHALL the wizard offer to save it, to the configuration path rather than to a prompted one, creating the parent directory, appending to a file that already exists rather than clobbering it, and falling back to printing so the generated document is never lost.

The printed fragment is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped.

## REMOVED Requirements
