---
cairn: change
id: a-caldav-read-answered-400-is-a-miss
status: landed
created: 2026-11-15
---

# A CalDAV read answered 400 is a miss

> Line K20 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), from the live testing campaign of 2026-11-15 (its reports/calendula/caldav-icloud.md I-F1).

## Why

iCloud answers 400, not 404, to a GET of a resource name without `.ics`. At a1ca6dd `event delete <uid>` read the id first and stopped on that uncoded 400 before its UID fallback, and `item read nope` failed uncoded too. Since [an-event-is-named-by-its-id](../an-event-is-named-by-its-id/proposal.md) (de9ea97) `event delete` has no UID fallback, so what is left is the code: the read of an id the server does not take names no item, and `event delete <uid>` should fail `item-not-found` pointing at `event search`, as it does on every other server.

## What

- `CaldavBackend::get_item`: a 400 is `item-not-found`, as a 404 or a 410 is, the server's answer in `sources`. A GET sends no body for the server to refuse, so a 400 to it refuses the name. A write answered 400 keeps no code: its body is what the server refused.
- Spec: backends ("A server's refusal carries its code").
