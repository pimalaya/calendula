---
cairn: change
id: wizard-runs-when-unconfigured
status: landed
created: 2026-08-28
---

# The wizard runs when asked, or when nothing is configured

Bare `calendula` ran the wizard unconditionally, reading no configuration first. A user with four accounts in `~/.calendularc` typing `calendula` was dropped into a first-run flow, and there was no other way to reach the wizard, since it had no command of its own.

himalaya and cardamum settled this differently and settled it together: the wizard is a `configure` command, and it is *offered* from the two places nothing can happen without a configuration, a bare invocation finding none and a command needing an account finding none. calendula is the odd one out, and the divergence is user-visible on the very first thing a user types.

## What changes

- `calendula configure` (alias `wizard`) runs the wizard, which is the way to reach it whether or not a configuration exists.
- Bare `calendula` offers to configure only when no configuration is found; otherwise it prints the help. `--account` also gets the help, naming an account to act on being a half-typed command rather than a first run.
- A command finding no configuration raises the same offer, then re-reads and fails the ordinary way, since the wizard may print the account instead of writing it.
- The offer is skipped in JSON mode and when stdin is not a terminal: neither a script nor a JSON consumer can answer a prompt.
- `discover::run` returns the account it built and nothing else. Where it lands moves to `configure`, which saves it to a target that does not exist, appends it to one that does, or prints it.
- An account joining an existing configuration is appended as plain text, so comments and hand formatting survive; its name is suffixed until free; it claims the default only when no account already does.
- `AccountConfig::render` renders one `[accounts.<name>]` block in reading order: the groups in a fixed order, the endpoint first inside each.
- `CONFIG_SAMPLE_URL` moves to `config`, being about the configuration rather than about the wizard, and a build with no wizard-capable backend says so where the offer would have run.

## What does not change

Everything the wizard does once it runs: one prompt orienting the flow, the typed-URL deviation, the bounded parallel discovery, one entry per service then auth, OAuth folded into the API token, the connection tested before anything is emitted, the local backend auto-detected, and gcal left out.
