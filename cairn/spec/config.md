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
The account SHALL be resolved through pimalaya-config's `TomlConfig::load` and `take_account`: the account `-a` names, `default` being a name like any other, else, `-a` absent or empty, the one marked `default = true`. Several accounts marked default, layers and `-c` files merged, SHALL be refused, naming them sorted, never one taken at random; an unknown `-a` name SHALL be refused, listing the accounts the configuration holds, and for the name `default` adding to omit `-a` to take the account marked default; no default and no `-a` SHALL be refused, naming both ways of picking one. An account named with `-a` SHALL resolve whatever the defaults. Under `--json`, the three refusals SHALL be coded `account-missing` (an unknown name), `account-default-missing` (no default) and `account-default-several` (several defaults).

#### Scenario: Two defaults
- GIVEN a configuration marking `work` and `perso` with `default = true`
- WHEN `calendula event list` runs
- THEN it fails with `Several accounts are marked default (perso, work)`, and `calendula -a work event list` runs on `work`

#### Scenario: `-a default` names an account
- GIVEN a configuration holding `work`, marked `default = true`, and no account named `default`
- WHEN `calendula --json -a default event list` runs
- THEN it fails coded `account-missing`, the message listing `work` and saying to omit `-a` to take the account marked default
- AND once the configuration holds an account named `default`, the same command runs on it, whichever account is marked default

### Requirement: The configuration is never written
calendula SHALL NOT write a configuration file: the user writes it, from the documented sample.

### Requirement: The global block folds under the account
The top-level options SHALL be folded under the selected account, the account's own values overriding them field by field. A value set once at the top therefore applies everywhere it is not overridden.

### Requirement: Unknown fields
Every block of the configuration, the top-level and account blocks included, SHALL set `deny_unknown_fields`, so a key calendula does not know fails the load naming it, a typo reported rather than ignored. The block of a backend the build leaves out SHALL load unread, so one file serves every build, and a command reaching it SHALL fail with `backend-not-built` (see "Backend selection order" in backends). The account `proxy` of a build with no network backend SHALL load unread too. A table or a key calendula no longer reads (`todo`, `journal`, `table`, the top-level `account`, `calendar.list`, `event.list.table`, `item.list.table`, `caldav.discover`, `vdir.home-dir`) SHALL fail the same way, with no sentence of its own.

#### Scenario: A typo in an account
- GIVEN `[accounts.a]` holding `defualt = true`
- WHEN the configuration is loaded
- THEN it fails naming the unknown field `defualt`

#### Scenario: A backend left out of the build
- GIVEN an account holding a `gcal` block, and a build without the `gcal` feature
- WHEN the configuration is loaded
- THEN it loads, the `gcal` block unread, and a unified command on that account fails coded `backend-not-built`, naming `--features gcal`

### Requirement: No colour or table is configured
The configuration SHALL hold no colour and no table shape. A `table` table (`table.preset`, `table.arrangement`), at the top level or under an account, a top-level `account` table (`account.list.table`), and the `calendar.list`, `event.list.table` and `item.list.table` tables holding the `*-color` keys, at the top level or under an account, SHALL fail the load as unknown fields. `event.list.page-size` and `item.list.page-size` stay.

#### Scenario: A colour key
- GIVEN a configuration written for 0.2 with `event.list.table.id-color = "red"`
- WHEN any calendula command loads it
- THEN it fails naming the unknown field `table`

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `root`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS, ALPN and authentication, `jmap` with its server, TLS, ALPN and authentication, `gcal` and `msgraph` with TLS, ALPN and authentication.

An account MAY carry several; which one a unified command uses is the backend capability's business.

An account MAY carry a `proxy`, and `caldav`, `jmap`, `gcal` and `msgraph` each one of their own ("A proxy is set per account and per backend").

Every paged listing family SHALL additionally carry a block naming its default page size: `event.list` and `item.list`.

### Requirement: TLS is set per account and per backend
Every network backend block SHALL take a `tls` table spelled as pimalaya-config's toml.md writes it (*Every binary spells its TLS table alike*) and as Himalaya's: `provider` (`rustls` or `native-tls`), `rustls.crypto` (`aws` or `ring`), `cert` (a PEM certificate to trust) and `client` (`cert` and `key`, a PEM client certificate and its PEM private key, PKCS#8 under native-tls, both required once `client` is given), each optional, its paths shell-expanded. An account MAY carry a `tls` table of the same keys, which each network backend's `tls` SHALL take from, field by field, for what it leaves unset, `client` as one so a certificate is never paired with another table's key. A key the table does not read SHALL be refused by serde's `deny_unknown_fields` alone.

#### Scenario: A backend table over the account's
- GIVEN an account holding `tls.provider = "native-tls"`, `tls.cert = "/ca.pem"` and `tls.client`, and its `caldav.tls.cert = "/dav.pem"`
- WHEN the configuration is loaded
- THEN that backend runs native-tls trusting `/dav.pem` and presenting the account's client certificate, and every other network backend of the account native-tls trusting `/ca.pem`

### Requirement: A proxy is set per account and per backend
An account's `proxy` SHALL apply to every network backend of that account (CalDAV, JMAP, Google Calendar, Microsoft Graph) whose own block names no `proxy`, and a backend's own `proxy` SHALL win over it. With neither, the connection SHALL read the `all_proxy` and `https_proxy` environment variables, honouring `no_proxy`.

`proxy.url` SHALL take `socks5://`, `socks5h://` or `http://`. `proxy.username` and `proxy.password` SHALL override the URL's user info, the password being a secret like any credential; a password without a username SHALL be rejected.

### Requirement: Credentials nest under `auth`
Every backend that authenticates SHALL carry its credentials under an `auth` sub-block, so the same concept is spelled the same way across the Pimalaya CLIs: `caldav.auth.basic.password`, `gcal.auth.token`, as cardamum spells `people.auth.token` and `msgraph.auth.token`.

A backend accepting exactly one kind SHALL still nest, as a struct rather than an enumeration of kinds, so gaining a second kind later is not a breaking change to the first.

### Requirement: A CalDAV server may ask for no credentials
`caldav.auth` SHALL carry a credential-less variant alongside Basic and Bearer, spelled `auth = "none"`, which emits no `Authorization` header, and SHALL be optional, a block naming none reading as `auth = "none"`: a server asking for no credential (Radicale with `auth = none`, a server behind a VPN or an authenticating reverse proxy) needs no key for it, as cardamum and carillon read their DAV blocks.

A server answering a request 401 to a block sending no credential SHALL fail with a message saying so, "CalDAV server answered 401: it asks for credentials, and the account configures no `auth`", above io-webdav's `WebDAV server returned HTTP 401`, on every command reaching CalDAV (the unified ones, the `caldav` namespace and `account check`), its code unchanged (`null`). A block naming a credential the server refuses fails as before, without the sentence.

#### Scenario: A block with no auth
- GIVEN an account whose `caldav` block names `server` and no `auth`
- WHEN the configuration is loaded
- THEN it loads, and every request goes without an `Authorization` header

#### Scenario: A server asking for credentials a block does not give
- GIVEN an account whose `caldav` block names no `auth`, its server answering 401
- WHEN `calendula --json account check` runs
- THEN the `caldav` row is `ok: false`, `code` `null`, its `error` saying the server asks for credentials and the account configures no `auth`

### Requirement: The jmap block
An account MAY carry a `jmap` block with `server`, `tls`, `alpn`, `proxy` and `auth`, shaped as cardamum's and himalaya's, so one JMAP account reads the same in all three. `server` SHALL take a full URL, fetched as the session endpoint, or a bare `host[:port]`, taken over `https` and discovered through `GET /.well-known/jmap`; its schemes SHALL be `http`, `https`, `jmap` and `jmaps`, the last two mapping to the first two, any other refused, and its host ASCII, an internationalised one written in Punycode. `auth` SHALL be one of `header` (a whole `Authorization` value), `bearer.token` and `basic` (`username`, `password`), each secret read as any credential. No DNS discovery runs: a server name is used as written.

### Requirement: gcal has nothing to discover
The Calendar API lives at one fixed base URL, so the `gcal` block SHALL name no server and offer no discovery route: a bearer token and a TLS profile are the whole block.

### Requirement: Paths are shell-expanded
Every configured filesystem path SHALL be shell-expanded by the deserializer that reads it, so `~` and environment variables both work and no call site can forget. A path used raw would resolve against the working directory, which for a store root silently creates an empty one.

`vdir.root`, `pimdir.root`, `tls.cert`, `tls.client.cert` and `tls.client.key` are the paths concerned; the optional one goes through a local deserializer until pimalaya-config ships an optional variant of its own.

### Requirement: The vdir home is `vdir.root`
An account's `vdir` block SHALL name the vdir home, the directory holding one calendar per immediate subdirectory, in `root`, its `~` and environment variables expanded as it is read ("Paths are shell-expanded"), as `pimdir.root` names a store. `vdir.home-dir`, the key 0.2 read, SHALL fail as an unknown field. A build without `vdir` loads the block unread.

#### Scenario: A 0.2 vdir home
- GIVEN an account holding `vdir.home-dir = "~/calendars"`
- WHEN `calendula --json calendar list` runs
- THEN it exits 1 naming the unknown field `home-dir`

#### Scenario: The same home under `root`
- GIVEN an account holding `vdir.root = "~/calendars"`
- WHEN the configuration is loaded
- THEN the vdir home is the calendars directory of the user's home

### Requirement: CalDAV locates its home-set two ways
The `caldav` block SHALL offer exactly two mutually exclusive routes, `server` and `home`, and calendula SHALL resolve no domain through DNS SRV records.

`server` names the context root the principal and home-set walk starts from, taking a full URL, a bare domain or `domain:port`, and is parsed when it is used rather than when it is read. A value naming no scheme (no `://`) SHALL be read as `https://` followed by it; a value naming one SHALL be read as written and refused when it does not parse or its scheme is not `http` nor `https`, never prefixed. A `server` naming an `https` origin (no path) SHALL be probed at `.well-known/caldav` (RFC 6764 §5), as cardamum probes `.well-known/carddav`, and the walk SHALL start from the redirect target when the server answers one over `https`; a redirect to another scheme SHALL be ignored with a warning, the credential going where the target points, and a failed probe SHALL leave the origin as it is. `home` pins the home-set outright.

A block carrying neither SHALL be rejected by name, and one carrying both SHALL be refused before anything connects, `account check` included.

A `caldav.discover` key, the RFC 6764 route an earlier calendula offered, SHALL fail the load as an unknown field.

#### Scenario: A discover route left in a configuration
- GIVEN a configuration written for 0.2 with `caldav.discover = "fastmail.com"`
- WHEN any calendula command loads it
- THEN it fails before reaching a backend, naming the unknown field `discover`

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

`caldav.alpn`, `jmap.alpn`, `gcal.alpn` and `msgraph.alpn` default to `http/1.1`, the only version io-http speaks.

The runtime TLS profile SHALL be built through `TlsConfig::into_tls(alpn)`, never through a conversion assuming a list, so no caller can negotiate a protocol it did not name.

### Requirement: The composer is a command, not a library
`item.composer` SHALL hold the command an item is edited through, at the top level and per account, as a shell line or an argv list. calendula SHALL spawn it rather than embed an editor: what edits an iCalendar is the user's choice, and a terminal editor, a graphical one and a purpose-built one are all one configuration line apart.

The key SHALL sit under `item` rather than under `event`: `event` and `item` are views over the same resources, and what is edited is an iCalendar object, which is what `item` names.

### Requirement: One sample configuration, with a section per backend
The repository SHALL ship config.sample.toml holding one `[accounts.example]` table with dotted keys and no other table header. It SHALL list the options every account shares and each of their values, the default active and the others commented as `#key = value`, every line ending in one tab-aligned comment, and every password command a failing `exit 127` placeholder. It SHALL end with one section per way in (CalDAV, JMAP, Google Calendar, Google CalDAV, Microsoft Graph, Fastmail, vdir, pimdir), each a paragraph of commented code headed by its name, its own alternatives commented twice, an OAuth 2.0 token read through `ortie token show -a example`, so the file holds a working account once one section is uncommented. The test suite SHALL parse config.sample.toml through the loader `-c` uses with each section of a compiled backend uncommented whole, then on top of it each line commented twice inside it and each line of commented code outside the sections, in place of the active lines of its field, and SHALL resolve `example` as the default account declaring a backend each time, so a configuration change breaking one fails the build.

#### Scenario: A field is renamed
- GIVEN config.sample.toml, whose CalDAV section sets `caldav.tls.cert`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample and the line uncommented

#### Scenario: A build without a backend
- GIVEN a build without the `gcal` feature
- WHEN the test suite runs
- THEN it parses every section and its variants, the Google Calendar block loading unread
