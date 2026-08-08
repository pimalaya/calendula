# Migration guide

## 0.1.x -> 0.2.0

calendula 0.2.0 drops `pimalaya-toolbox` and the inline CalDAV implementation in favour of the Pimalaya stack, and owns its cross-protocol layer rather than borrowing one: `pimalaya-cli` for the clap scaffolding, prompts, spinners and tables, `pimalaya-config` for the TOML loading and secrets, `pimalaya-stream` for the transport, `io-webdav` for CalDAV over WebDAV, `io-vdir` for the filesystem backend, `io-pimdir` and `io-replica` for the local store, `io-pim-discovery` for server discovery, and `ical-rs` for iCalendar.

### CLI

- The command tree was reorganised. Shared commands live under `calendar`, `event` and `item` (the plural forms remain as hidden aliases); protocol-specific commands live under `caldav`, `pimdir` and `vdir`.
- A new global `-b/--backend` flag selects the backend the shared commands use. The default is `auto`: the first configured backend wins, in the order vdir, pimdir, caldav.
- `account list` and `account check` were added under the new `account` subcommand. **`account configure` does not exist**: the wizard prints a configuration rather than writing one, so generate an account with `calendula` and merge it into your file.
- `completions` and `manuals` were renamed from the previous `completion` / `man` shapes.
- `event list` gained `--from` and `--to` (YYYY-MM-DD, both inclusive). A range returns every match rather than the first page.

### Item ids

CalDAV item ids are now the resource names the server returned, verbatim: calendula neither appends nor strips a `.ics` extension. This fixes read, update and delete addressing the wrong resource whenever an id did not end in `.ics`, and a create returning an id that addressed nothing when the server named the resource itself.

An id a listing shows now round-trips through every verb, so the usual workflow is unaffected. A script that built an id by hand, stripping or appending `.ics` around a listed one, must stop doing so.

### Configuration

- The `[accounts.<name>.caldav]` block keeps its name. Locate the calendar home-set with exactly one of `caldav.discover`, `caldav.server` or `caldav.home`.
- `caldav.auth = "plain"` was dropped. Authentication accepts `none`, `basic { username, password }` or `bearer { token }`.
- A new `[accounts.<name>.pimdir]` block configures a local pimdir store, with a `root` and an optional `source`.
- The global `[table]`, `[calendar]`, `[event]` and `[item]` sections carry the per-list rendering options.
- `deny_unknown_fields` was relaxed on the top-level and account blocks, so a future TUI can share the same file. It stays on the leaf blocks, so a typo in an option is still reported.
- `table.preset` keeps its comfy-table v7 string. The table library moved to v8 underneath, so its default truncation indicator changes from `...` to `…`.

### Wizard

Running `calendula` with no command launches the wizard. It replaces the previous flow entirely.

- It asks one question, taking an email address, a server URL, or a local folder path. It no longer asks you to confirm, to name the account, or to pick a backend: the account name is derived from what you type, and the backend follows from its shape.
- An address runs discovery against every mechanism in parallel, under a short deadline, and offers each reachable server. A `http`, `https`, `caldav` or `caldavs` URL is taken as the context root, which is how a self-hosted server publishing no SRV record gets configured. A folder is detected as a vdir home or a pimdir store from its own markers.
- The account is tested before anything is printed, so a bad credential stops the wizard rather than producing a configuration that cannot connect.
- **It writes nothing on its own.** The generated account is printed on stdout, so `calendula > ~/.config/calendula/config.toml` is the write-back. When stdout is a terminal it offers to save the file for you, and never overwrites an existing one without asking.
- A command that finds no configuration now fails with a message pointing at the wizard, instead of launching it mid-command.
