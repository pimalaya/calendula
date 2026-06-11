# Migration guide

## 0.1.x -> 0.2.0

Calendula 0.2.0 drops `pimalaya-toolbox` and the inline CalDAV implementation in favour of the new Pimalaya stack:

- `pimalaya-cli` for clap scaffolding, prompts, spinners, and tables.
- `pimalaya-config` for TOML config loading and secret resolution.
- `pimalaya-stream` for TCP/TLS streams.
- `io-vdir 0.0.3` for the filesystem-backed backend.
- `io-webdav 0.0.1` for CalDAV (RFC 4791) over WebDAV (RFC 4918).
- `io-calendar 0.0.3` for the shared `CalendarClientStd` dispatch layer.

### CLI

- The top-level command tree was reorganised. Shared commands live under `calendar`, `event`, and `item` (the plural `calendars` / `events` / `items` forms remain as hidden aliases); protocol-specific commands live under `vdir` and `caldav`.
- A new global `-b/--backend` flag selects the backend used by shared commands. The default is `auto` (first configured backend wins; vdir wins over caldav).
- `account list`, `account check`, and `account configure` were added under the new `account` subcommand.
- `completions` and `manuals` were renamed from the previous `completion` / `man` shapes.

### Configuration

- The `[accounts.<name>.caldav]` section keeps its name; locate the calendar home-set with one of `caldav.discover`, `caldav.server`, or `caldav.home`.
- `caldav.auth = "plain"` was dropped. Authentication now accepts `none`, `basic { username, password }`, or `bearer { token }`.
- The new global sections `[table]`, `[calendar]`, `[event]`, `[item]` carry per-list rendering options.
- The legacy `serde(deny_unknown_fields)` behaviour was relaxed so future TUI fields can coexist in the same TOML file.

### Wizard

- Running `calendula` without a config file proposes to bootstrap one at the default platform path (`~/.config/calendula/config.toml` on Linux).
- `calendula account configure --account <name>` runs the same wizard against an explicit account, creating it when missing.
