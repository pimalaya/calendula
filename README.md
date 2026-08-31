# 📅 Calendula [![crates.io](https://img.shields.io/crates/v/calendula.svg)](https://crates.io/crates/calendula) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage calendars.

> [!IMPORTANT]
> This README documents Calendula v0.2.0. If you are running v0.1.0, refer to the [v0.1.0 README](https://github.com/pimalaya/calendula/blob/v0.1.0/README.md). The [MIGRATION.md](./MIGRATION.md) guide walks v0.1 users through the breaking changes.

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Shared API**: `calendar`, `event`, `todo`, `journal` and `item` work the same whichever backend serves the account.
- **One family per component kind**: `event` (VEVENT), `todo` (VTODO) and `journal` (VJOURNAL) each render the columns their kind is read by; `item` keeps the raw, unfiltered view.
- **Protocol-specific APIs**: `caldav`, `gcal`, `pimdir` and `vdir` each expose what only that backend has.
- **CalDAV**: talk to any standard calendar server, with basic or bearer authentication.
- **Google Calendar**: the native API v3, where Google's CalDAV bridge is crippled, with the iCalendar document synthesized both ways. `gcal` adds sharing, free/busy, recurrence expansion and quick add.
- **vdir**: read and write a local [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html) home, one directory per calendar.
- **pimdir**: read and stage writes against a local [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache a sync engine fills.
- **Agenda view**: `event agenda` draws a cal(1)-style grid marking the days that carry an event.
- **Discovery**: an email address is enough to find a provider's server, through SRV records, `.well-known` and the provider configuration documents.
- **Interactive wizard**: `calendula configure` discovers an account, tests it, and saves it; a first run with no configuration is offered it.
- **Multi-account**: one TOML file, one block per account, several files deep-merged when you want secrets apart.
- **JSON output**: every command switches to JSON with `--json`, for scripts and other tools, and `calendula json-schema` describes the shape each one returns.
- Full standard, blocking client with **TLS** support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto (requires `rustls-ring` feature, enabled by default)
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)

> [!TIP]
> Each backend sits behind its own cargo feature (`caldav`, `gcal`, `vdir`, `pimdir`), all enabled by default. Build with `--no-default-features` and pick the ones you need.

## RFC coverage

| RFC    | What is covered                                                                              |
|--------|----------------------------------------------------------------------------------------------|
| [4791] | CalDAV: calendar collections, calendar object resources, and the `calendar-query` REPORT with its time-range filter |
| [4918] | WebDAV: the `PROPFIND`, `PROPPATCH`, `MKCOL`, `GET`, `PUT` and `DELETE` methods CalDAV builds on |
| [5397] | Current-user-principal, the first step of the CalDAV discovery walk                           |
| [5545] | iCalendar: parsing and editing the event, to-do and journal components a calendar holds       |
| [6764] | CalDAV service discovery: the `_caldav` and `_caldavs` SRV records, and `.well-known/caldav`  |
| [6578] | Collection synchronization, whose sync token a CalDAV calendar listing reports                |
| [7617] | HTTP Basic authentication                                                                     |
| [6750] | HTTP Bearer authentication, for a provider-issued or broker-refreshed API token               |

[4791]: https://www.rfc-editor.org/rfc/rfc4791
[4918]: https://www.rfc-editor.org/rfc/rfc4918
[5397]: https://www.rfc-editor.org/rfc/rfc5397
[5545]: https://www.rfc-editor.org/rfc/rfc5545
[6578]: https://www.rfc-editor.org/rfc/rfc6578
[6750]: https://www.rfc-editor.org/rfc/rfc6750
[6764]: https://www.rfc-editor.org/rfc/rfc6764
[7617]: https://www.rfc-editor.org/rfc/rfc7617

## Installation

### Pre-built binary

As root:

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/calendula/master/install.sh | sudo sh
```

As a regular user:

```sh
curl -sSL https://raw.githubusercontent.com/pimalaya/calendula/master/install.sh | PREFIX=~/.local sh
```

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/calendula/releases) section.

For a more up-to-date version than the latest release, check out the [releases](https://github.com/pimalaya/calendula/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section. These pre-built binaries are built from the `master` branch.

> [!NOTE]
> Such binaries are built with the default cargo features. If you need specific features, please use another installation method.

### Cargo

```sh
cargo install --locked --git https://github.com/pimalaya/calendula.git
```

With only the local backends, and no network code at all:

```sh
cargo install --locked --git https://github.com/pimalaya/calendula.git \
  --no-default-features \
  --features vdir,pimdir,rustls-ring
```

### Nix

If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

```sh
nix profile install github:pimalaya/calendula
```

Or run without installing:

```sh
nix run github:pimalaya/calendula
```

### Sources

```sh
git clone https://github.com/pimalaya/calendula
cd calendula
nix run
```

## Configuration

The configuration is loaded from the first existing path among:

- `$XDG_CONFIG_HOME/calendula/config.toml`
- `$HOME/.config/calendula/config.toml`
- `$HOME/.calendularc`

Override the path with `calendula -c <PATH>` or `CALENDULA_CONFIG=<PATH>`; multiple paths can be passed at once, separated by `:`. The first is the base and the rest are deep-merged on top. The full field reference lives in [config.sample.toml](./config.sample.toml).

Run `calendula configure` to launch the wizard. It asks one question, taking an email address, a server URL or a local folder path, and the shape of what you type decides the rest.

An address is discovered: every reachable server is offered, and picking one prompts only its credentials. A URL is taken as the CalDAV context root, which is how a self-hosted server publishing no SRV record gets configured. A folder is detected as a vdir home or a pimdir store.

The wizard tests the account before showing you anything, then offers to save it: to a new configuration file, or appended to the one you already have, leaving its comments and formatting untouched.

A bare `calendula` finding no configuration offers the wizard, as does any command needing an account; once you have one, bare `calendula` prints the help.

Redirect it instead to keep the document yourself:

```sh
calendula configure > ~/.config/calendula/config.toml
```

### Apple

Apple exposes calendars via CalDAV, but you cannot use your regular password. You need to generate an [app-specific password](https://support.apple.com/en-us/HT204397) (required once two-factor authentication is on):

```toml
[accounts.example]
caldav.discover = "icloud.com"
caldav.server = "https://caldav.icloud.com/"
# The home URL is usually of this shape:
#caldav.home = "https://caldav.icloud.com/<id>/calendars/"

caldav.auth.basic.username = "example@icloud.com"
caldav.auth.basic.password.raw = "***"

calendar.default = "home"
```

### Google

Use the `gcal` backend, which speaks the [Calendar API v3](https://developers.google.com/workspace/calendar/api/v3/reference) directly. Both routes need [OAuth 2.0](https://developers.google.com/workspace/calendar/caldav/v2/guide), and an access token expires within the hour, so point the token at a broker such as [Ortie](https://github.com/pimalaya/ortie) rather than pasting one in.

```toml
[accounts.example]
gcal.auth.token.command = ["ortie", "token", "show"]

# Your email address for the primary calendar, or a
# "...@group.calendar.google.com" value for a secondary one. Run
# `calendula calendar list` to read them.
calendar.default = "example@gmail.com"
```

Google stores events as JSON and exposes no iCalendar representation of one, so calendula synthesizes the document you read and re-projects what you write.

Properties with a well-defined iCalendar slot are authoritative in both directions, and provider-scoped ones surface as read-only `X-GOOGLE-*` properties.

Google-only fields (the event colour, the guest permissions, working-location and out-of-office blocks) survive an update untouched, and everything else is stashed verbatim on the server so it round-trips instead of being dropped on the next write.

Only events project: a VTODO or VJOURNAL is refused by name, since Google models neither.

Google is also reachable over CalDAV, but only in a crippled form: bearer tokens are the sole authentication, `MKCALENDAR` is refused outright, and the discovery entry point is off-spec, so each calendar has to be addressed by hand and none can be created.

If you want it anyway, set `caldav.home` to the base URL and make the calendar id the `<CALENDAR-ID>/events` segment:

```toml
[accounts.example]
caldav.home = "https://apidata.googleusercontent.com/caldav/v2"
caldav.auth.bearer.token.command = ["ortie", "token", "show"]

# Primary calendar: "<your-email>/events".
calendar.default = "example@gmail.com/events"
```

### Microsoft

Not supported *yet*: Microsoft offers no CalDAV for calendars, only the [Graph API](https://learn.microsoft.com/en-us/graph/api/resources/calendar). Native Graph support is planned.

### Proton

Not supported: Proton exposes no calendar API, neither CalDAV nor through [Proton Bridge](https://proton.me/mail/bridge) (which proxies mail only). Calendars are reachable only from Proton's own web and mobile apps.

### Fastmail

Standard CalDAV with the mailbox address and its [app password](https://www.fastmail.help/hc/en-us/articles/360058752854-App-passwords). If `caldav.discover` / `caldav.server` return a 404, point `caldav.home` straight at the calendar home-set to skip the discovery walk:

```toml
[accounts.example]
caldav.home = "https://caldav.fastmail.com/dav/calendars/user/example@fastmail.com/"
caldav.auth.basic.username = "example@fastmail.com"
caldav.auth.basic.password.raw = "***"
```

Run `calendula calendar list` once connected to read the calendar ids (the ID column), then set `calendar.default` to the one you want.

### Posteo

Standard CalDAV with the mailbox address and its password.

```toml
[accounts.posteo]
caldav.discover = "posteo.de"
caldav.server = "https://posteo.de:8443/"
# The home URL is usually of this shape:
#caldav.home = "https://posteo.de:8443/calendars/<username>/"

caldav.auth.basic.username = "example@posteo.net"
caldav.auth.basic.password.raw = "***"

calendar.default = "default"
```

### Local calendars

No server is involved, so nothing needs discovering. Point calendula at a directory and it works offline.

A [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html) home is one directory per calendar, holding one `.ics` file per item. This is what vdirsyncer writes, and what most local tools read:

```toml
[accounts.local]
vdir.home-dir = "~/.local/share/vdirsyncer/calendars"
calendar.default = "personal"
```

A [pimdir](https://github.com/pimalaya/pimdir) store is the offline cache a sync engine fills: a SQLite index plus content-addressed bodies, shared with the other Pimalaya clients reading the same store.

It is a cache, not a server, so calendars come from the sync and the collection verbs refuse here. Writes are staged for the next sync to push:

```toml
[accounts.cached]
pimdir.root = "~/.local/state/neverest/example"
# Usually left unset: a store synced by one account is read as that one.
#pimdir.account = "personal"
```

Run `calendula pimdir status` to see which account you are reading, how much of each calendar is downloaded, and how many creations are still queued. An item that is listed but not downloaded reads as "body not fetched" until a sync hydrates it.

## Usage

Run `calendula --help` for the full command tree, and `calendula <command> --help` for any subcommand's arguments and its JSON output shape (printed when the global `--json` flag is set). `calendula json-schema --dir ./schemas` writes the JSON Schema of every `--json` payload.

A few real command lines:

```sh
calendula calendar list
calendula event list --calendar personal --from 2026-08-01 --to 2026-08-31
calendula event agenda -3
calendula todo list --calendar tasks
calendula journal list --calendar notes
calendula item read --calendar personal event-1.ics
calendula pimdir status
calendula gcal free-busy --from 2026-08-10 --to 2026-08-14
calendula gcal quick-add "Lunch with Ada tomorrow at noon"
```

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- 2026 → 2027: [NGI Zero Commons Fund](https://nlnet.nl/project/Pimalaya-pimdir/)

This program is part of Pimalaya, free software funded entirely by grants and donations. If you find it useful, consider [sponsoring](https://pimalaya.org/sponsor/) its development:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/pimalaya)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/pimalaya)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/pimalaya)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/u/gh/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
