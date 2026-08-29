---
cairn: change
id: secret-resolver-per-account
status: landed
created: 2026-08-29
---

# A shared credential command was unlocked once per backend

An account may configure several backends at once, and two of them may read the same credential: a `caldav` block and a `gcal` block pointing at one `pass` entry is the ordinary case for someone whose provider is reachable both ways.

Every secret was resolved on its own, `Secret::get` spawning the command at the moment a backend needed its value and caching nothing. Two backends naming one command therefore ran it twice, and for a `pass` or `gpg` entry each run is a key unlock: `account check` on such an account asked for the passphrase twice, and so did the wizard's connection test, which runs the same check on the account it has just generated.

## What changes

pimalaya-config 0.2 carries `secret::SecretResolver`, which spawns each distinct command once and hands its value to every field naming it. `Secret::Command` now holds a `CommandConfig` (a shell line, or a program and its arguments) rather than a built `std::process::Command`, which is what makes a configured command comparable and so memoizable.

`check_account` builds one resolver, passes it to the CalDAV and gcal checks, and drops it with the check. The resolver holds plaintext while it lives, so it is never a global and never stored on a client.

## What does not change

The TOML shapes: a string is still a shell line, an array still a program and its arguments, and a configuration file written against an earlier calendula is read exactly as before.

The prompts, the errors and the wizard's flow are untouched. A command still runs at first use rather than up front, and a backend built on its own (a shared command picking one backend, `caldav`/`gcal` subcommands) resolves through a resolver of its own, which memoizes nothing it will not reuse.
