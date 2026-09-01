---
cairn: log
change: ical-composer-and-build
landed: 2026-09-01
---

# An item is written in a command of your choosing, and can be built without a backend

calendula could read an iCalendar object and write one, and had no way to make one: every create and update took a source it could not help you write. Cardamum landed that surface over three changes the day before, and this is the twin taking it, with the parts an iCalendar changes.

**The handoff is a file, not a pipe** (shared/composer.rs): calendula writes the item to `$TMPDIR/calendula-<uuid>.ics`, spawns the command on that path with stdin, stdout and stderr all inherited, and reads it back when the command exits. Capturing any of the three is what breaks an editor, which is why the three `Stdio::inherit()` calls are written out rather than left to the default.

The path is appended as the last argument. An argv command takes it through `Command::arg`; a shell line takes it interpolated into the line, single-quoted, because `sh -c <line> <path>` binds the path to `$0` and never passes it on. That trap is the one implementation detail worth remembering.

**The editor is the decision.** Three outcomes, one rule: a file that came back changed is the item, a file the composer emptied or handed back byte for byte is an edit given up on, and a non-zero exit is a failure. Nothing is asked after the editor, so a graphical editor owning its own save and discard is not second-guessed somewhere nobody is looking, and `:q!` keeps the meaning it already has. Cardamum shipped a `Save / Preview / Edit again / Abort` menu that morning and removed it that evening; calendula never grew one.

**`item.composer`** (config.rs, account/context.rs) is a pimalaya-config `CommandConfig` at the top level and per account, so `composer = "tcal edit"` and `composer = ["tcal", "edit"]` both work. It sits under `item` rather than under a component family on purpose: `event`, `todo`, `journal` and `item` are four views over the same resources, and what an editor opens is an iCalendar object, which is what `item` names. `--composer <COMMAND>` overrides it for one invocation and requires `-i`.

**One pipeline, on the three verbs that take a source** (shared/build.rs and the four families' create.rs and update.rs). The source and `-i` stack: the source is the item to start from, and `-i` opens it in the composer. `build` is that pipeline stopped before the write. Both positionals became optional, and a command given neither a source nor `-i` says so rather than sending an unchanged item back to the server.

**A create with no source mints its item** (shared/ical.rs `blank_item`), a VCALENDAR carrying a PRODID and one component of the family's kind carrying a UID and a DTSTAMP, built through the decoded model and encoded back rather than hand-rolled, so the 75-octet folding is ical-rs's problem. Handing a composer an empty file does not hold: an editor invents no identity, and both the CalDAV resource name and the pimdir link id derive from the UID.

**`item` mints nothing.** This is the one place the four families genuinely differ: `event`, `todo` and `journal` each name a component kind, and `item` is the raw view over any of them, so it has none to mint. `item build` and `item create` with no source bail naming the three that can, which is a real boundary rather than a gap.

**An update with no source starts from the stored item** and sends the ETag it read as `If-Match`, so an edit taking a minute cannot silently overwrite a write that landed in that minute. `--if-match` still wins, and a source given on the command line is a rewrite that was asked for.

**No field flags, deliberately.** Cardamum's `--full-name` and `--email` set one vCard property each and are worth having. The iCalendar equivalents are a recurrence rule, a time zone reference and an attendee list; `--rrule 'FREQ=WEEKLY;BYDAY=MO,WE'` is not a convenience over an editor, it is the editor with worse ergonomics. The composer is the authoring surface here, and the spec says so, so the absence reads as the boundary it is. Everything cardamum's flags bought that calendula still needed, the minted identity and the checked write, is in the composer path instead.

**What the composer wrote is checked before it is written** (shared/ical.rs `check`), through ical-rs's validator rather than a look at the first line. A card that does not pass has its violations printed and offers to re-open the editor, so the only question asked is whether to fix it, and declining is an error rather than an abandon. It catches exactly what a plain editor gets wrong: `Component VCALENDAR is missing required property PRODID`, `Component VEVENT is missing required property DTSTAMP`. An iCalendar given on the command line goes through untouched, which is the promise the projections already make.

**The connection is opened by the call that needs it** (shared/client.rs). `CalendarClient` now selects a `BackendConfig` without connecting and opens on first use, which is a bug fix rather than an optimization: `WebdavClientStd` owns one stream, opened when the client is built and discovery runs, and a server closes it while an editor is up. `create -i` connects for the first time when it creates, after the editor. `update -i` has to read the item before the editor, so it drops that connection before spawning the composer and the write opens a fresh one.

**Nothing typed is lost** (`IcalDraft`): the temporary file outlives the composer and the caller settles it with `finish`, which drops it on a write that landed and keeps it, naming the path in the error, on one that did not. Abandoning drops it too, an emptied or untouched file being nothing to lose.

**Per-subcommand account resolution** (the four cli.rs files, `CalendarClient::resolve`). `CalendulaCommand::Event` and its three siblings built a client before the subcommand ran, so every subcommand needed a configuration file and a resolvable calendar. Each family now resolves per subcommand, `build` resolving nothing unless `-i` without `--composer` needs the configured composer. Verified with an empty `HOME` and `XDG_CONFIG_HOME`: the item prints, exit 0.

**`create` and `update` answer a named output** rather than the printer's `Message`, an abandoned edit being an outcome they have to report. Both are untagged, so a write serializes as `{"id"}` and the abandoned arm carries nothing; the second shape is reachable through `-i` alone, which `--json` refuses to run, so no consumer meets it. `update` gained an id in its payload, which it did not carry before.

`getrandom` is a new dependency, for the UUID a minted UID needs (shared/uuid.rs), the same function cardamum carries.

Verified: 101 unit tests green with every feature, eight of them new over the minting, the check and the shell quoting; `clippy --all-targets` warning-free on the full set. End to end against a vdir store with a real composer: a create through the composer, an update by composer with the stored item as the base, and both abandoned. The refusals were exercised too: no source and no `-i` on all three verbs, `item build -i` and `item create` naming the three families that mint, a non-zero composer, a composer leaving the item untouched, one emptying it, one writing something that is not a valid iCalendar with the re-edit declined, `-i` with no composer configured, `--composer` without `-i`, `--json` refusing to spawn, a blank stdin source, `-o` writing the file and answering a message, and the temporary file removed on every abandon and kept on every failure.

Spec updated: `commands` (ADDED "An item is edited through a command, never through a pipe", "The composer's own exit is the decision", "A composed item is checked before it is written", "An edit is never lost", "A composer is not spawned under --json", "A source and the composer stack", "The composer is the authoring surface", "A minted item carries an identity, and its family names its kind", "An item can be built without an account", "A source carries an item or it is refused"; MODIFIED "Every data command returns a named output type"), `config` (ADDED "The composer is a command, not a library"), `backends` (ADDED "The connection is opened by the call that needs it").
