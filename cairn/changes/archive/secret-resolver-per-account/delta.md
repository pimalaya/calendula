---
cairn: change
change: secret-resolver-per-account
---

# Delta

## ADDED Requirements

### Requirement: A credential command runs once per account
Every secret an account resolves in one run SHALL go through a single resolver, so a command two of its backend blocks name is spawned once and its value handed to both. A `pass` or `gpg` entry shared by `caldav` and `gcal` therefore costs one unlock per account check, not one per backend.

Distinctness is the configured command's own shape, and never crosses the two of them: a shell line and the argv spelling that runs it through the platform shell are two commands.

The resolver holds plaintext for as long as it lives, so it SHALL be built where the backends of one account are assembled and dropped with them, never held for the life of the process and never stored on a client.

A caller opening a single backend SHALL keep resolving through a resolver of its own, so the shape stays the same everywhere and nothing outlives its account.

## MODIFIED Requirements

(none)

## REMOVED Requirements

(none)
