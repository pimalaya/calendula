---
cairn: change
id: the-provider-cells-close
status: landed
created: 2026-11-15
---

# The provider cells close

> Line K24 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), the low findings of the live testing campaign of 2026-11-15 (its reports/calendula/). Its iCloud colour item (I-F3) landed with [a-calendar-update-clears-and-colours-first](../a-calendar-update-clears-and-colours-first/proposal.md), its duplicate CalDAV create (I-F5) with [a-create-refuses-a-held-uid](../a-create-refuses-a-held-uid/proposal.md).

## Why

- P2-F4: `event update --start 2026-11-26T11:00` on `DTSTART;TZID=Europe/Paris:...` wrote a floating start and end, as specified, and neither Google nor Graph takes a floating time: the pushed update was rejected on every sync.
- P2-F7: `pimdir reply` queued an answer to an event Graph had made the account organise (its `ORGANIZER` the account, no `ATTENDEE`), and the next sync parked it: "You can't respond to this meeting because you're the meeting organizer". calendula holds no address of the account, so it cannot tell the organizer is the account, but an event naming no `ATTENDEE` invites nobody to answer as.
- I-F4: a VTODO created on CalDAV got a digest resource name: `new_resource_name` read the UID of a VEVENT only.
- G-F4: after `event delete`, `event read` of the Google event exited 0 with `STATUS:CANCELLED`: Google keeps a cancelled stub its `events.get` returns.
- G-F5: `gcal acl update user:x --role reader` answered HTTP 400 (no `--value`), and a `--value` disagreeing with the id answered 403 coded `permission-denied`, though the id names the scope.
- G-F7, M-F2: the Google primary calendar read `"default": false`, and an unset Graph colour read `""` where every other backend prints `null`. Graph names its default calendar too (`isDefaultCalendar`), which read `false`.

## What

- `event update`: a `--start` or `--end` naming no zone takes the zone of the event's `DTSTART` when it names one the database knows, `--time-zone` keeping the last word, before the local anchoring of gcal and msgraph (`EventFieldsArgs::inherit_zone`).
- `pimdir reply` refuses an event naming no `ATTENDEE`; the refusals of `pimdir reply` and `pimdir cancel` carry the new code `invitation-missing`.
- CalDAV `new_resource_name` reads the UID of the first VEVENT, VTODO or VJOURNAL.
- gcal: a read (`get_item`, the base of `update_item`) of a cancelled event is `item-not-found`; the primary calendar is the default.
- `gcal acl update` reads scope and value from the rule id, `--scope` and `--value` optional and refused `usage-invalid` when they disagree with it, before any request.
- Graph: an empty `hexColor` reads as none, and `isDefaultCalendar` names the default.
- Spec: commands (field flags, the gcal family, the code table), backends (CalDAV, Google Calendar, Graph, pimdir intents).
