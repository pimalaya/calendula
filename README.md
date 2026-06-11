# 📅 Calendula [![crates.io](https://img.shields.io/crates/v/calendula.svg)](https://crates.io/crates/calendula) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

CLI to manage calendars.

> [!IMPORTANT]
> This README documents Calendula v0.2.0. If you are running v0.1.0, refer to the [v0.1.0 README](https://github.com/pimalaya/calendula/blob/v0.1.0/README.md). The [MIGRATION.md](./MIGRATION.md) guide walks v0.1 users through the breaking changes.

## Table of contents

- [Features](#features)
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
- [Usage](#usage)
- [License](#license)
- [AI disclosure](#ai-disclosure)
- [Contributing](CONTRIBUTING.md)
- [Social](#social)
- [Sponsoring](#sponsoring)

## Features

- Shared API mapping `calendars`, `events` and `items` to the active backend
- Protocol-specific APIs exposing each backend's full surface (`calendula vdir/caldav`)
- Remote backend: **CalDAV** (RFC 4791)
- Local (filesystem) backend: **vdir** [specs](https://vdirsyncer.pimutils.org/en/stable/vdir.html)
- ncal-style `event agenda` view highlighting days that carry a VEVENT
- HTTP auth support: basic, bearer
- TLS support:
  - [Rustls](https://crates.io/crates/rustls) with ring crypto
  - [Rustls](https://crates.io/crates/rustls) with aws crypto (requires `rustls-aws` feature)
  - [Native TLS](https://crates.io/crates/native-tls) (requires `native-tls` feature)
- Discovery support:
  - `.well-known/caldav` [rfc6764](https://datatracker.ietf.org/doc/html/rfc6764)
  - Current-user-principal [rfc5397](https://datatracker.ietf.org/doc/html/rfc5397)
  - Calendar-home-set [rfc4791](https://datatracker.ietf.org/doc/html/rfc4791)
- TOML configuration with multi-account support
- Interactive wizard on first run
- JSON output via `--json`

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

With only vdir support:

```sh
cargo install --locked --git https://github.com/pimalaya/calendula.git \
  --no-default-features \
  --features vdir,rustls-ring
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

Override with `calendula -c <PATH>`. Multiple paths can be passed at once, separated by `:`; the first is the base and the rest are deep-merged on top.

Run `calendula` once with no config file to launch the wizard. The wizard prompts for an account name, lets you pick a backend, then walks you through the vdir or CalDAV setup (CalDAV also asks for an email and tests the connection before saving). To edit (or add) an account later, use `calendula account configure --account <name>`.

A documented sample lives at [config.sample.toml](./config.sample.toml).

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

## Usage

Run `calendula --help` for the full command tree, and `calendula <command> --help` for any subcommand's arguments and its JSON output shape (printed when the global `--json` flag is set).

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), Opus 4.8, invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the relevant RFC or upstream spec, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale RFC references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 16/06/2026

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
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2LjY4Mi4xOTMuMTU3LjQzNy4yNi43MzIuMzEyLjI5NS4wNS42MjMuMDc2Ljk4NC4wNzZoLjk4NVptMTQuMzE0LTcuNzA2aC0uNTg4Yy0xLjEwOCAwLTEuODg4LjIyMy0yLjM0LjY2OS0uNDUuNDQ1LS42NzcgMS4xNzctLjY3NyAyLjE5NVYxNC4xYzAgMS4xNDQtLjM0IDIuMDEzLTEuMDIgMi42MDYtLjY4LjU5My0xLjYwNS44OS0yLjc3NC44OWgtMi4zODR2LTEuOTg4aC45ODRjLjM2MiAwIC42ODgtLjAyNy45OC0uMDguMjkyLS4wNTUuNTM4LS4xNTcuNzM3LS4zMDguMjA0LS4xNTcuMzU4LS4zODQuNDYtLjY4Mi4xMDMtLjI5OC4xNTQtLjY4Mi4xNTQtMS4xNTJ2LTEuMDJjMC0uODY4LjI0OC0xLjU4Ni43NDUtMi4xNTUuNDk3LS41NyAxLjE1OC0xLjAwNCAxLjk4My0xLjMwNXYtLjIxN2MtLjgyNS0uMzAxLTEuNDg2LS43MzYtMS45ODMtMS4zMDUtLjQ5Ny0uNTctLjc0NS0xLjI4OC0uNzQ1LTIuMTU1di0xLjAyYzAtLjQ3LS4wNTEtLjg1NC0uMTU0LTEuMTUyLS4xMDItLjI5OC0uMjU2LS41MjYtLjQ2LS42ODJhMS43MTkgMS43MTkgMCAwIDAtLjczNy0uMzA3IDUuMzk1IDUuMzk1IDAgMCAwLS45OC0uMDgyaC0uOTg0VjBoMi4zODRjMS4xNjkgMCAyLjA5My4yOTcgMi43NzQuODkuNjguNTkzIDEuMDIgMS40NjIgMS4wMiAyLjYwNnYxLjM0NmMwIDEuMDE4LjIyNiAxLjc1LjY3OCAyLjE5NS40NTEuNDQ2IDEuMjMxLjY2OCAyLjM0LjY2OGguNTg3eiIgZmlsbD0iI2ZmZiIvPjwvc3ZnPg==)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
