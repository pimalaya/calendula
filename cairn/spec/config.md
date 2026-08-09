---
cairn: spec
capability: config
status: current
---

# Configuration

Configuration is a TOML file loaded by pimalaya-config, holding a top-level block of rendering options plus one `[accounts.<name>]` block per account, each carrying an optional sub-block per backend.

### Requirement: Loading and merging
A configuration SHALL be read from the first existing path among the canonical platform locations, or from `-c` / `CALENDULA_CONFIG` when given. Several paths MAY be passed at once, separated by `:`: the first is the base and the rest are deep-merged on top, which is how a public configuration and a private one stay separate files.

### Requirement: The global block folds under the account
The top-level rendering options SHALL be folded under the selected account, the account's own values overriding them field by field. A value set once at the top therefore applies everywhere it is not overridden.

### Requirement: Unknown fields
`deny_unknown_fields` SHALL be set on the leaf blocks, so a typo in an option is reported rather than ignored, and SHALL NOT be set on the top-level and account blocks, so a future TUI reading the same file can add its own sections without breaking this one.

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `source`, `caldav` with its endpoint, TLS and authentication, `gcal` with TLS and authentication. An account MAY carry several; which one a shared command uses is the backend capability's business.

Every listing family SHALL additionally carry a rendering block naming its default page size and its column colours: `calendar.list`, `event.list`, `todo.list`, `journal.list` and `item.list`.

### Requirement: Credentials nest under `auth`
Every backend that authenticates SHALL carry its credentials under an `auth` sub-block, so the same concept is spelled the same way across the Pimalaya CLIs: `caldav.auth.basic.password`, `gcal.auth.token`, as cardamum spells `people.auth.token` and `msgraph.auth.token`. A backend accepting exactly one kind SHALL still nest, as a struct rather than an enumeration of kinds, so gaining a second kind later is not a breaking change to the first.

### Requirement: gcal has nothing to discover
The Calendar API lives at one fixed base URL, so the `gcal` block SHALL name no server and offer no discovery route: a bearer token and a TLS profile are the whole block.

### Requirement: Paths are shell-expanded
Every configured filesystem path SHALL be shell-expanded before use, so `~` and environment variables both work. A path used raw would resolve against the working directory, which for a store root silently creates an empty one.

### Requirement: CalDAV locates its home-set three ways
The `caldav` block SHALL offer exactly three mutually exclusive routes, from most to least discovery: `discover` resolves a bare domain through RFC 6764 SRV records and the `.well-known` path, `server` names the context root the principal and home-set walk starts from, and `home` pins the home-set outright. A block carrying none of the three SHALL be rejected by name.

### Requirement: Secrets are read, never written
A CalDAV password, a CalDAV token or a gcal token SHALL be a secret read from the configuration itself or from the standard output of a command. calendula SHALL NOT write a secret anywhere: an OAuth 2.0 token broker is a command like any other, and a missing value surfaces when the account is tested. Google expires an access token within the hour, so a token broker is the practical answer there rather than a stored value.

### Requirement: Table rendering keeps its preset string
The `table.preset` option SHALL keep accepting the comfy-table v7 positional preset string, mapped onto the v8 typed style, so a configuration written against an earlier calendula stays valid. A character left out of a short string, or written as a space, SHALL leave its component undrawn.
