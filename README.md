# 📅 Calendula [![crates.io](https://img.shields.io/crates/v/calendula.svg)](https://crates.io/crates/calendula) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

CLI to manage calendars.

> [!IMPORTANT]
> This README documents Calendula v0.2.0. If you are running v0.1.0, refer to the [v0.1.0 README](https://github.com/pimalaya/calendula/blob/v0.1.0/README.md). The [MIGRATION.md](./MIGRATION.md) guide walks v0.1 users through the breaking changes.

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Installation](#installation)
  - [Pre-built binary](#pre-built-binary)
  - [Cargo](#cargo)
  - [Nix](#nix)
  - [Sources](#sources)
- [Configuration](#configuration)
  - [Apple](#apple)
  - [Google](#google)
  - [Microsoft](#microsoft)
  - [Fastmail](#fastmail)
  - [Proton](#proton)
  - [Posteo](#posteo)
  - [Local calendars](#local-calendars)
- [Usage](#usage)
- [AI disclosure](#ai-disclosure)
- [License](#license)
- [Social](#social)
- [Contributing](#contributing)
- [Sponsoring](#sponsoring)

## Features

- **Shared API**: `calendar`, `event` and `item` work the same whichever backend serves the account.
- **Protocol-specific APIs**: `caldav`, `pimdir` and `vdir` each expose what only that backend has.
- **CalDAV**: talk to any standard calendar server, with basic or bearer authentication.
- **vdir**: read and write a local [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html) home, one directory per calendar.
- **pimdir**: read and stage writes against a local [pimdir](https://github.com/pimalaya/pimdir) store, the offline cache a sync engine fills.
- **Agenda view**: `event agenda` draws a cal(1)-style grid marking the days that carry an event.
- **Discovery**: an email address is enough to find a provider's server, through SRV records, `.well-known` and the provider configuration documents.
- **Interactive wizard**: bare `calendula` discovers an account, tests it, and prints a ready-to-save configuration.
- **Multi-account**: one TOML file, one block per account, several files deep-merged when you want secrets apart.
- **JSON output**: every command switches to JSON with `--json`, for scripts and other tools.
- Full standard, blocking client with **TLS** support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto (requires `rustls-ring` feature, enabled by default)
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)

> [!TIP]
> Each backend sits behind its own cargo feature (`caldav`, `vdir`, `pimdir`), all enabled by default. Build with `--no-default-features` and pick the ones you need.

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

Override the path with `calendula -c <PATH>` or `CALENDULA_CONFIG=<PATH>`. Multiple paths can be passed at once, separated by `:`; the first is the base and the rest are deep-merged on top, which is how a public configuration and a private one stay separate files. The full field reference lives in [config.sample.toml](./config.sample.toml).

Run `calendula` with no command to launch the wizard. It asks one question, taking an email address, a server URL, or a local folder path, and the shape of what you type decides the rest. An address is discovered: every reachable server is offered, and picking one prompts only its credentials. A URL is taken as the CalDAV context root, which is how a self-hosted server publishing no SRV record gets configured. A folder is detected as a vdir home or a pimdir store.

The wizard tests the account before showing you anything, then prints a ready-to-save configuration. Redirect it to keep it, or let the wizard save it for you:

```sh
calendula > ~/.config/calendula/config.toml
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

Google exposes calendars via CalDAV, but only behind [OAuth 2.0](https://developers.google.com/workspace/calendar/caldav/v2/guide). Once set up, you can use any tool to manage token refreshing (for example using [Ortie](https://github.com/pimalaya/ortie)).

Google's CalDAV layout is non-standard: each calendar lives at `https://apidata.googleusercontent.com/caldav/v2/<CALENDAR-ID>/events`, and it does not enumerate the home-set the way `caldav discover` expects. So set `caldav.home` to the base URL and make the calendar id the `<CALENDAR-ID>/events` segment. `<CALENDAR-ID>` is your email for the primary calendar, or the `...@group.calendar.google.com` value from Google Calendar's *Settings and sharing > Integrate calendar > Calendar ID* for secondary ones.

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

A [pimdir](https://github.com/pimalaya/pimdir) store is the offline cache a sync engine fills: a SQLite index plus content-addressed bodies, shared with the other Pimalaya clients reading the same store. It is a cache, not a server, so calendars come from the sync and the collection verbs refuse here. Writes are staged for the next sync to push:

```toml
[accounts.cached]
pimdir.root = "~/.local/state/neverest/example"
# Usually left unset: a store synced as a single source is opened as it.
#pimdir.source = "caldav"
```

Run `calendula pimdir status` to see which source your writes are attributed to and how much of each calendar is downloaded. An item that is listed but not downloaded reads as "body not fetched" until a sync hydrates it.

## Usage

Run `calendula --help` for the full command tree, and `calendula <command> --help` for any subcommand's arguments and its JSON output shape (printed when the global `--json` flag is set).

A few real command lines:

```sh
calendula calendar list
calendula event list --calendar personal --from 2026-08-01 --to 2026-08-31
calendula event agenda -3
calendula item read --calendar personal event-1.ics
calendula pimdir status
```

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale RFC references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 08/08/2026

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Contributing

Contributions are welcome: start with [CONTRIBUTING.md](./CONTRIBUTING.md), which opens with the Pimalaya-wide guides to read first.

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2LjY4Mi4xOTMuMTU3LjQzNy4yNi43MzIuMzEyLjI5NS4wNS42MjMuMDc2Ljk4NC4wNzZoLjk4NVptMTQuMzE0LTcuNzA2aC0uNTg4Yy0xLjEwOCAwLTEuODg4LjIyMy0yLjM0LjY2OS0uNDUuNDQ1LS42NzcgMS4xNzctLjY3NyAyLjE5NVYxNC4xYzAgMS4xNDQtLjM0IDIuMDEzLTEuMDIgMi42MDYtLjY4LjU5My0xLjYwNS44OS0yLjc3NC44OWgtMi4zODR2LTEuOTg4aC45ODRjLjM2MiAwIC42ODgtLjAyNy45OC0uMDguMjkyLS4wNTUuNTM4LS4xNTcuNzM3LS4zMDguMjA0LS4xNTcuMzU4LS4zODQuNDYtLjY4Mi4xMDMtLjI5OC4xNTQtLjY4Mi4xNTQtMS4xNTJ2LTEuMDJjMC0uODY4LjI0OC0xLjU4Ni43NDUtMi4xNTUuNDk3LS41NyAxLjE1OC0xLjAwNCAxLjk4My0xLjMwNXYtLjIxN2MtLjgyNS0uMzAxLTEuNDg2LS43MzYtMS45ODMtMS4zMDUtLjQ5Ny0uNTctLjc0NS0xLjI4OC0uNzQ1LTIuMTU1di0xLjAyYzAtLjQ3LS4wNTEtLjg1NC0uMTU0LTEuMTUyLS4xMDItLjI5OC0uMjU2LS41MjYtLjQ2LS42ODJhMS43MTkgMS43MTkgMCAwIDAtLjczNy0uMzA3IDUuMzk1IDUuMzk1IDAgMCAwLS45OC0uMDgyaC0uOTg0VjBoMi4zODRjMS4xNjkgMCAyLjA5My4yOTcgMi43NzQuODkuNjguNTkzIDEuMDIgMS40NjIgMS4wMiAyLjYwNnYxLjM0NmMwIDEuMDE4LjIyNiAxLjc1LjY3OCAyLjE5NS40NTEuNDQ2IDEuMjMxLjY2OCAyLjM0LjY2OGguNTg3eiIgZmlsbD0iI2ZmZiIvPjwvc3ZnPg==)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
