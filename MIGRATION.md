# Migration guide

## 0.2.x -> unreleased

### Events only

calendula is the Pimalaya CLI for events: the `todo` and `journal` commands are gone, with every subcommand they had. Manage to-dos with [unfog](https://github.com/pimalaya/unfog), the Pimalaya CLI for tasks, and journal entries with [journal](https://github.com/pimalaya/journal), the Pimalaya CLI for journal entries. `item` still lists, reads, writes and deletes any calendar object raw, a VTODO or a VJOURNAL included.

A `todo` or `journal` table left in the configuration, at the top level or under an account (`todo.list.page-size`, `[accounts.<name>.journal]`, ...), is refused when the configuration loads, naming the CLI that took it over. Remove it from the calendula configuration.

`item build -i` and `item create -i` given no source still mint nothing; their error now names `calendula event`, `unfog` and `journal` as what composes an event, a to-do and a journal entry.

### No wizard

The configuration wizard is gone: `calendula configure` (alias `wizard`), the offer a bare `calendula` and a command without a configuration made, the `wizard` cargo feature and the `calendula-configure` JSON Schema. Write the configuration by hand from [config.sample.toml](./config.sample.toml), copied to `~/.config/calendula/config.toml`: keep the blocks of the backends your account uses and fill them in. A configuration the wizard wrote keeps working as it is.

Run without a configuration at a terminal, calendula prints the commands creating it from the sample, then the first command to run; a script or a `--json` caller gets the error naming the path looked at and the sample. A bare `calendula` prints the help when a configuration exists. A build naming the `wizard` feature (`--features wizard`) must drop it.

### Configuration path

`-c` takes one path per flag and never splits it: `-c public.toml:private.toml` is one path now. Repeat the flag instead, `-c public.toml -c private.toml`, the first one the base. `CALENDULA_CONFIG` is no longer read: pass a path kept in a variable through the flag, `-c "$CALENDULA_CONFIG"`.

Several accounts marked `default = true`, across every file loaded, are refused, naming them: keep the mark on one, or name the account with `-a <NAME>`.

### Plain text output

Text output is plain text: no border, no colour, nothing sized to the terminal, and `-w/--max-width` is gone from every list. A script reading the text should read `--json` instead, whose keys are unchanged; `event list`, `event find` and `item list` add `nextPage`. In text, `event list` prints one line per event (id, start, end, summary) between a header (`Account:`, `Calendar:`) and a footer (`Page:`, `Next page:`); `calendar list` prints one record per calendar.

The table and colour options are gone and refused: remove `table.preset`, `table.arrangement`, the `*-color` keys under `calendar.list.table`, `event.list.table`, `item.list.table` and `account.list.table`, and the `table` and top-level `account` tables holding them. `event.list.page-size` and `item.list.page-size` stay.

### pimdir store

The pimdir backend follows pimdir storage-draft-09, and a store is never migrated: one any earlier neverest wrote is refused with the code `store-version`. Reinstall it, in this order, so no queued write is lost:

1. Run `neverest sync` with the neverest that wrote the store until nothing is queued: the writes calendula queued live in that store alone.
2. Upgrade to a neverest following storage-draft-09.
3. Run `neverest sync --reset --force -a <account>`, which moves the store aside into a sibling `<dir>.reset-<instant>` directory, deleting nothing, and syncs a new one.

The store keeps its path, so `pimdir.root` does not change.

### Error codes

The `--json` error codes `store-newer` and `store-stale` are gone: a store of another format version, earlier or newer, is `store-version`. `store-exposed` (a store path another user can reach, or a link), `flag-invalid` and `action-malformed` are new.

### pimdir reply and cancel

Every `calendar-reply` and `calendar-cancel` now names the source performing it, so the `source` of their JSON output is never `null`. In a calendar whose sources declare no capability, it is `--source`, else the one source syncing the calendar; a calendar several sources sync needs `--source`.

### To-dos

A calendar refuses a to-do where its sync engine states `calendar.component.vtodo` `none` (pimdir STORAGE §15.6), no longer `x-vtodo`; a sync engine stating nothing refuses none.

## 0.1.x -> 0.2.0

calendula 0.2.0 drops `pimalaya-toolbox` and the inline CalDAV implementation in favour of the Pimalaya stack, and owns its cross-protocol layer rather than borrowing one: `pimalaya-cli` for the clap scaffolding, prompts, spinners and tables, `pimalaya-config` for the TOML loading and secrets, `pimalaya-stream` for the transport, `io-webdav` for CalDAV over WebDAV, `io-vdir` for the filesystem backend, `io-pimdir` for the local store, `io-pim-discovery` for server discovery, and `ical-rs` for iCalendar.

### CLI

- The command tree was reorganised. Shared commands live under `calendar`, `event` and `item` (the plural forms remain as hidden aliases); protocol-specific commands live under `caldav`, `pimdir` and `vdir`.
- A new global `-b/--backend` flag selects the backend the shared commands use. The default is `auto`: the first configured backend wins, in the order vdir, pimdir, caldav.
- `account list` and `account check` were added under the new `account` subcommand. **`account configure` moved to the top level**, as `calendula configure`.
- `completion` and `manual` were renamed from the previous `completion` / `man` shapes; the plural spellings work as hidden aliases.
- `event list` gained `--from` and `--to` (YYYY-MM-DD, both inclusive). A range returns every match rather than the first page.

### Item ids

CalDAV item ids are now the resource names the server returned, verbatim: calendula neither appends nor strips a `.ics` extension. This fixes read, update and delete addressing the wrong resource whenever an id did not end in `.ics`, and a create returning an id that addressed nothing when the server named the resource itself.

An id a listing shows now round-trips through every verb, so the usual workflow is unaffected. A script that built an id by hand, stripping or appending `.ics` around a listed one, must stop doing so.

### Configuration

- The `[accounts.<name>.caldav]` block keeps its name. Locate the calendar home-set with exactly one of `caldav.discover`, `caldav.server` or `caldav.home`.
- `caldav.auth = "plain"` was dropped. Authentication accepts `none`, `basic { username, password }` or `bearer { token }`.
- A new `[accounts.<name>.pimdir]` block configures a local pimdir store, with a `root` and an optional `account`.
- The global `[table]`, `[calendar]`, `[event]` and `[item]` sections carry the per-list rendering options.
- `deny_unknown_fields` was relaxed on the top-level and account blocks, so a future TUI can share the same file. It stays on the leaf blocks, so a typo in an option is still reported.
- `table.preset` keeps its comfy-table v7 string. The table library moved to v8 underneath, so its default truncation indicator changes from `...` to `…`.

### Wizard

`calendula configure` (alias `wizard`) launches the wizard. It replaces the previous flow entirely.

- It asks one question, taking an email address, a server URL, or a local folder path. It no longer asks you to confirm, to name the account, or to pick a backend: the account name is derived from what you type, and the backend follows from its shape.
- An address runs discovery against every mechanism in parallel, under a short deadline, and offers each reachable server. A `http`, `https`, `caldav` or `caldavs` URL is taken as the context root, which is how a self-hosted server publishing no SRV record gets configured. A folder is detected as a vdir home or a pimdir store from its own markers.
- The account is tested before anything is printed, so a bad credential stops the wizard rather than producing a configuration that cannot connect.
- The generated account is saved to a configuration file that does not exist yet, or appended to the one you already have, leaving its comments and formatting untouched. It never overwrites. Redirecting stdout prints the document instead, so `calendula configure > ~/.config/calendula/config.toml` still works.
- Bare `calendula` no longer runs the wizard when a configuration exists: it prints the help. With no configuration, it offers the wizard, as does any command needing an account.
