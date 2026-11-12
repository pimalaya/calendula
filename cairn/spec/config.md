---
cairn: spec
capability: config
status: current
---

# Configuration

Configuration is a TOML file loaded by pimalaya-config, holding a top-level block of options plus one `[accounts.<name>]` block per account, each carrying an optional sub-block per backend.

### Requirement: Loading and merging
Without `-c`, a configuration SHALL be read from every existing canonical path, each a layer deep-merged on top of the ones before: `$XDG_CONFIG_HOME/calendula/config.toml`, `$HOME/.config/calendula/config.toml`, then `$HOME/.calendularc`, a path naming a file already loaded skipped. `-c` SHALL replace them (The configuration path comes from `-c` only).

### Requirement: The configuration path comes from `-c` only
`-c/--config` SHALL take one path per occurrence, never split on a delimiter, and MAY be repeated: the first path is the base and the next ones are deep-merged on top, in order, which is how a public configuration and a private one stay separate files. No environment variable SHALL be read for it: a path kept in one goes through the flag, `-c "$CALENDULA_CONFIG"`.

#### Scenario: A path holding a colon
- GIVEN `-c 'one.toml:two.toml;three.toml'`
- WHEN calendula reads its flags
- THEN it reads one path, spelled as given

### Requirement: A missing configuration meets a person with the welcome
When no configuration exists at the first `-c` path or the default location, and stderr is a terminal with `--json` off, a bare `calendula` and every command loading the configuration SHALL print pimalaya-cli's welcome on stderr and exit 1, printing nothing on stdout. The welcome SHALL frame Calendula in one sentence, name the missing path, give the commands creating it from config.sample.toml (copied from share/doc/calendula next to the binary when one ships, downloaded with `curl` otherwise), invite the reader to edit it, and give the `event list` command carrying the `-c` paths.

Otherwise the command SHALL fail naming the path it looked at and the sample, as the failure object under `--json`. calendula SHALL NOT prompt to create a configuration.

#### Scenario: A mistyped path in a script
- GIVEN no file at `~/.config/calendula/confg.toml`, and stderr a pipe
- WHEN `calendula -c ~/.config/calendula/confg.toml calendar list` runs
- THEN it fails with `No configuration found at <that path>, start from the sample: https://github.com/pimalaya/calendula/blob/master/config.sample.toml`, and prints no welcome

#### Scenario: A first run at a terminal
- GIVEN an empty home and a terminal
- WHEN `calendula` runs
- THEN stderr says `Welcome to Calendula, the CLI to manage calendars.`, names `~/.config/calendula/config.toml`, gives the `curl` command creating it and then `calendula event list`, and the command exits 1

### Requirement: Account resolution failures name what is missing
The account SHALL be resolved through pimalaya-config's `TomlConfig::load` and `take_account`: the account `-a` names, else the one marked `default = true`. Several accounts marked default, layers and `-c` files merged, SHALL be refused, naming them sorted, never one taken at random; an unknown `-a` name SHALL be refused, listing the accounts the configuration holds; no default and no `-a` SHALL be refused, naming both ways of picking one. An account named with `-a` SHALL resolve whatever the defaults.

#### Scenario: Two defaults
- GIVEN a configuration marking `work` and `perso` with `default = true`
- WHEN `calendula event list` runs
- THEN it fails with `Several accounts are marked default (perso, work)`, and `calendula -a work event list` runs on `work`

### Requirement: The configuration is never written
calendula SHALL NOT write a configuration file: the user writes it, from the documented sample.

### Requirement: The global block folds under the account
The top-level options SHALL be folded under the selected account, the account's own values overriding them field by field. A value set once at the top therefore applies everywhere it is not overridden.

### Requirement: Unknown fields
`deny_unknown_fields` SHALL be set on the leaf blocks, so a typo in an option is reported rather than ignored, and SHALL NOT be set on the top-level and account blocks, whose backend blocks exist only in the builds carrying their feature: a build without a backend still loads a file naming it. A table calendula no longer reads SHALL be refused there by name instead: `todo`, `journal`, `table` and the top-level `account`.

### Requirement: A leftover todo or journal table is refused by name
A `todo` or `journal` table left in a configuration, at the top level or under an account, SHALL fail the load, naming the CLI that took the domain over (unfog, the Pimalaya CLI for tasks, for `todo`; journal, the Pimalaya CLI for journal entries, for `journal`) and saying to remove the table. The top-level and account blocks still deny no unknown key: these two tables are refused by name, as the family refuses its removed keys, rather than read or ignored.

#### Scenario: An upgraded configuration
- GIVEN a configuration written for 0.2 with a `[todo.list]` table
- WHEN any calendula command loads it
- THEN the command fails before reaching a backend, the error locating the table (its line, or its key once several files are merged) and naming unfog

#### Scenario: A table under an account
- GIVEN a configuration with an `[accounts.work.journal]` table
- WHEN any calendula command loads it
- THEN the command fails, the error naming journal

### Requirement: No colour or table is configured
The configuration SHALL hold no colour and no table shape. A `table` table (`table.preset`, `table.arrangement`), at the top level or under an account, and a top-level `account` table (`account.list.table`) SHALL fail the load by name, saying the output is plain text and to remove it. A `*-color` key under `event.list.table` or `item.list.table`, or a `calendar.list` block, SHALL fail the load as an unknown field of the block that holds it. `event.list.page-size` and `item.list.page-size` stay.

#### Scenario: A colour key
- GIVEN a configuration written for 0.2 with `event.list.table.id-color = "red"`
- WHEN any calendula command loads it
- THEN it fails naming the line and the unknown field `table`, where `page-size` was expected

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS, ALPN and authentication, `gcal` and `msgraph` with TLS, ALPN and authentication.

An account MAY carry several; which one a unified command uses is the backend capability's business.

Every paged listing family SHALL additionally carry a block naming its default page size: `event.list` and `item.list`.

### Requirement: Credentials nest under `auth`
Every backend that authenticates SHALL carry its credentials under an `auth` sub-block, so the same concept is spelled the same way across the Pimalaya CLIs: `caldav.auth.basic.password`, `gcal.auth.token`, as cardamum spells `people.auth.token` and `msgraph.auth.token`.

A backend accepting exactly one kind SHALL still nest, as a struct rather than an enumeration of kinds, so gaining a second kind later is not a breaking change to the first.

### Requirement: gcal has nothing to discover
The Calendar API lives at one fixed base URL, so the `gcal` block SHALL name no server and offer no discovery route: a bearer token and a TLS profile are the whole block.

### Requirement: Paths are shell-expanded
Every configured filesystem path SHALL be shell-expanded by the deserializer that reads it, so `~` and environment variables both work and no call site can forget. A path used raw would resolve against the working directory, which for a store root silently creates an empty one.

`vdir.home-dir`, `pimdir.root` and `tls.cert` are the paths concerned; the optional one goes through a local deserializer until pimalaya-config ships an optional variant of its own.

### Requirement: CalDAV locates its home-set three ways
The `caldav` block SHALL offer exactly three mutually exclusive routes, from most to least discovery. `discover` resolves a bare domain through RFC 6764 SRV records and the `.well-known` path.

`server` names the context root the principal and home-set walk starts from, taking a full URL, a bare domain or `domain:port` with `https` assumed for a bare authority, and is parsed when it is used rather than when it is read. `home` pins the home-set outright.

A block carrying none of the three SHALL be rejected by name.

### Requirement: Secrets are read, never written
A CalDAV password, a CalDAV token or a gcal token SHALL be a secret read from the configuration itself or from the standard output of a command.

calendula SHALL NOT write a secret anywhere: an OAuth 2.0 token broker is a command like any other, and a missing value surfaces when the account is tested. Google expires an access token within the hour, so a token broker is the practical answer there rather than a stored value.

### Requirement: A credential command runs once per account
Every secret an account resolves in one run SHALL go through a single resolver, so a command two of its backend blocks name is spawned once and its value handed to both. A `pass` or `gpg` entry shared by `caldav` and `gcal` therefore costs one unlock per account check, not one per backend.

Distinctness is the configured command's own shape, and never crosses the two of them: a shell line and the argv spelling that runs it through the platform shell are two commands.

The resolver holds plaintext for as long as it lives, so it SHALL be built where the backends of one account are assembled, the unified client of the account or `account check`, and dropped with them, never held for the life of the process. The unified client SHALL keep it across the reconnection that follows a composer, so a reopened backend runs no secret command again.

A caller opening a single backend SHALL keep resolving through a resolver of its own, so the shape stays the same everywhere and nothing outlives its account.

### Requirement: A backend names the ALPN it offers
A backend whose protocol pins an HTTP version SHALL expose an `alpn` key, a list of identifiers offered during the TLS handshake. Unset offers the protocol default, an explicit empty list skips ALPN negotiation and a non-empty list replaces the default.

`caldav.alpn`, `gcal.alpn` and `msgraph.alpn` default to `http/1.1`, the only version io-http speaks.

The runtime TLS profile SHALL be built through `TlsConfig::into_tls(alpn)`, never through a conversion assuming a list, so no caller can negotiate a protocol it did not name.

### Requirement: The composer is a command, not a library
`item.composer` SHALL hold the command an item is edited through, at the top level and per account, as a shell line or an argv list. calendula SHALL spawn it rather than embed an editor: what edits an iCalendar is the user's choice, and a terminal editor, a graphical one and a purpose-built one are all one configuration line apart.

The key SHALL sit under `item` rather than under `event`: `event` and `item` are views over the same resources, and what is edited is an iCalendar object, which is what `item` names.
