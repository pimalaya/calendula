# calendula architecture

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (layering, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to calendula, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where calendula fits

calendula is an **application**, the top layer of the Pimalaya stack: it has no library target (only `main.rs`) and writes no protocol or storage logic of its own. It is a thin CLI shell that drives the sans-I/O libraries below it:

- [io-calendar](https://github.com/pimalaya/io-calendar): the cross-protocol calendar/item domain API;
- [io-webdav](https://github.com/pimalaya/io-webdav): the CalDAV (WebDAV) protocol coroutines;
- [io-vdir](https://github.com/pimalaya/io-vdir): the local vdir filesystem coroutines;
- [io-http](https://github.com/pimalaya/io-http): the HTTP request/response state machines WebDAV runs on;
- [pimconf](https://github.com/pimalaya/pimconf): CalDAV server discovery (RFC 6764 SRV + TXT + `.well-known`);
- [pimalaya-cli](https://github.com/pimalaya/cli), [pimalaya-config](https://github.com/pimalaya/config), [pimalaya-stream](https://github.com/pimalaya/stream): shared CLI plumbing (clap args, printer, logger), TOML config loading, and the blocking I/O runtime.

The sans-I/O coroutines live in those libraries; calendula never implements one. It consumes their blocking `*Std` clients (`CalendarClientStd`, `WebdavClientStd`, `VdirClient`), which run pimalaya-stream under the hood. So all real I/O (network, filesystem, clock, DNS) is concentrated in the libraries; calendula only orchestrates them and renders results.

## Three command families

The command tree (`cli.rs`) is split into three groups, in this order:

1. **Shared API** (`calendar`, `event`, `item`): the cross-protocol, least-common-denominator surface. Every operation here works the same regardless of which backend serves the active account. Backend-specific concepts that are not common to both backends do not appear here.
2. **Protocol-specific APIs** (`caldav`, `vdir`): each exposes the full surface of one backend, including operations the shared API cannot model (`caldav discover`, `vdir rename`). Each is gated behind its own cargo feature.
3. **Meta** (`account`, `completions`, `manuals`): account configuration and inspection, shell completions, and man pages.

This is the standard Pimalaya CLI split: a portable shared API plus per-protocol escape hatches.

### event vs item

The shared API exposes two views over the same calendar items, because a calendar collection mixes component kinds (VEVENT, VTODO, VJOURNAL):

- `item` is the raw, unfiltered view: it lists, reads, creates, updates and deletes any iCalendar item by id, leaving the bytes untouched.
- `event` is the VEVENT-focused view: `event list` filters out non-VEVENT components and renders summary/start/end columns, and `event agenda` draws a cal(1)-style grid highlighting days that carry a VEVENT. Its create/read/update/delete operate on the same items as `item`, scoped to events.

Both share the `-k/--calendar` selector and the same io-calendar item API; only the rendering and the VEVENT filter differ.

## Backend selection

The shared commands target a backend chosen by the global `--backend` flag, a `Backend` enum (`backend.rs`) with `auto` (default), `caldav` and `vdir` variants (each named variant gated behind its feature):

- `auto` picks the first configured-and-allowed backend in calendula's priority order (vdir wins over caldav when both are configured);
- a named value pins the command to that backend, and bails if the account has no matching config block.

The shared commands receive a `CalendarClient` (`shared/client.rs`): a wrapper that holds the merged `Account` plus an inner `CalendarClientStd`. `CalendarClientStd` is an enum holding exactly one backend, built by trying each allowed backend in turn and wrapping the first configured one via `From`. The wrapper `Deref`s onto the inner client, so command code calls the io-calendar API directly. The protocol-specific commands skip this entirely and build their own `CaldavClient` / `VdirClient`, ignoring `--backend`.

## Command conventions

Each subcommand is a clap-derived struct carrying its own arguments, with an `execute(self, printer, client)` method (the shared nested-execute convention). `CalendulaCommand::execute` in `cli.rs` is the single dispatch point: it loads the config (running the wizard if none exists), selects the account, builds the appropriate client, and hands it to the subcommand.

Shared commands that operate inside a calendar take the calendar through the shared `CalendarIdArg` (`shared/arg.rs`): a flattened `-k/--calendar` flag resolved by `Account::calendar_id` (the flag wins, otherwise `calendar.default`, otherwise the command bails). `calendar delete` is the one exception: it inlines a mandatory `-k/--calendar`, never falling back to a default.

Output follows the Pimalaya stdout/stderr rule: all data and errors go to stdout through `pimalaya_cli::printer` (with `--json` switching every command to JSON), and stderr carries logs only. A command returns a `Serialize + Display` type to the printer rather than printing inline.

Each command's doc comment is its `--help` text: the first paragraph is the short summary shown by `-h`, and the full text (shown by `--help`) ends with the command's JSON output shape. So `calendula <command> --help` is the canonical usage reference for both humans and AI agents; the README intentionally documents no per-command usage.

## Configuration and the wizard

Config is loaded by pimalaya-config from the first existing path among the three canonical locations (or the `-c` / `CALENDULA_CONFIG` override), with later paths deep-merged on top of the first. The schema is multi-account: a top-level block plus named `[accounts.<name>]` blocks, each carrying an optional `[caldav]` and/or `[vdir]` sub-block. `Account::from(config).merge(Account::from(account_config))` flattens the global defaults under the selected account.

When no config file exists, `load_or_wizard` runs the interactive wizard (`wizard/`) to bootstrap one, prompting for an account, then walking the vdir or CalDAV setup before writing the file at the target path.

## CalDAV discovery

The `[caldav]` block resolves the calendar home set through one of three routes, in decreasing order of magic (`caldav/client.rs`, `connect_and_resolve`):

- `home`: short-circuits all discovery; the configured URL is used as the calendar home set directly;
- `server`: connects to the given context root, then walks current-user-principal (RFC 5397) and calendar-home-set (RFC 4791);
- `discover`: resolves a bare domain to a context root through pimconf (RFC 6764 SRV + TXT + `.well-known`) before running that same walk.

## Module layout

```
src/
  main.rs                entry point: parse Cli, build printer, dispatch
  cli.rs                 Cli/Command, global flags, execute dispatch
  backend.rs             Backend enum (auto/caldav/vdir) + selection rules
  config.rs              TOML schema: Config, AccountConfig, per-backend blocks
  shared/                cross-protocol least-common-denominator API
    arg.rs               CalendarIdArg (shared -k/--calendar flag)
    client.rs            CalendarClient wrapper (picks one backend)
    ical.rs              IcalArg (path / raw / stdin iCalendar source)
    calendars/           calendar list/create/update/delete
    events/              event agenda/list/read/create/update/delete
    items/               item list/read/create/update/delete (raw view)
  caldav/                [caldav] protocol-specific API
    client.rs            WebdavClientStd builder + discovery routes
    discover/list/create/delete
  vdir/                  [vdir] protocol-specific API
    client.rs            VdirClient builder
    list/create/rename/delete
  account/               account list/check/configure + Account context
  wizard/                first-run interactive config bootstrap
```

`shared/` is the portable surface; `caldav/` and `vdir/` are the per-protocol escape hatches; `account/` and `wizard/` are the meta and bootstrap concerns.
