# 📅 Calendula [![crates.io](https://img.shields.io/crates/v/calendula.svg)](https://crates.io/crates/calendula) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya) [![Sponsor](https://img.shields.io/badge/sponsor-pink?style=flat&logo=github-sponsors&logoColor=white)](https://pimalaya.org/sponsor/)

CLI to manage calendars.

> [!CAUTION]
> Calendula is `v0.x`: expect breaking changes between releases until it stabilises.

## Table of contents

- [Features](#features)
- [RFC coverage](#rfc-coverage)
- [Installation](#installation)
- [Configuration](#configuration)
  - [Apple](#apple)
  - [Google](#google)
  - [Microsoft](#microsoft)
  - [Proton](#proton)
  - [Fastmail](#fastmail)
  - [Posteo](#posteo)
  - [Local calendars](#local-calendars)
- [Usage](#usage)
- [Migration](./MIGRATION.md)
- [AI policy](https://github.com/pimalaya/.github/blob/master/AI_POLICY.md)
- [License](#license)
- [Social](#social)
- [Contributing](./CONTRIBUTING.md)
- [Sponsoring](#sponsoring)

## Features

- **Unified API**: `calendar`, `event` and `item` serve the union of the backends, refusing what one does not serve.
- **Remote backends**: CalDAV, the Google Calendar API, and Microsoft Graph for Outlook and Microsoft 365.
- **Local stores**: a vdir home, or a pimdir store a sync engine fills, its writes queued and its index searched.
- **Tested sample**: config.sample.toml holds a section per backend and provider, the first run printing how to copy it.
- **REPL**: `calendula repl` runs commands over one account, the backend connecting once for the session.
- **MCP server**: `calendula mcp` serves the unified API to an agent, read-only unless `--allow-write`.
- **JSON output**: `--json` on every command, its schema given by `calendula json-schema`.
- **Full client**: TLS via Rustls (`rustls-ring` default, `rustls-aws`) or Native TLS (`native-tls`).

> [!TIP]
> Each backend sits behind its own cargo feature (`caldav`, `gcal`, `msgraph`, `vdir`, `pimdir`), all enabled by default. Build with `--no-default-features` and pick the ones you need.
>
> The default `vendored` feature builds SQLite, and OpenSSL under `native-tls`, from source; leave it out to link the system ones.

## RFC coverage

| RFC    | What is covered                                                                              |
|--------|----------------------------------------------------------------------------------------------|
| [4791] | CalDAV: calendar collections, calendar object resources, and the calendar-query report with its time-range filter |
| [4918] | WebDAV: the PROPFIND, PROPPATCH, MKCOL, GET, PUT and DELETE methods CalDAV builds on         |
| [5397] | Current-user-principal, the first step of the CalDAV discovery walk                           |
| [5545] | iCalendar: parsing and editing events, and carrying any other component a calendar holds     |
| [6578] | Collection synchronization, whose sync token a CalDAV calendar listing reports                |
| [6750] | HTTP Bearer authentication, for a provider-issued or broker-refreshed API token               |
| [6764] | CalDAV service discovery: the SRV records and the well-known path leading to a server        |
| [7617] | HTTP Basic authentication                                                                     |
| [7986] | iCalendar extensions: the conference property an event's online meeting link is read from     |

[4791]: https://www.rfc-editor.org/rfc/rfc4791
[4918]: https://www.rfc-editor.org/rfc/rfc4918
[5397]: https://www.rfc-editor.org/rfc/rfc5397
[5545]: https://www.rfc-editor.org/rfc/rfc5545
[6578]: https://www.rfc-editor.org/rfc/rfc6578
[6750]: https://www.rfc-editor.org/rfc/rfc6750
[6764]: https://www.rfc-editor.org/rfc/rfc6764
[7617]: https://www.rfc-editor.org/rfc/rfc7617
[7986]: https://www.rfc-editor.org/rfc/rfc7986

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

calendula reads a TOML configuration you write, and never writes one. [config.sample.toml](./config.sample.toml) holds one account: the options every account shares with their values, then one commented section per backend or provider, each a working account once uncommented: CalDAV, Google Calendar, Google CalDAV, Microsoft Graph, Fastmail, vdir and pimdir. Releases and packages ship it as share/doc/calendula/config.sample.toml.

The OAuth 2.0 sections read their token from `ortie token show`: calendula runs no OAuth flow and refreshes no token, so [Ortie](https://github.com/pimalaya/ortie), whose sample has the matching Google and Microsoft Graph sections, or another broker printing a fresh token, does it.

Run without a configuration at a terminal, calendula welcomes you with the commands copying the shipped sample in place, or downloading it when none ships, then the first command to run. Under `--json` or in a script, it fails naming the path it looked at and the sample.

The `caldav` block reaches its server one way: `caldav.server`, a URL or a bare authority to walk from, `caldav.discover`, a domain resolved through SRV records and `.well-known` on every run, or `caldav.home`, the calendar home set itself.

It takes one authentication: `basic` with a `username` and a `password`, `bearer` with a `token`, or `caldav.auth = "none"` for a server asking for none. `gcal.auth` and `msgraph.auth` take a `token`. A secret is given as `raw` or as `command`, a shell string or an argument list printing it.

`calendar.default`, `event.list.page-size`, `item.list.page-size` and `item.composer` may also sit at the top level, before any table, as the fallback of every account not setting them: drop the sample's default lines from an account for the top level to apply.

A configuration is loaded from every existing path among the following, each one deep-merged on top of the ones before, so $HOME/.calendularc overrides the others:

- $XDG_CONFIG_HOME/calendula/config.toml
- $HOME/.config/calendula/config.toml
- $HOME/.calendularc

At most one account may be marked `default = true` across them: several are refused, naming them, and so is an unknown `-a <NAME>`, naming the accounts the configuration holds.

Override those paths with `-c <PATH>`. Repeat it to merge several files, such as `-c public.toml -c private.toml`: the first one is the base and the next ones are deep-merged on top. Each `-c` takes one path, never split, and no environment variable is read: a path kept in a shell variable goes through the flag, as `-c "$CALENDULA_CONFIG"`.

To add an account, add an `[accounts.<name>]` table to your configuration. `calendula account list` and `calendula account check` inspect and validate the accounts declared, and `calendula account capability` lists what each of its backends serves.

### Apple

Apple exposes calendars via CalDAV, but you cannot use your regular password. You need to generate an [app-specific password](https://support.apple.com/en-us/HT204397) (required once two-factor authentication is on):

```toml
[accounts.example]
caldav.server = "https://caldav.icloud.com/"
# The home URL, skipping discovery, is usually of this shape:
#caldav.home = "https://caldav.icloud.com/<id>/calendars/"

caldav.auth.basic.username = "example@icloud.com"
caldav.auth.basic.password.raw = "***"

calendar.default = "home"
```

### Google

Use the `gcal` backend, which speaks the [Calendar API v3](https://developers.google.com/workspace/calendar/api/v3/reference) directly. Both routes need [OAuth 2.0](https://developers.google.com/workspace/calendar/caldav/v2/guide), and an access token expires within the hour, so point the token at a broker such as [Ortie](https://github.com/pimalaya/ortie) rather than pasting one in.

```toml
[accounts.example]
gcal.auth.token.command = ["ortie", "token", "show", "-a", "example"]

# Your email address for the primary calendar, or a
# "...@group.calendar.google.com" value for a secondary one. Run
# `calendula calendar list` to read them.
calendar.default = "example@gmail.com"
```

Google stores events as JSON and exposes no iCalendar representation of one, so calendula synthesizes the document you read and re-projects what you write.

Properties with a well-defined iCalendar slot are authoritative in both directions, and provider-scoped ones surface as read-only `X-GOOGLE-*` properties.

Google-only fields (the event colour, the guest permissions, working-location and out-of-office blocks) survive an update untouched, and everything else is stashed verbatim on the server so it round-trips instead of being dropped on the next write.

Only events project: a VTODO or VJOURNAL is refused by name, since Google models neither.

Google is also reachable over CalDAV, but only in a crippled form: bearer tokens are the sole authentication, `MKCALENDAR` is refused outright, and no `.well-known` path leads to the server, so `caldav.discover` finds nothing and `caldav.server` names the principal:

```toml
[accounts.example]
caldav.server = "https://apidata.googleusercontent.com/caldav/v2/"
caldav.auth.bearer.token.command = ["ortie", "token", "show", "-a", "example"]

# The primary calendar; `calendula calendar list` names the others.
calendar.default = "events"
```

### Microsoft

Use the `msgraph` backend, which speaks the [Graph API](https://learn.microsoft.com/en-us/graph/api/resources/calendar): Microsoft offers no CalDAV for calendars. Graph takes an OAuth 2.0 token with the `Calendars.ReadWrite` scope, so point the token at a broker such as [Ortie](https://github.com/pimalaya/ortie).

```toml
[accounts.example]
msgraph.auth.token.command = ["ortie", "token", "show", "-a", "example"]
```

### Proton

Not supported: Proton exposes no calendar API, neither CalDAV nor through [Proton Bridge](https://proton.me/mail/bridge) (which proxies mail only). Calendars are reachable only from Proton's own web and mobile apps.

### Fastmail

Standard CalDAV with the mailbox address and an [app password](https://www.fastmail.help/hc/en-us/articles/360058752854-App-passwords), Fastmail's CalDAV taking no bearer token:

```toml
[accounts.example]
caldav.server = "https://caldav.fastmail.com/dav/"
caldav.auth.basic.username = "example@fastmail.com"
caldav.auth.basic.password.command = "pass show fastmail"
```

Run `calendula calendar list` once connected to read the calendar ids (the first line of each record), then set `calendar.default` to the one you want.

### Posteo

Standard CalDAV with the mailbox address and its password.

```toml
[accounts.posteo]
caldav.server = "https://posteo.de:8443/"
# The home URL, skipping discovery, is usually of this shape:
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
pimdir.root = "~/.local/state/neverest"
# Usually left unset: a store synced by one account is read as that one.
#pimdir.account = "example"
```

Run `calendula pimdir status` to see which account you are reading, how much of each calendar is downloaded, how many creations are still queued and which writes the sync engine parked. An item that is listed but not downloaded reads as "body not fetched" until a sync hydrates it.

Run `calendula pimdir search <query>` to search the account's events through the search index the sync engine keeps, in the [pimdir SEARCH](https://github.com/pimalaya/pimdir) query language: `budget location:lisbon`, `attendee:carol@example.org date:2026-11`, `organizer:alice@example.org category:work`. `calendula pimdir count <query>` counts the same.

Days, `today` and `yesterday` are read in the local zone (`TZ`, then the system's); a line under the hits names the index's zone when the sync engine reads floating times and dates in another. A write still queued shows once a sync indexed it.

## Usage

Every command and flag is documented behind `--help`: `calendula --help` for the command tree, `calendula <command> --help` for a command's arguments, the backends serving it and the shape of its `--json` output. `calendula json-schema --dir ./schemas` writes the JSON Schema of every `--json` output.

A few real command lines:

```sh
calendula calendar list
calendula event list --calendar personal --from 2026-11-16 --to 2026-11-22
calendula event agenda -3
calendula event create --calendar personal --summary Lunch --start 2026-11-20T12:00 --duration PT1H
calendula event update --calendar personal -i <EVENT-ID>
calendula item read --calendar personal <ITEM-ID>
calendula account capability
calendula pimdir search budget date:2026-11
calendula gcal free-busy --from 2026-11-16 --to 2026-11-20
calendula gcal quick-add "Lunch with Ada tomorrow at noon"
```

### REPL

`calendula repl` runs the CLI's commands over one account, one per line until `exit` or EOF, the backend connecting once for the session; a program can spawn `calendula repl --json` and read one JSON value per line.

```
$ calendula repl
calendula> calendar list
calendula> event list --from 2026-11-16 --to 2026-11-22
calendula> event agenda
calendula> exit
```

### MCP

`calendula mcp` serves the unified API as [Model Context Protocol](https://modelcontextprotocol.io) tools over stdio, answering the JSON of `--json`: reads only by default, the writes under `--allow-write`, and under `--allow-send` too the attendee arguments and any write or delete touching an item that names attendees.

```json
{
  "mcpServers": {
    "calendula": { "command": "calendula", "args": ["mcp", "--account", "work", "--allow-write"] }
  }
}
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
