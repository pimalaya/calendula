---
cairn: log
date: 2026-08-29
change: secret-resolver-per-account
---

# A shared credential command was unlocked once per backend

Landed with the move onto pimalaya-config 0.2, which is where the resolver comes from.

## What landed

**[src/account/check.rs](../../src/account/check.rs) builds one `SecretResolver` per account check.** It is passed to the CalDAV and gcal checks and dropped with the check.

`account check` and the wizard's connection test both exercise every backend the account configures. An account whose `caldav` and `gcal` blocks name one `pass` entry asked for the passphrase twice, each `Secret::get` spawning the command afresh; the second is now answered from the first.

**[src/caldav/client.rs](../../src/caldav/client.rs) and [src/gcal/client.rs](../../src/gcal/client.rs) gained a `_with` entry point taking the resolver**, the plain one keeping a resolver of its own for a caller opening a single backend.

**[src/wizard/secret.rs](../../src/wizard/secret.rs) builds a `CommandConfig`** rather than a `std::process::Command`, which is the shape `Secret::Command` now holds and what makes a configured command comparable enough to memoize. The two TOML shapes parse and serialize exactly as before, so no configuration file changes.

## Capabilities moved

- config: how many times a credential command runs
