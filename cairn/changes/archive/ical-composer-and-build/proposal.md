---
cairn: change
id: ical-composer-and-build
status: landed
created: 2026-09-01
---

# An item is refined in a command of your choosing, and can be built without a backend

calendula can read an iCalendar object and write one, and has no way to make one: every create and update takes a source it cannot help you write. Cardamum landed that surface over three changes on 2026-08-31 (card-composer, card-build-command, the-editor-is-the-decision), and calendula is the twin that has none of it.

## Why

`event create -k work event.ics` assumes the file already exists. Writing it means knowing that VCALENDAR needs a PRODID, that VEVENT needs a UID and a DTSTAMP, and that the whole thing folds at 75 octets. Nothing in calendula helps, and the first look anyone gets at what they wrote is the item the server already holds.

## What

**A composer**, the command an item is edited through: calendula writes the iCalendar to a temporary file, spawns the configured command on its path with every stream inherited, and reads it back. `item.composer` names it, `--composer` overrides it, and `-i/--interactive` opts in on `create` and `update` of all four families.

The composer's own exit is the decision, as it is in cardamum: changed bytes are the item, an emptied or untouched file is an edit given up on, and a non-zero exit is a failure. No menu.

**A `build` verb** per family, the create pipeline stopped one step early: it applies the source and the composer, prints the iCalendar instead of sending it, and reaches no backend. `-o/--output` captures it, since `-i` owns stdout.

**What calendula does not take from cardamum**: the field flags. Cardamum's `--full-name` / `--email` set one vCard property each; the iCalendar equivalent is a recurrence rule, a time zone reference and an attendee list, which is where a flag stops being ergonomic and the composer starts. The composer is the whole authoring surface here, and the source is the other half.

## Consequences

- `create` and `update` take an optional source on all four families, and bail when given neither a source nor `-i`.
- `create` and `update` answer a named output type rather than the printer's `Message`, having an abandoned outcome to report.
- `CalendarClient` connects on the first call that needs it, and drops the connection before a composer runs: a CalDAV server closes an idle socket while an editor is up.
- The four families resolve their account per subcommand, so `build` runs on a machine holding no configuration.
- `item` names no component kind, so it mints nothing: `item build` and `item create` given no source bail and name the three families that can.
