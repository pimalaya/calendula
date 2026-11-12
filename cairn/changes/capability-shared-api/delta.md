---
cairn: delta
change: capability-shared-api
---

# Delta

Folded into cairn/spec/commands.md and cairn/spec/backends.md on 2026-11-12, the opening of commands.md saying the unified API is the union of what the backends serve.

## ADDED Requirements

### Requirement: The unified API is the union of the backends
(commands) The `calendar`, `event` and `item` families SHALL expose what any backend serves, each backend declaring what it serves of them. A backend SHALL declare a capability served only when it gives the result every other declarer gives; a backend whose result differs SHALL declare it `none` and keep its own operation in its protocol-specific family, so a verb keeps one meaning. A backend added that cannot serve a unified operation SHALL declare it `none` rather than emulate it.

### Requirement: Capability registry
(commands) Each backend SHALL declare, in one table per backend (src/capability.rs) keyed by the name `--backend` takes and compiled into every build, compiled-out backends included, each unified capability: `calendar.list`, `calendar.create`, `calendar.rename`, `calendar.delete`, `calendar.description`, `calendar.color`, `item.list`, `item.read`, `item.add`, `item.update`, `item.remove`, `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`, named in pimdir STORAGE Annex B's grammar without the domain word, the collection read as the calendar and the calendar resource as the item. Each SHALL be at a level: `full`; `partial`, with a detail saying what differs; `server`, with a detail saying what the server may refuse; or `none`, with a detail saying why and what serves it instead. The registry SHALL hold the capabilities a unified verb or flag reaches, and nothing a protocol-specific family alone reaches.

### Requirement: An unserved verb is refused before connecting
(commands) A unified command SHALL require the capabilities its verb and its flags reach (`--if-match`, `--online-meeting`, `--send-updates`, a calendar's name, description or colour) from the backend the account resolves to, before it reads a store, resolves a credential or connects. One the backend declares `none` SHALL fail with the code `capability-missing` and a message naming the backend, the capability and the detail, and, when another backend of the account serves it, that backend and the `--backend` reaching it. A capability served `partial` or `server` SHALL pass, the adapter, the server or the store refusing the case it cannot serve; an adapter MAY keep its own refusal as a second guard.

#### Scenario: A version on vdir
- GIVEN an account whose vdir home holds the event `1` and which configures a CalDAV block too
- WHEN `calendula --json item delete -k cal --if-match x 1` runs
- THEN it fails with `capability-missing` naming `item.remove.if-match`, the vdir backend and `--backend caldav`, the event file untouched and the CalDAV password command never run

### Requirement: A served verb keeps its output
(commands) Requiring the capabilities SHALL leave the output of every verb a backend serves unchanged. Only a verb or a flag a backend declares `none` changes: from an uncoded refusal, or from a flag ignored (`--if-match` on a vdir update), to `capability-missing` before anything is reached.

### Requirement: An account lists what it serves
(commands) `account capability` SHALL print, for each backend the account configures that `--backend` allows, in the order `auto` tries them, every capability with its level and detail, reading the configuration and the tables and connecting nowhere: `{"account", "backends": [{"backend", "capabilities": [{"name", "level", "detail"}]}]}`, `detail` `null` for a full one. A `--backend` naming no block of the account SHALL be refused as `account check` refuses it.

#### Scenario: A vdir account
- GIVEN the account `test` configuring a vdir home alone
- WHEN `calendula account capability` runs
- THEN it prints the header `CAPABILITY  BACKEND  LEVEL  DETAIL` and fifteen `vdir` rows, each `full` but `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`, each `none` with why

### Requirement: The help names what each backend serves
(commands) The `--help` of each unified leaf reaching a backend SHALL end with a `Backends:` footer generated from the tables: per capability the leaf reaches, flag-dependent ones included, the backends grouped by level, then each detail with the backends it applies to. A `build` reaches no backend and SHALL carry none, `-h` SHALL show none, and the top-level help SHALL keep the Pimalaya footer.

## MODIFIED Requirements

### Requirement: A JSON error carries a stable code
(commands) The code table gains, first, `capability-missing`: the backend the account resolves to does not serve a capability the command reaches; nothing was read, opened or connected (the rest of the requirement unchanged).

### Requirement: A delete can be gated on a version
(commands) `event delete` and `item delete` SHALL take `--if-match <ETAG>`. CalDAV and gcal SHALL send it as `If-Match`, msgraph SHALL compare it with the event's `changeKey`, and pimdir SHALL check it as an update does. A backend that cannot gate a write (vdir) SHALL declare `item.update.if-match` and `item.remove.if-match` `none`, so `--if-match` on an update or a delete is refused with `capability-missing` rather than dropped.

`item read` SHALL add the `etag` of the version read to its JSON, `null` when the backend has none.

### Requirement: An online meeting can be asked for
(commands) `--online-meeting` SHALL write `X-PIMDIR-ONLINE-MEETING:TRUE` on the VEVENT (pimdir STORAGE Annex B.1), replacing any instance it carried. On pimdir, which declares `online-meeting` `partial`, the write SHALL be refused before it is queued when no source declares `calendar.online-meeting` (see the backends capability); every other backend declares it `none`, so `event create` and `event update` SHALL refuse the flag there with `capability-missing`, before reaching the backend. `event build` reaches no backend and SHALL simply write the property.

### Requirement: A gcal write chooses who is mailed
(commands) The paragraph on other backends becomes: on `event create` the flag SHALL override that choice wherever the event is inserted. An event that is imported instead mails nobody: `none` SHALL pass, and `all` or `external-only` SHALL be refused, naming the event's `ORGANIZER`, which gcal's `partial` `scheduling.send-updates` says. Every other backend declares `scheduling.send-updates` `none`, so the three commands SHALL refuse the flag there with `capability-missing`, before reaching the backend. `event build` SHALL NOT take it. The scenario "Another backend" ends: THEN the command is refused with `capability-missing` naming `scheduling.send-updates` and `--send-updates`, and nothing is deleted (the rest of the requirement unchanged).

### Requirement: Text output is plain text
(commands) The `Columns` lists gain `account capability` (CAPABILITY, BACKEND, LEVEL, DETAIL), one row per capability of each backend, the detail last (the rest of the requirement unchanged).

### Requirement: Unified operation set
(backends) The shared adapters SHALL cover, per backend: `list_calendars`, `create_calendar`, `update_calendar`, `delete_calendar`, `list_items`, `get_item`, `create_item`, `update_item` and `delete_item`.

A backend that cannot model an operation SHALL declare its capability `none` in its table, the detail naming what to do instead, rather than emulate it, and its adapter MAY refuse it again.

### Requirement: Backend selection order
(backends) The unified commands SHALL target the backend the global `--backend` flag selects. Its default, `auto`, SHALL take the first configured-and-compiled backend in the order vdir, pimdir, CalDAV, gcal, msgraph, preferring a local read to a network round-trip and a protocol-standard server to a vendor API.

`auto` SHALL NOT skip a backend for a capability it lacks: every unified verb exchanges calendar or item ids the others take, so all resolve to that one backend, and a refusal names the backend of the account serving the capability.

A named value SHALL pin the command to that backend and bail when the account carries no matching configuration block. The protocol-specific commands SHALL ignore the flag.

### Requirement: Microsoft Graph backend
(backends) Graph keeps no calendar description and names colours from a palette, so msgraph SHALL declare `calendar.description` and `calendar.color` `none`, a create or update naming either refused before connecting (the rest of the requirement unchanged).

### Requirement: vdir backend
(backends) vdir has no entity tag, so it SHALL declare `item.update.if-match` and `item.remove.if-match` `none`, `--if-match` refused before anything is read, where an update ignored it (the rest of the requirement unchanged).

### Requirement: pimdir backend
(backends) Collections come from the sync, so pimdir SHALL declare `calendar.create`, `calendar.rename`, `calendar.delete`, `calendar.description` and `calendar.color` `none`, a calendar write refused before the store is opened with a detail pointing at the account the store syncs; `create_calendar`, `update_calendar` and `delete_calendar` SHALL refuse it again (the rest of the requirement unchanged).

## REMOVED Requirements

### Requirement: The unified API is a strict least common denominator
(commands) Replaced by "The unified API is the union of the backends".
