---
cairn: change
id: io-webdav-0-10
status: landed
created: 2026-11-15
---

# io-webdav 0.10

> calendula's part of line W8 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), after io-webdav 0.10.0 (W5, W6, W7: its changes a-calendar-color-in-the-ical-namespace, a-calendar-update-removes and a-listed-id-is-decoded).

## Why

- **W6.** io-webdav 0.9 updated a calendar by setting alone, so calendula pumped its own PROPPATCH over the client's stream to remove a cleared description or colour (src/caldav/client.rs `proppatch` and `accepted`, change a-calendar-update-clears-and-colours-first). That request bypassed the client's resend: a server closing the connection after the listing `update_calendar` reads first failed the clear with `connection reset`. io-webdav 0.10 takes a `CaldavCalendarPatch` whose fields are doubly optional, removing a cleared property in the same PROPPATCH, through the client.
- **W7.** io-webdav 0.10 decodes a listed id when the decoding encodes back to the served segment: a calendar served at `My%20Cal` lists as `My Cal`, an item `My%20Event.ics` as `My Event.ics`. `update_calendar` looks the calendar up among the listed ones by comparing ids as strings, so `calendar update My%20Cal`, an id 0.2 printed, would fail `calendar-not-found` though it addresses the calendar; under 0.9.1, `calendar update "My Cal"` failed so though `calendar create "My Cal"` printed that id. The colour read back after `create_calendar` compares the same way.
- **W5.** io-webdav 0.10 writes a calendar's colour as Apple's `calendar-color` (`http://apple.com/ns/ical/`), which iCloud keeps, where it wrote the inf-it one iCloud ignores, and reads Apple's, the inf-it one when it is absent.

## What

- io-webdav 0.10.0 (`cargo update -p io-webdav`, the lock diff that crate alone), the requirement raised to `0.10.0`.
- src/caldav/backend.rs `update_calendar` builds a `CaldavCalendarPatch` for the colour, then one for the name and the description, each through io-webdav's `update_calendar`: the colour still goes first in a request of its own, failing ``Write the colour of calendar `<id>`, nothing else written`` before the rest is sent, and a cleared description or colour is still a `DAV:remove`. `patch` goes.
- src/caldav/client.rs: `proppatch` and `accepted` go, with the imports only they used.
- `CaldavBackend::listed` finds the listed calendar an id addresses by comparing the paths `rfc4918::collection_path` builds under the home set, so the listed id and the served segment both name it, and a segment kept as served (`a%2Fb`) is never found as `a/b`. `update_calendar`'s existence check and `create_calendar`'s colour read back use it.
- src/caldav/proppatch.rs (`caldav proppatch`, from K6) builds the patch, each flag given a set: it still clears nothing, as cardamum's `carddav proppatch`.
- The warning `create_calendar` logs when the server kept no colour still reads right: io-webdav reads the colour it now writes.
- Spec: backends ("CalDAV backend", "A CalDAV id addresses its own resource", modified).

## Scope / non-goals

- `create_calendar` returns the id as given, as cardamum's `create_addressbook` does: a name with a space prints as the listing shows it; an id given encoded prints encoded and addresses the same calendar.
- No version bump: the release is the owner's.
