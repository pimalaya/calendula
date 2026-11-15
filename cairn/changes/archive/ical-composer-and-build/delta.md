---
cairn: delta
id: ical-composer-and-build
---

# Delta

## ADDED Requirements

### Requirement: An item is edited through a command, never through a pipe
A composer SHALL be a command the configuration names, spawned on the path of a temporary file holding the iCalendar, with stdin, stdout and stderr all inherited. calendula SHALL capture none of the command's streams: a composer that spawns an editor would otherwise hand it a pipe instead of the terminal, and the editor hangs or writes where nothing reads. The command edits the file in place, and calendula reads it back once the command exits.

The path SHALL be appended as the command's last argument. For a shell line that means interpolating it into the line, single-quoted, since a shell invoked as `sh -c <line> <path>` binds the path to `$0` rather than passing it on.

### Requirement: The composer's own exit is the decision
When the composer exits, what it left in the file SHALL settle the edit, and nothing SHALL be asked after it. A file that came back changed is the item, and the command writes it. A file the composer emptied, and one handed back byte for byte as it was given, are an edit given up on: nothing is written, and nothing failed. A non-zero exit status is a failure and SHALL be reported as one.

Those three outcomes SHALL be the whole protocol, so a composer owning its own save and discard is not second-guessed by a menu it cannot see, and a plain editor keeps the meaning its own quit already has.

Abandoning SHALL exit as a success. A `build` SHALL print nothing when abandoned; `create` and `update` SHALL say that nothing was written.

### Requirement: A composed item is checked before it is written
What the composer wrote SHALL be checked against the RFC 5545 contract through ical-rs's validator rather than a look at its first line. Reading is liberal and this is the strict half: a VCALENDAR missing its required PRODID, a VEVENT missing its UID or DTSTAMP, are caught here rather than by the server or by nobody.

An item that does not pass SHALL have its violations printed and SHALL offer to re-open the editor, defaulting to yes. This is not a menu: the only question is whether to fix it, and declining is an error rather than an abandon.

An iCalendar given on the command line SHALL NOT be checked, going to the backend as it was written: that is the promise the projections already make, calendula never rewriting bytes it was handed.

### Requirement: An edit is never lost
When a composed item cannot be written, whether the check was declined or the backend rejected it, the temporary file SHALL be kept and the error SHALL name it: `Cannot edit iCalendar <path>`, which says what failed and where the work is in one line.

An abandoned edit SHALL drop the file instead: an emptied one holds nothing, and an untouched one holds only what calendula put there.

### Requirement: A composer is not spawned under --json
`--json` SHALL refuse to run a composer rather than spawn one. The child inherits calendula's own stdout, where it would interleave with the JSON payload, and a consumer parsing that output has no terminal to edit in.

### Requirement: A source and the composer stack
`create` and `update` SHALL take an iCalendar source and `-i/--interactive` together: the source is the item to start from, and `-i` opens it in the composer. `build` is the same pipeline stopped before the write.

The composer SHALL be opt-in through `-i`, never opt-out, which is what keeps every verb scriptable. `-i` SHALL bail when no composer is configured rather than fall back to one. `--composer <COMMAND>` SHALL override the configured one for a single invocation, and SHALL require `-i`. The pair SHALL be one shared argument, spelled once for every command that takes it.

A `create` given no source SHALL mint the item it starts from rather than open an empty file, so no composer is asked to invent an identity. An `update` given no source SHALL start from the item the backend holds, and SHALL send the entity tag it read as `If-Match` unless `--if-match` names another: an edit that takes a minute must not silently overwrite a write that landed during it.

A command given neither a source nor `-i` has nothing to write and SHALL say so.

### Requirement: The composer is the authoring surface
calendula SHALL NOT grow per-property flags for the component families. A recurrence rule, a time zone reference and an attendee list are where a flag stops being ergonomic, and the documentation SHALL say the composer is the complete surface, so the absence reads as the boundary it is.

### Requirement: A minted item carries an identity, and its family names its kind
An item minted from nothing SHALL be a VCALENDAR carrying a PRODID and one component of the family's own kind, carrying a UID minted as a fresh UUID and a DTSTAMP of now, and no more. An editor handed an empty file mints no identity, and the CalDAV resource name and the pimdir link id both derive from the UID.

`item` names no component kind, so it SHALL mint nothing: `item build` and `item create` given no source SHALL bail, naming `event`, `todo` and `journal` as the families that can.

### Requirement: An item can be built without an account
`build` SHALL take the same iCalendar source and the same composer as `create`, apply them in the same order, and print the resulting iCalendar rather than sending it anywhere. It SHALL reach no backend.

It SHALL read no configuration, unless `-i` is given without `--composer` and the configured composer is the one thing it needs. An iCalendar can therefore be composed on a machine holding no configuration at all, and the account a family's subcommand runs against SHALL be resolved by that subcommand rather than ahead of the whole family.

`-o/--output <PATH>` SHALL write the item to that file instead of printing it, answering a message rather than the item. The flag earns its place on this command alone: the composer inherits stdout, so `event build -i > event.ics` hands the editor the file as its terminal, and with `-i` there is no redirection to fall back on.

### Requirement: A source carries an item or it is refused
A source that holds nothing but whitespace SHALL be refused, naming where it was read from, rather than read as an item. Handing a backend an empty body is never what was meant, and an abandoned `build -i` prints nothing, which pipes straight into the next command.

### Requirement: The connection is opened by the call that needs it
`CalendarClient` SHALL select its backend without connecting, and open on the first call that needs the network. A command running a composer SHALL hold no connection open while the editor is up: a server closes an idle connection, and a write landing after a long edit would read the end of a socket nobody is on the other end of any more.

## MODIFIED Requirements

### Requirement: Every data command returns a named output type
Every command answering data SHALL return a type of its own deriving `Display`, `Serialize` and `JsonSchema`, rather than answering through the printer's `Message`. `Message` SHALL carry confirmations only.

A `create`, an `update` and a `build` answer data too: an abandoned edit is an outcome they have to report, so each SHALL return an untagged enum whose write arm serializes exactly as it would on its own, and whose abandoned arm carries nothing. A `build` writing to `-o` answers a `Message`, having handed the data to a file.

Every payload key SHALL be camelCase.
