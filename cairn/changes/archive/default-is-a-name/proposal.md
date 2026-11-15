---
cairn: change
id: default-is-a-name
status: landed
created: 2026-11-15
---

# Default is a name

> Cross-repo change, same id in pimalaya-config (d5a33cb). Calendula follows it (item 24 of the 2026-11-15 owner round).

## Why

pimalaya-config 0.4 read `-a default` as the account marked default, so an account named `default` was unreachable whenever another one was marked. pimalaya-config master d5a33cb reads `default` as a name like any other, an absent or empty `-a` taking the marked account, and its `AccountError::Missing` for the name `default` adds ", omit `-a` to take the account marked default". The three account refusals carried no code, so a `--json` caller had to match their wording.

## What

- Pin pimalaya-config to d5a33cb in `[patch.crates-io]`.
- Three codes in `ErrorCode`: `account-missing` (`AccountError::Missing`), `account-default-missing` (`NoDefault`), `account-default-several` (`SeveralDefaults`), read off the chain by `error::code` (a new `account_code` reader beside the protocol refusal readers), so the failure object carries them through the same `with_code` call every command already goes through.
- Spec: config ("Account resolution failures name what is missing"), commands ("A JSON error carries a stable code", its code table).
