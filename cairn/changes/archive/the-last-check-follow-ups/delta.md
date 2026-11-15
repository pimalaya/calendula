---
cairn: delta
change: the-last-check-follow-ups
---

# Delta

## ADDED Requirements

### Requirement: vdir list carries the collection's metadata
(commands) `vdir list` SHALL print each collection `{id, displayName, description, color, path}` under `--json`, each of the three `null` when its marker file is absent, as cardamum's does; its text SHALL be the columns ID, PATH, COLOR, NAME, the description under `--json` alone, as `calendar list` prints it.

### Requirement: A missing item reads one sentence
(commands) A unified command naming an item its calendar does not hold SHALL fail coded `item-not-found` with one sentence whatever the backend, Himalaya's for a missing message: `item read`, `item update` and `item delete` ``Item `<id>` not found in calendar `<calendar>` ``, and `event read`, `event update` and `event delete` ``Event `<id>` not found in calendar `<calendar>`; an event is named by the id `event list` shows, `event search <UID>` finding it by its iCalendar UID``. The backend's own words SHALL follow as its `sources`.

#### Scenario: An unknown id on a vdir calendar
- GIVEN the vdir calendar `personal` holding no item `nope`
- WHEN `calendula --json item read nope` and `calendula --json event delete nope` run
- THEN the first fails coded `item-not-found` with ``Item `nope` not found in calendar `personal` ``, the second with ``Event `nope` not found in calendar `personal`; ...``, each naming io-vdir's miss in `sources`

### Requirement: The help reads as the family's
(commands) `-b/--backend` SHALL read as Himalaya's, `Force a specific backend for cross-protocol commands.`, its help listing the possible values, `auto` then every backend calendula knows in the order `auto` tries them, built in or not, each with the backend it pins. A value none is spelled SHALL be refused by clap listing them (exit 2, `usage-invalid` under `--json`); a backend the build leaves out SHALL still be refused naming the cargo feature bringing it back. `account` SHALL read `Manage accounts defined in the TOML configuration file`, `account list` `List all accounts declared in the configuration`, and each protocol namespace `<API>-specific API`: `CalDAV-specific API`, `Microsoft Graph-specific API`, `Google Calendar-specific API`, `pimdir-specific API` and `vdir-specific API`.

#### Scenario: An unknown backend
- GIVEN any build
- WHEN `calendula -b nope account list` runs
- THEN it exits 2, clap's error listing `[possible values: auto, vdir, pimdir, caldav, gcal, msgraph]`

## MODIFIED Requirements

### Requirement: Tool annotations say what a tool does
(mcp) A reading tool SHALL be `readOnlyHint`; an update or a delete SHALL be `destructiveHint`. `openWorldHint` SHALL be true exactly when the tool lists an argument inviting or notifying people: on `event_add`, `event_update` and `event_delete` under `--allow-send`, and on no tool otherwise. A gate SHALL move neither hint by itself (pimalaya-cli's mcp spec): `item_add`, `item_update` and `item_delete` list the same arguments with or without `--allow-send`, and carry the same hints.

### Requirement: The event projection
(commands) `startsAt` and `endsAt` SHALL be RFC 3339 with the offset, a zero one spelled `Z` as pimalaya-cli's output contract and Himalaya spell UTC, or `YYYY-MM-DD` with an exclusive end for a whole day. The rest of the requirement is unchanged.

### Requirement: An item's bytes are text or base64
(commands) An item SHALL print as one row in `item list`, `item read` and `event read`: `{id, calendarId, etag, contents, encoding}`, `etag` `null` when the backend versions no item. `contents` SHALL be the iCalendar text when its bytes are valid UTF-8, `encoding` then `utf-8`, and the base64 of the bytes otherwise (RFC 4648 standard alphabet, padded), `encoding` then `base64`, as cardamum prints a vCard. A read SHALL always carry both, an empty file the empty text; `item list` alone SHALL print both `null`, for an item carrying no bytes (a pimdir item whose body is not local yet or cannot be read), which a read refuses. No output SHALL decode an item's bytes lossily under `--json`.

### Requirement: An event build answers its projection
(commands) `event build` under `--json` SHALL answer `{contents, encoding, method, events}` and `item build` `{contents, encoding}`, `contents` and `encoding` as a read prints them ("An item's bytes are text or base64"). A built item that is not UTF-8 SHALL print as its bytes in text, as cardamum's `card build` does. The rest of the requirement is unchanged.

### Requirement: Text output is plain text
(commands) Among the lists printing through `Columns`, `vdir list` SHALL print ID, PATH, COLOR, NAME. The rest of the requirement is unchanged.

## REMOVED Requirements
