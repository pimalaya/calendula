---
cairn: log
change: a-caldav-read-answered-400-is-a-miss
landed: 2026-11-15
---

# A CalDAV read answered 400 is a miss

A CalDAV item read answered 400 is `item-not-found`, as a 404 is: iCloud answers 400 to a resource name without `.ics`, so `item read nope` and `event delete <uid>` failed uncoded there. Line K20 of the CLI alignment round, from the live testing campaign (I-F1). The UID fallback the finding names left with de9ea97 (an-event-is-named-by-its-id): `event delete <uid>` now fails `item-not-found` naming `event search`, as on every server. Capability **backends** moved.

- **src/caldav/backend.rs**: `get_item` codes a 400 `item-not-found`; a write answered 400 keeps no code.
- **Evidence**: `a_read_answered_400_is_not_found_and_a_write_is_not` failed on the code before, the read carrying no code.

CHANGELOG (Changed). Spec: backends ("A server's refusal carries its code").
