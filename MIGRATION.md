# Migration guide

## 0.2.x -> unreleased

The next release brings calendula onto the standards Himalaya, Ortie and cardamum share: it manages events alone, drops the configuration wizard, takes one configuration path per `-c`, prints plain text with no table nor colour, serves a unified API whose backends say what they serve, and adds a REPL and an MCP server.

A 0.2 configuration loads as it is, unless it holds a `todo`, `journal`, `table` or top-level `account` table, a colour key, a non-ASCII CalDAV host, or several default accounts.

The text output changes everywhere, a few JSON outputs and exit codes move, and some commands fail earlier. The CHANGELOG lists every change; these are the ones to act on.

### Events only

- **`todo` and `journal` are gone**, with every subcommand they had. Manage to-dos with [unfog](https://github.com/pimalaya/unfog), the Pimalaya CLI for tasks, and journal entries with [journal](https://github.com/pimalaya/journal), the Pimalaya CLI for journal entries.
- **`item` still lists, reads, writes and deletes any calendar object raw**, a VTODO or a VJOURNAL included, so a script reading or writing raw to-dos keeps working through it.

### Configuration and first run

- **The wizard is gone.** Bare `calendula` prints the help, `calendula configure` (alias `wizard`) and its `calendula-configure` schema no longer exist, and no command offers to create a configuration. A configuration the wizard wrote keeps working.
- **Write a new configuration from the sample.** [config.sample.toml](./config.sample.toml), shipped as share/doc/calendula/config.sample.toml, holds one section per backend and provider: uncomment yours. Run at a terminal without a configuration, calendula prints the commands copying it in place; a script or a `--json` caller gets the error naming the path it looked at and the sample.
- **The `wizard` cargo feature is gone.** Drop it from a build's `--features`.
- **`-c` takes one path per flag.** A value is no longer split on `:`, so a Windows path stays whole. Repeat the flag to merge several files: `-c public.toml -c private.toml`, the first one the base.
- **`CALENDULA_CONFIG` is no longer read.** Pass `-c "$CALENDULA_CONFIG"` for one path, and one `-c` per path for what was a `:`-separated list.
- **Every default path is read, as layers.** $XDG_CONFIG_HOME/calendula/config.toml, $HOME/.config/calendula/config.toml and $HOME/.calendularc are each deep-merged on top of the ones before, where 0.2 read the first one found. An rc file left beside a config.toml now overrides it: delete the one you do not use.
- **Several `default = true` accounts are refused**, naming them, where 0.2 took one at random. Keep the mark on one account across the merged files, or name the account with `-a <NAME>`.
- **A `todo` or `journal` table is refused**, at the top level or under an account (`todo.list.page-size`, `[accounts.<name>.journal]`), naming the CLI that took it over: remove it.
- **Table and colour keys are refused.** Remove the `table` table (`table.preset`, `table.arrangement`), at the top level or under an account, and the top-level `account` table (`account.list.table`), both refused by name, and the `*-color` keys of `calendar.list.table`, `event.list.table` and `item.list.table`, refused as unknown keys of their block. `event.list.page-size` and `item.list.page-size` stay.
- **CalDAV hosts are ASCII.** A `caldav.server` or `caldav.home` with an internationalised host is refused, `home` when the configuration loads and `server` when the account connects: write the host in Punycode (`https://xn--bcher-kva.example/` for `bücher.example`).

### pimdir store

The pimdir backend follows pimdir storage-draft-10, sync-draft-10 and search-draft-10, and a store is never migrated: one any earlier neverest wrote is refused with the code `store-version`. Reinstall it, in this order, so no queued write is lost:

1. Run `neverest sync` with the neverest that wrote the store until nothing is queued: the writes calendula queued live in that store alone.
2. Upgrade to a neverest following storage-draft-10.
3. Run `neverest sync --reset --force -a <account>`, which moves the store aside into a sibling `<dir>.reset-<instant>` directory, deleting nothing, and syncs a new one.

The store keeps its path, so `pimdir.root` does not change. The new sync builds the search index `pimdir search` and `pimdir count` read; one an earlier draft built is refused with `index-version` until then.

An update or a delete gated on a version (`--if-match`, or an update starting from the stored event) names the body it read: when a sync replaces that body before the queue drains, the sync engine parks the write rather than apply it. It parks an update or a delete over an event a sync left conflicted too, calendula resolving no conflict. `calendula pimdir status` lists what is parked.

With `pimdir.account` unset, the backend reads the one account the store groups its collections under, and refuses a store several accounts share, naming them, where 0.2 merged their calendars: set `pimdir.account` to the one to read.

### Text output

- **Every list is plain text**, with no border, no colour and nothing sized to the terminal. A script cutting the 0.2 tables reads `--json` instead.
- **`event list` prints one line per event**, `id  start  end  summary`, the times in the local zone (a whole-day event by its days) and an empty cell as `-`, between `Account:` and `Calendar:` above and `Page:` and `Next page:` under it, each block a blank line apart. `item list` prints its entity tag, size and id the same way, and `gcal instances` its id, status, start, end and summary.
- **Other lists print columns or records.** `account list`, `vdir list`, `gcal acl list` and `gcal colors` print columns under an uppercase header; `calendar list`, `caldav list`, `gcal calendars` and `gcal free-busy` print records the way `git log` prints commits; `account check`, `caldav discover`, `gcal settings` and `pimdir status` print `Label: value` lines.
- **`event agenda` highlights no day.** Its grid writes no escape any more, today and the days holding an event included; the events listed under it name their days.
- **`-w/--max-width` is gone** from `calendar list`, `event list` and `item list`: a command passing it now fails, so drop it.
- **`-s 0` lists every item**, where it listed none.
- **`--help` prints without colour**, each paragraph on one line for the terminal to fold.

The greps the 0.2 tables allowed read `--json` with jq:

```sh
# id, start, end and summary, one event per line
calendula event list --json | jq -r '.events[] | [.id, .startsAt, .endsAt, .summary] | @tsv'
# the ids of the events whose summary mentions a meeting
calendula event list --json | jq -r '.events[] | select(.summary // "" | test("meeting"; "i")) | .id'
# the whole-day events of a week
calendula event list --from 2026-11-16 --to 2026-11-22 --json | jq -r '.events[] | select(.allDay) | .summary'
# the calendar ids
calendula calendar list --json | jq -r '.calendars[].id'
```

### JSON output

- **Writes print a receipt.** The `{"message": ...}` confirmations are gone: `calendar create|update|delete` and `event|item delete` print `{id, queued}`, the item acted on and whether the write waits for a sync; `caldav create|delete`, `vdir create|rename|delete`, `gcal acl create|update|delete`, `gcal move` and `gcal quick-add` print `{id}`; `event|item build -o` prints `{path, size}`.
- **Piped `--json` prints one value per line**, and every schema `json-schema` writes carries an `$id` naming the contract version.
- **A failure carries a stable `code`** when a caller is expected to act on it (`capability-missing`, `precondition-failed`, `item-duplicate`, `body-pending`, the pimdir store and index codes), described by the `calendula-error` schema, and `null` otherwise. Match on the code, never on the message, which may be reworded.
- **Listings say whether more follow.** `event list` and `item list` print `nextPage`, the next page number as a string, `null` once the listing is complete: give it back through `--page-token`.
- **A failed `account check` prints its report and exits 2**, each backend carrying `ok`, its failure's `code` and `error`, where 0.2 exited 0 whatever failed. Read which backends passed from `backends`.

### Commands that now fail earlier

- **A verb or a flag the account's backend does not serve is refused before connecting**, with the `capability-missing` code, naming the backend of the account that serves it: a description or a colour on a Graph calendar, refused once its token command ran in 0.2, `calendar create|update|delete` on pimdir, refused once the store was open, and `--if-match` on a vdir `event update` or `item update`, which 0.2 ignored.
- **`calendula account capability` lists what each backend of an account serves**, and each `calendar`, `event` and `item` command's `--help` ends with a `Backends:` section naming the backends serving it.
- **pimdir writes are checked before they are queued**: a create whose `UID` the calendar holds or has queued (`item-duplicate`), an update or a delete naming a version the item no longer has (`precondition-failed`), a to-do into a calendar whose source takes none, and a write no source of the store supports are refused, where 0.2 queued them for the sync engine to park or ignored the version.

### New

- `calendula repl` runs commands over one account, the unified client connecting once, and `calendula mcp` serves the unified API to an AI agent, reads only unless `--allow-write`, and nothing reaching an attendee unless `--allow-send`.
- `event build`, `event create` and `event update` take field flags (`--summary`, `--start`, `--attendee`, `--alarm`...), `event list --from/--to` expands recurring series, and `event find` reads the events carrying a `UID`.
- `pimdir search`, `pimdir count`, `pimdir reply` and `pimdir cancel` query the store's index and queue the replies and cancellations its sync engine sends.

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
