---
cairn: change
id: the-last-check-follow-ups
status: landed
created: 2026-11-15
---

# The last check's follow-ups

> Rows M2, M8, L2, L6, L7, L8, L9, L10, L11 and L19 of X3, the last check of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15/reports/x3-2026-11-15.md); cardamum is the twin, Himalaya the reference.

## Why

X3 compared the six binaries once the round's lines had landed and found calendula apart from the family in ten places: `openWorldHint` moved by `--allow-send` on item writes listing the same arguments, against pimalaya-cli's mcp spec (M2); `startsAt` and `endsAt` spelled UTC `+00:00` where Himalaya and pimalaya-cli's output contract spell it `Z` (M8); `vdir list` rows lacked the description and the colour cardamum's carry (L2); reads printed `contents` and `encoding` nullable where cardamum's are not, and `item build` printed no `encoding` (L6); the `-b` help, the refusal of an unknown `-b`, the `account` abouts and the namespace abouts were worded apart from Himalaya's (L9, L10, L11); a missing item read `Read item <x> from calendar <y>`, or the backend's own words, where Himalaya says ``Message `x` not found in `<mailbox>` `` (L19).

## What

- MCP: `openWorldHint` follows the arguments a tool lists, true on `event_add`, `event_update` and `event_delete` under `--allow-send`, which list `attendee`, `organizer` or `send-updates`; `item_add`, `item_update` and `item_delete` keep `false` whatever the gate.
- `startsAt` and `endsAt` spell a zero offset `Z`, any other as resolved.
- `vdir list` prints `{id, displayName, description, color, path}`, its text the columns ID, PATH, COLOR, NAME.
- A read (`item read`, `event read`, `caldav get`, `caldav report query|multiget`, `vdir item get`) always carries `contents` and `encoding`, an empty file the empty text; `item list` alone keeps both `null`, for a pimdir item whose body is not local yet or cannot be read, which a read refuses (`body-pending`). `item build` and `event build` print `encoding` beside `contents`, and a built item that is not UTF-8 prints in base64 under `--json` and as its bytes in text, as cardamum's `card build` does, where it failed.
- `-b` reads as Himalaya's, its help listing `auto` and every backend calendula knows, built in or not; an unknown value is refused by clap listing them; one the build leaves out still names its cargo feature. `account` reads `Manage accounts defined in the TOML configuration file`, `account list` `List all accounts declared in the configuration`, and `gcal`, `pimdir` and `vdir` `<API>-specific API`, as `caldav` and `msgraph` already did.
- A missing item reads ``Item `x` not found in calendar `y` `` on `item read|update|delete` and ``Event `x` not found in calendar `y`; ...`` (how an event is named) on `event read|update|delete`, whatever the backend, the code unchanged and the backend's words in `sources`.
- Not done: L7, `tag-refused` is raised by io-pimdir's search tag alone (`PimdirError::Tag`), which calendula never calls, so it stays out of the relayed codes as backends.md says (domain); L8, cardamum's `-k` reads `<ADDRESSBOOK-ID>` (cardamum/src/shared/arg.rs:14), the twin of calendula's `<CALENDAR-ID>`, so the value names already match.

None of the outputs changed shipped in 0.2: `startsAt`, `mcp`, the item row and the build projection are new since, so nothing here breaks a 0.2 user. CHANGELOG lines are reworded where the [Unreleased] ones stated the old shapes.
