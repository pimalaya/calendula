# 📅 Calendula [![Matrix](https://img.shields.io/matrix/pimalaya:matrix.org?color=success&label=chat)](https://matrix.to/#/#pimalaya:matrix.org)

CLI to manage calendars

## Table of contents

- [Features](#features)
- [Usage](#usage)
  - [List calendars](#list-calendars)
  - [List calendar items](#list-calendar-items)
  - [Edit calendar item](#edit-calendar-item)
- [Installation](#installation)
- [Configuration](#configuration)
  - [Google](#google)
  - [Apple](#apple)
  - [Microsoft](#microsoft)
  - [Posteo](#posteo)
- [FAQ](#faq)
- [Sponsoring](#sponsoring)

## Features

- **CalDAV** and **Vdir** support
- Native TLS support via [native-tls](https://crates.io/crates/native-tls) crate (requires `native-tls` feature)
- Rust TLS support via [rustls](https://crates.io/crates/rustls) crate with:
  - AWS crypto support (requires `rustls-aws` feature)
  - Ring crypto support (requires `rustls-ring` feature)
- Shell command and keyring **storages** (requires `command` and `keyring` features)
- **JSON** support with `--json`

*Calendula CLI is written in [Rust](https://www.rust-lang.org/), and relies on [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to enable or disable functionalities. Default features can be found in the `features` section of the [`Cargo.toml`](https://github.com/pimalaya/calendula/blob/master/Cargo.toml#L18), or on [docs.rs](https://docs.rs/crate/calendula/latest/features).*

## Usage

### List calendars

```
$ calendula calendars list

┌─────────┬───────────────┬──────┬───────┐
│ ID      ┆ NAME          ┆ DESC ┆ COLOR │
╞═════════╪═══════════════╪══════╪═══════╡
│ default ┆ Clément DOUIN ┆      ┆       │
└─────────┴───────────────┴──────┴───────┘
```

### List calendar items

```
$ calendula items list default

┌─────────────────────────────────────────┬──────────────────────┬─────────────────────────────────────────┬───────────────────────────┐
│ ID                                      ┆ DESC                 ┆ COMPONENTS                              ┆ DATE                      │
╞═════════════════════════════════════════╪══════════════════════╪═════════════════════════════════════════╪═══════════════════════════╡
│ 289e89c0-b351-4bc3-a7a4-4ababca26161    ┆ Appeler Didier       ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-12T18:20:36+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 005ca92b-211f-4c7e-8c18-0a3644ec8792    ┆ Garage               ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-07T13:54:57+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 9045759c-1752-45e2-a9ba-d0faa72d6750    ┆ Charlie              ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-07T13:54:57+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 5a90ec56-d5f6-4e03-b143-3c992e472e16    ┆ Ball-trap            ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-07T13:54:57+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 6df73dd4-aae7-437d-8438-6f052774d77a    ┆ Garage Dacia         ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-02T09:22:05+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 9de2158d-15ca-447d-b7cd-5e3e432d140d    ┆ Garage               ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-02T09:22:05+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 942db423-edcf-4ddb-a5e5-dac5b079ab3d    ┆ Garage Fiat          ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-10-02T09:22:05+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 02405656-00a3-4242-a7c2-0e668d39ca8e    ┆ Sage femme           ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-09-20T13:15:59+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ 236b94a6-ee38-4b3e-a434-a4fd24d9c987    ┆ Internet             ┆ VCALENDAR, VEVENT, VALARM, VTIMEZONE,   ┆ 2025-09-20T13:11:56+00:00 │
│                                         ┆                      ┆ STANDARD, DAYLIGHT                      ┆                           │
└─────────────────────────────────────────┴──────────────────────┴─────────────────────────────────────────┴───────────────────────────┘
```

### Edit calendar item

```
$ calendula item update default 62196d36-65cb-4a6b-b107-f3d8dc8d8b62
```

You text editor opens with the content of your iCalendar:

```
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//hacksw/handcal//NONSGML v1.0//EN
BEGIN:VEVENT
DTSTART:19970714T170000Z
DTEND:19970715T035900Z
SUMMARY:Fête à la Bastille
END:VEVENT
END:VCALENDAR
```

Once edition done, you should see the following message:

```
Item successfully updated
```

## Installation

### Pre-built binary

Calendula CLI can be installed with the installer:

*As root:*

```
curl -sSL https://raw.githubusercontent.com/pimalaya/calendula/master/install.sh | sudo sh
```

*As a regular user:*

```
curl -sSL https://raw.githubusercontent.com/pimalaya/calendula/master/install.sh | PREFIX=~/.local sh
```

These commands install the latest binary from the GitHub [releases](https://github.com/pimalaya/calendula/releases) section.

If you want a more up-to-date version than the latest release, check out the [releases](https://github.com/pimalaya/calendula/actions/workflows/releases.yml) GitHub workflow and look for the *Artifacts* section. You should find a pre-built binary matching your OS. These pre-built binaries are built from the `master` branch, using default features.

### Cargo

Calendula CLI can be installed with [cargo](https://doc.rust-lang.org/cargo/):

```
cargo install calendula
```

*With only Vdir support:*

```
cargo install calendula --no-default-features --features vdir
```

You can also use the git repository for a more up-to-date (but less stable) version:

```
cargo install --locked --git https://github.com/pimalaya/calendula.git
```

### Nix

Calendula CLI can be installed with [Nix](https://serokell.io/blog/what-is-nix):

```
nix-env -i calendula
```

You can also use the git repository for a more up-to-date (but less stable) version:

```
nix-env -if https://github.com/pimalaya/calendula/archive/master.tar.gz
```

*Or, from within the source tree checkout:*

```
nix-env -if .
```

If you have the [Flakes](https://nixos.wiki/wiki/Flakes) feature enabled:

```
nix profile install calendula
```

*Or, from within the source tree checkout:*

```
nix profile install
```

*You can also run Calendula directly without installing it:*

```
nix run calendula
```

## Configuration

The wizard is not yet available (it should come soon, see [#7](https://github.com/pimalaya/calendula/issues/7)), so the only way to configure Calendula CLI is to copy the [sample config file](https://github.com/pimalaya/calendula/blob/master/config.sample.toml), to store it either at `~/.config/calendula.toml` or `~/.calendularc` then to customize it by commenting or uncommenting the options you need.

### Google

Google Calendar requires OAuth 2.0. The first step is to configure an OAuth 2.0 token manager like [Ortie](https://github.com/pimalaya/ortie#google), with the calendar scope `https://www.googleapis.com/auth/calendar`:

```toml
caldav.auth.bearer.command = ["ortie", "token", "show"]
```

Google Calendar does not propose discovery service, nor server URI. You need to directly use the following home URI:

```toml
caldav.home-uri = "https://apidata.googleusercontent.com/caldav/v2/your.email@gmail.com"
```

### Apple

Apple Calendars does not propose discovery service. The only way is to use their server URI combined with basic authentication:

```toml
caldav.server-uri = "https://calendars.icloud.com"
caldav.auth.basic.username = "your.email@icloud.com"
caldav.auth.basic.password.raw = "your.password"
```

If you check attentively the `--trace` logs, you should see the home URI. It is not recommended to use it directly, but it can make the CLI definitely faster. It should look like this:

```toml
caldav.home-uri = "https://p156-caldav.icloud.com:443/17170244959/calendars/"
```

### Microsoft

Microsoft only proposes a proprietary, non-standard API.

### Posteo

Posteo proposes a discovery service, combined with basic authentication:

```toml
caldav.discover.host = "posteo.de"
caldav.auth.basic.username = "your.email"
caldav.auth.basic.password.raw = "your.password"
```

Discovery is the recommended way to go, but it is slow. If you want faster calls you can "hardcode" the server URI and/or the home URI, at your own risk:

```toml
caldav.server-uri = "https://posteo.de:8843"
caldav.home-uri = "https://posteo.de:8843/calendars/your.email/"
```

## FAQ

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that helped the project to receive financial support from various programs:

- [NGI Assure](https://nlnet.nl/project/Himalaya/) in 2022
- [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/) in 2023
- [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/) in 2024 *(still ongoing)*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![thanks.dev](https://img.shields.io/badge/-thanks.dev-000000?logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQuMDk3IiBoZWlnaHQ9IjE3LjU5NyIgY2xhc3M9InctMzYgbWwtMiBsZzpteC0wIHByaW50Om14LTAgcHJpbnQ6aW52ZXJ0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik05Ljc4MyAxNy41OTdINy4zOThjLTEuMTY4IDAtMi4wOTItLjI5Ny0yLjc3My0uODktLjY4LS41OTMtMS4wMi0xLjQ2Mi0xLjAyLTIuNjA2di0xLjM0NmMwLTEuMDE4LS4yMjctMS43NS0uNjc4LTIuMTk1LS40NTItLjQ0Ni0xLjIzMi0uNjY5LTIuMzQtLjY2OUgwVjcuNzA1aC41ODdjMS4xMDggMCAxLjg4OC0uMjIyIDIuMzQtLjY2OC40NTEtLjQ0Ni42NzctMS4xNzcuNjc3LTIuMTk1VjMuNDk2YzAtMS4xNDQuMzQtMi4wMTMgMS4wMjEtMi42MDZDNS4zMDUuMjk3IDYuMjMgMCA3LjM5OCAwaDIuMzg1djEuOTg3aC0uOTg1Yy0uMzYxIDAtLjY4OC4wMjctLjk4LjA4MmExLjcxOSAxLjcxOSAwIDAgMC0uNzM2LjMwN2MtLjIwNS4xNTYtLjM1OC4zODQtLjQ2LjY4Mi0uMTAzLjI5OC0uMTU0LjY4Mi0uMTU0IDEuMTUxVjUuMjNjMCAuODY3LS4yNDkgMS41ODYtLjc0NSAyLjE1NS0uNDk3LjU2OS0xLjE1OCAxLjAwNC0xLjk4MyAxLjMwNXYuMjE3Yy44MjUuMyAxLjQ4Ni43MzYgMS45ODMgMS4zMDUuNDk2LjU3Ljc0NSAxLjI4Ny43NDUgMi4xNTR2MS4wMjFjMCAuNDcuMDUxLjg1NC4xNTMgMS4xNTIuMTAzLjI5OC4yNTYuNTI1LjQ2MS42ODIuMTkzLjE1Ny40MzcuMjYuNzMyLjMxMi4yOTUuMDUuNjIzLjA3Ni45ODQuMDc2aC45ODVabTE0LjMxNC03LjcwNmgtLjU4OGMtMS4xMDggMC0xLjg4OC4yMjMtMi4zNC42NjktLjQ1LjQ0NS0uNjc3IDEuMTc3LS42NzcgMi4xOTVWMTQuMWMwIDEuMTQ0LS4zNCAyLjAxMy0xLjAyIDIuNjA2LS42OC41OTMtMS42MDUuODktMi43NzQuODloLTIuMzg0di0xLjk4OGguOTg0Yy4zNjIgMCAuNjg4LS4wMjcuOTgtLjA4LjI5Mi0uMDU1LjUzOC0uMTU3LjczNy0uMzA4LjIwNC0uMTU3LjM1OC0uMzg0LjQ2LS42ODIuMTAzLS4yOTguMTU0LS42ODIuMTU0LTEuMTUydi0xLjAyYzAtLjg2OC4yNDgtMS41ODYuNzQ1LTIuMTU1LjQ5Ny0uNTcgMS4xNTgtMS4wMDQgMS45ODMtMS4zMDV2LS4yMTdjLS44MjUtLjMwMS0xLjQ4Ni0uNzM2LTEuOTgzLTEuMzA1LS40OTctLjU3LS43NDUtMS4yODgtLjc0NS0yLjE1NXYtMS4wMmMwLS40Ny0uMDUxLS44NTQtLjE1NC0xLjE1Mi0uMTAyLS4yOTgtLjI1Ni0uNTI2LS40Ni0uNjgyYTEuNzE5IDEuNzE5IDAgMCAwLS43MzctLjMwNyA1LjM5NSA1LjM5NSAwIDAgMC0uOTgtLjA4MmgtLjk4NFYwaDIuMzg0YzEuMTY5IDAgMi4wOTMuMjk3IDIuNzc0Ljg5LjY4LjU5MyAxLjAyIDEuNDYyIDEuMDIgMi42MDZ2MS4zNDZjMCAxLjAxOC4yMjYgMS43NS42NzggMi4xOTUuNDUxLjQ0NiAxLjIzMS42NjggMi4zNC42NjhoLjU4N3oiIGZpbGw9IiNmZmYiLz48L3N2Zz4=)](https://thanks.dev/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
