---
cairn: spec
capability: config
status: current
---

# Configuration

Configuration is a TOML file loaded by pimalaya-config, holding a top-level block of rendering options plus one `[accounts.<name>]` block per account, each carrying an optional sub-block per backend.

### Requirement: Loading and merging
A configuration SHALL be read from the first existing path among the canonical platform locations, or from `-c` / `CALENDULA_CONFIG` when given.

Several paths MAY be passed at once, separated by `:`: the first is the base and the rest are deep-merged on top, which is how a public configuration and a private one stay separate files.

### Requirement: The global block folds under the account
The top-level rendering options SHALL be folded under the selected account, the account's own values overriding them field by field. A value set once at the top therefore applies everywhere it is not overridden.

### Requirement: Unknown fields
`deny_unknown_fields` SHALL be set on the leaf blocks, so a typo in an option is reported rather than ignored, and SHALL NOT be set on the top-level and account blocks, so a future TUI reading the same file can add its own sections without breaking this one.

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `account`, `caldav` with its endpoint, TLS and authentication, `gcal` with TLS and authentication.

An account MAY carry several; which one a shared command uses is the backend capability's business.

Every listing family SHALL additionally carry a rendering block naming its default page size and its column colours: `calendar.list`, `event.list`, `todo.list`, `journal.list` and `item.list`.

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

The resolver holds plaintext for as long as it lives, so it SHALL be built where the backends of one account are assembled and dropped with them, never held for the life of the process and never stored on a client.

A caller opening a single backend SHALL keep resolving through a resolver of its own, so the shape stays the same everywhere and nothing outlives its account.

### Requirement: Table rendering keeps its preset string
The `table.preset` option SHALL keep accepting the comfy-table v7 positional preset string, mapped onto the v8 typed style, so a configuration written against an earlier calendula stays valid. A character left out of a short string, or written as a space, SHALL leave its component undrawn.

### Requirement: A backend names the ALPN it offers
A backend whose protocol pins an HTTP version SHALL expose an `alpn` key, a list of identifiers offered during the TLS handshake. Unset offers the protocol default, an explicit empty list skips ALPN negotiation and a non-empty list replaces the default.

`gcal.alpn` defaults to `http/1.1`, the only version io-http speaks. The `caldav` block exposes none: its client pins HTTP/1.1 itself, as every DAV backend of the family does.

The runtime TLS profile SHALL be built through `TlsConfig::into_tls(alpn)`, never through a conversion assuming a list, so no caller can negotiate a protocol it did not name.

### Requirement: A defaulted scalar is not written back
A generated account SHALL omit a scalar equal to its type's default, `default = false` first among them, so what the wizard writes is what the user chose rather than the whole schema.
