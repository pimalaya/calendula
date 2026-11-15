---
cairn: delta
change: default-is-a-name
---

# Delta

## ADDED Requirements

## MODIFIED Requirements

### Requirement: Account resolution failures name what is missing
(config) The account SHALL be resolved through pimalaya-config's `TomlConfig::load` and `take_account`: the account `-a` names, `default` being a name like any other, else, `-a` absent or empty, the one marked `default = true`. Several accounts marked default, layers and `-c` files merged, SHALL be refused, naming them sorted, never one taken at random; an unknown `-a` name SHALL be refused, listing the accounts the configuration holds, and for the name `default` adding to omit `-a` to take the account marked default; no default and no `-a` SHALL be refused, naming both ways of picking one. An account named with `-a` SHALL resolve whatever the defaults. Under `--json`, the three refusals SHALL be coded `account-missing` (an unknown name), `account-default-missing` (no default) and `account-default-several` (several defaults).

#### Scenario: `-a default` names an account
- GIVEN a configuration holding `work`, marked `default = true`, and no account named `default`
- WHEN `calendula --json -a default event list` runs
- THEN it fails coded `account-missing`, the message listing `work` and saying to omit `-a` to take the account marked default
- AND once the configuration holds an account named `default`, the same command runs on it, whichever account is marked default

### Requirement: A JSON error carries a stable code
(commands) The code is also read from pimalaya-config's `AccountError`. The code table gains `account-missing`, `account-default-missing` and `account-default-several`.

## REMOVED Requirements
