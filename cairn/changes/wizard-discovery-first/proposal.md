---
cairn: change
id: wizard-discovery-first
status: landed
created: 2026-08-08
---

# Wizard: discovery first, print rather than write

## Why

The old wizard asked four questions before it could do anything: confirm, name the account, give an email, pick a backend. Only then did it discover, and only over SRV, unbounded, before falling through to hand-entered host, port and encryption. It wrote straight to disk, tested nothing, and ran implicitly whenever a command found no configuration, which meant a half-configured account could be written mid-command.

himalaya's wizard had already solved this shape: one prompt whose input orients everything, bounded parallel discovery, a connection tested before anything is emitted, and a document printed rather than written.

## What

The same shape, adapted to calendars. One prompt takes an address, a server URL or a folder path. An address runs bounded parallel discovery and each distinct context root becomes one entry, its authentication scheme picked in a second prompt from what it advertised. A folder is detected as a vdir home or a pimdir store from its own markers. The account name is derived, never asked. The account is tested with the same check `account check` runs, then printed on stdout, and only offered for saving when stdout is a terminal.

One deviation from himalaya is deliberate and specified: a typed server URL is configured as given. himalaya refuses hand entry because mail providers are near-universally discoverable. CalDAV servers are not, and a wizard that could not configure a Radicale would be a wizard most self-hosters cannot use.

`account configure` goes: it wrote to disk, which a printing wizard cannot, and himalaya has no equivalent.
