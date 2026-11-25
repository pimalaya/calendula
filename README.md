# 📅 Calendula [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage calendars

```sh
$ calendula event list --json
```

```json
{
  "events": [
    {
      "id": "lunch",
      "calendarId": "personal",
      "etag": null,
      "uid": "lunch",
      "recurrenceId": null,
      "summary": "Lunch with Ada",
      "description": null,
      "location": "Café de Flore",
      "start": "2026-11-20T11:30:00Z",
      "end": "2026-11-20T12:30:00Z",
      "allDay": false,
      "timeZone": "UTC",
      "zoneAssumed": false,
      "recurring": false,
      "sequence": 0,
      "status": null,
      "transparency": null,
      "busyStatus": null,
      "organizer": null,
      "attendees": [],
      "onlineMeetingUrl": null,
      "alarms": []
    }
  ],
  "nextPage": null,
  "errors": []
}
```

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Installation](#installation)
- [Configuration](#configuration)
  - [Google](#google)
  - [Microsoft](#microsoft)
  - [Fastmail](#fastmail)
  - [Posteo](#posteo)
  - [iCloud](#icloud)
- [Usage](#usage)
- [Alternatives](#alternatives)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Unified API** for calendars and events, the same on every backend
- **Protocol-specific APIs** exposing the full surface of CalDAV, Google Calendar, Microsoft Graph and vdir
- **Local stores**: vdir and pimdir, the offline store [Neverest](https://github.com/pimalaya/neverest) syncs
- **One query language** to search the events of the pimdir store
- **Agenda**: a month or a year drawn as cal(1) does, the events of its days listed under it
- **REPL** and **MCP server** running commands over a connection kept for the session
- **JSON output** of every command, described by `calendula json-schema`
- **TLS** via Rustls (`rustls-ring` default, `rustls-aws`) or Native TLS (`native-tls`), client certificates included

## RFC coverage

| RFC    | What is covered                                                                              |
|--------|----------------------------------------------------------------------------------------------|
| [4791] | CalDAV: calendar collections, calendar object resources, and the calendar-query report with its time-range filter |
| [4918] | WebDAV: the PROPFIND, PROPPATCH, MKCOL, GET, PUT and DELETE methods CalDAV builds on         |
| [5397] | Current-user-principal, the first step of the CalDAV discovery walk                           |
| [5545] | iCalendar: parsing and editing events, and carrying any other component a calendar holds     |
| [6578] | Collection synchronization, whose sync token a CalDAV calendar listing reports                |
| [6750] | HTTP Bearer authentication, for a provider-issued or broker-refreshed API token               |
| [7617] | HTTP Basic authentication                                                                     |
| [7986] | iCalendar extensions: the conference property an event's online meeting link is read from     |
| [8620] | JMAP core: the session, its capabilities and the method calls the JMAP backend sends            |
| [8984] | JSCalendar: the JMAP backend's events, converted to and from iCalendar                        |

[4791]: https://www.rfc-editor.org/rfc/rfc4791
[4918]: https://www.rfc-editor.org/rfc/rfc4918
[5397]: https://www.rfc-editor.org/rfc/rfc5397
[5545]: https://www.rfc-editor.org/rfc/rfc5545
[6578]: https://www.rfc-editor.org/rfc/rfc6578
[6750]: https://www.rfc-editor.org/rfc/rfc6750
[7617]: https://www.rfc-editor.org/rfc/rfc7617
[7986]: https://www.rfc-editor.org/rfc/rfc7986
[8620]: https://www.rfc-editor.org/rfc/rfc8620
[8984]: https://www.rfc-editor.org/rfc/rfc8984

## Installation

<details>
  <summary>Debian, Ubuntu</summary>

  Calendula is not in the official repositories yet, but the signed APT repository of [pkg.pimalaya.org](https://pkg.pimalaya.org) ships the latest release with its man pages and shell completions. Add the repository once:

  ```sh
  sudo install -d -m 0755 /etc/apt/keyrings
  curl -fsSL https://pkg.pimalaya.org/pimalaya.asc | sudo tee /etc/apt/keyrings/pimalaya.asc >/dev/null
  sudo tee /etc/apt/sources.list.d/pimalaya.sources >/dev/null <<EOF
  Types: deb
  URIs: https://pkg.pimalaya.org/apt
  Suites: stable
  Components: main
  Signed-By: /etc/apt/keyrings/pimalaya.asc
  EOF
  sudo apt update
  ```

  Then install Calendula:

  ```sh
  sudo apt install calendula
  ```
</details>

<details>
  <summary>Fedora, RHEL, Rocky Linux, AlmaLinux</summary>

  Calendula is not in the official repositories yet, but the signed DNF repository of [pkg.pimalaya.org](https://pkg.pimalaya.org) ships the latest release with its man pages and shell completions:

  ```sh
  sudo curl -fsSL -o /etc/yum.repos.d/pimalaya.repo https://pkg.pimalaya.org/pimalaya.repo
  sudo dnf install calendula
  ```
</details>

<details>
  <summary>Homebrew (macOS, Linux)</summary>

  Calendula is not in homebrew-core yet, but the [pimalaya/tap](https://github.com/pimalaya/homebrew-tap) formula installs the latest release with its man pages and shell completions:

  ```sh
  brew install pimalaya/tap/calendula
  ```
</details>

<details>
  <summary>Pre-built binary</summary>

  For other systems, Calendula can be installed with the installer:

  *As root:*

  ```sh
  curl -sSL https://raw.githubusercontent.com/pimalaya/calendula/master/install.sh | sudo sh
  ```

  *As a regular user:*

  ```sh
  curl -sSL https://raw.githubusercontent.com/pimalaya/calendula/master/install.sh | PREFIX=~/.local sh
  ```

  These commands install the latest [release](https://github.com/pimalaya/calendula/releases), built with the default cargo features. For a build of `master`, check out the [releases](https://github.com/pimalaya/calendula/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section.
</details>

<details>
  <summary>Cargo</summary>

  ```sh
  cargo install --locked calendula
  ```

  For a build of `master`:

  ```sh
  cargo install --locked --git https://github.com/pimalaya/calendula.git
  ```
</details>

<details>
  <summary>Nix</summary>

  ```sh
  nix profile install github:pimalaya/calendula
  ```
</details>

<details>
  <summary>Sources</summary>

  Requires [Rust](https://www.rust-lang.org/tools/install) 1.89 or later:

  ```sh
  git clone https://github.com/pimalaya/calendula
  cd calendula
  cargo build --release --locked
  ```

  The binary lands at target/release/calendula.
</details>

## Configuration

A TOML configuration is loaded from every existing path among the following, each one deep-merged on top of the ones before:

- `$XDG_CONFIG_HOME/calendula/config.toml`
- `$HOME/.config/calendula/config.toml`
- `$HOME/.calendularc`

Override the path with `-c <PATH>`, repeated to merge several files.

> [!TIP]
> See [config.sample.toml](./config.sample.toml) for a complete configuration of one example account with commented fields; rename the account and uncomment what you need, or start from a provider below.

An account reads a vdir home, or the pimdir store Neverest syncs, with no server: point `vdir.root` or `pimdir.root` at it. Calendula never issues nor refreshes an OAuth 2.0 token, so Google and Microsoft accounts read theirs from [Ortie](https://github.com/pimalaya/ortie).

### Google

Use the Google Calendar API, richer than Google's CalDAV bridge, with a token from Ortie's Google account. The primary calendar is named after your address, `calendula calendar list` naming the others.

```toml
[accounts.google]

gcal.auth.token.command = ["ortie", "token", "show", "-a", "google"]
calendar.default = "example@gmail.com"
```

### Microsoft

Microsoft serves calendars over Microsoft Graph alone, with a token from Ortie's Graph account.

```toml
[accounts.microsoft]

msgraph.auth.token.command = ["ortie", "token", "show", "-a", "microsoft-graph"]
```

### Fastmail

Standard CalDAV with an [app password](https://www.fastmail.help/hc/en-us/articles/360058752854), Fastmail's CalDAV taking no bearer token. `calendula calendar list` names the calendar ids `calendar.default` takes.

```toml
[accounts.fastmail]

caldav.server = "https://caldav.fastmail.com/dav/"
caldav.auth.basic.username = "example@fastmail.com"
caldav.auth.basic.password.command = "pass show fastmail"
```

### Posteo

Standard CalDAV with the account password.

```toml
[accounts.posteo]

caldav.server = "https://posteo.de:8443/"
caldav.auth.basic.username = "example@posteo.net"
caldav.auth.basic.password.command = "pass show posteo"
calendar.default = "default"
```

### iCloud

Use an [app-specific password](https://support.apple.com/en-us/HT204397), your account password being refused.

```toml
[accounts.icloud]

caldav.server = "https://caldav.icloud.com/"
caldav.auth.basic.username = "example@icloud.com"
caldav.auth.basic.password.command = "pass show icloud"
calendar.default = "home"
```

## Usage

**Step 1**: once your first account is configured, check that every backend answers:

```sh
calendula account check
```

**Step 2**: list the calendars:

```sh
calendula calendar list
```

**Step 3**: draw the current month and its events:

```sh
calendula event agenda
```

Every command runs on the default account unless `-a` names another, and its `--help` details its flags, exit codes and JSON output. More use cases:

```sh
# List the events of one week
calendula event list --from 2026-11-16 --to 2026-11-22

# Add a one-hour event
calendula event add --summary Lunch --start 2026-11-20T12:00 --duration PT1H

# Edit an event in your editor
calendula event update -i <EVENT-ID>

# Search the events of a pimdir account
calendula pimdir search budget date:2026-11

# Serve the account to an AI agent, writes allowed
calendula mcp --allow-write
```

## Alternatives

- [khal](https://github.com/pimutils/khal): a calendar CLI and TUI over a vdir, a sync tool such as vdirsyncer reaching the server
- [gcalcli](https://github.com/insanum/gcalcli): a CLI for Google Calendar alone
- [calcurse](https://calcurse.org/): a terminal calendar and todo list in its own file format, CalDAV through a helper script

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
