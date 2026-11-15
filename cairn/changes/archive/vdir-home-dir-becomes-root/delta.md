---
cairn: delta
change: vdir-home-dir-becomes-root
---

# Delta

Every requirement is of the config capability.

## ADDED Requirements

### Requirement: The vdir home is `vdir.root`
An account's `vdir` block SHALL name the vdir home, the directory holding one calendar per immediate subdirectory, in `root`, its `~` and environment variables expanded as it is read ("Paths are shell-expanded"), as `pimdir.root` names a store. `vdir.home-dir`, the key 0.2 read, SHALL be refused by name in the family's sentence (pimalaya-config's `removed_key!`), alone or beside `root`: ``The `vdir.home-dir` key is removed, the vdir home being named `root` as a store root is: rename it `vdir.root`, the path unchanged, see MIGRATION.md``. A build without `vdir` loads the block unread.

#### Scenario: A 0.2 vdir home
- GIVEN an account holding `vdir.home-dir = "~/calendars"`
- WHEN `calendula --json calendar list` runs
- THEN it exits 1, the sentence naming the `vdir.home-dir` key as removed, `vdir.root` and MIGRATION.md in `sources`

#### Scenario: The same home under `root`
- GIVEN an account holding `vdir.root = "~/calendars"`
- WHEN the configuration is loaded
- THEN the vdir home is the calendars directory of the user's home

## MODIFIED Requirements

### Requirement: Unknown fields
Every block of the configuration, the top-level and account blocks included, SHALL set `deny_unknown_fields`, so a key calendula does not know fails the load naming it, a typo reported rather than ignored. The block of a backend the build leaves out SHALL load unread, so one file serves every build, and a command reaching it SHALL fail with `backend-not-built` (see "Backend selection order" in backends). The account `proxy` of a build with no network backend SHALL load unread too. A table calendula no longer reads SHALL be refused by name: `todo`, `journal`, `table`, the top-level `account`, `calendar.list`, `event.list.table` and `item.list.table`, and so SHALL the `caldav.discover` and `vdir.home-dir` keys, each through pimalaya-config's `removed_key!`, in the family sentence ``The `<name>` table is removed, <why>: delete it, see MIGRATION.md`` (`key` for a key, and its own instead for `caldav.discover` and `vdir.home-dir`).

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `root`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS, ALPN and authentication, `gcal` and `msgraph` with TLS, ALPN and authentication. The rest of the requirement as it stands.

### Requirement: Paths are shell-expanded
`vdir.root`, `pimdir.root` and `tls.cert` are the paths concerned; the optional one goes through a local deserializer until pimalaya-config ships an optional variant of its own. The rest of the requirement as it stands.

## REMOVED Requirements
