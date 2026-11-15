---
cairn: tasks
change: io-webdav-0-10
---

# Tasks

- [x] io-webdav 0.10.0 in Cargo.lock (that crate alone) and Cargo.toml (`0.10.0`).
- [x] src/caldav/backend.rs: `update_calendar` builds a `CaldavCalendarPatch` for the colour, then for the name and the description; `patch` goes; `listed` finds a calendar by the path its id addresses, for the existence check and the colour read back.
- [x] src/caldav/client.rs: `proppatch` and `accepted` go, with the imports only they used.
- [x] src/caldav/proppatch.rs: `caldav proppatch` builds the patch, setting what it is given.
- [x] Tests, src/caldav/backend.rs over the scripted server: `a_colour_is_written_in_apples_namespace` (the MKCALENDAR and a PROPPATCH setting or removing the colour name Apple's namespace, never inf-it's), `a_clear_is_sent_again_on_a_fresh_connection` (a server resetting the connection after the listing, the clear sent again on the reconnected stream), `a_calendar_is_found_by_either_spelling` (a calendar served at `My%20Cal` lists as `My Cal`, an update naming either spelling PROPPATCHes `My%20Cal`, `a/b` does not find `a%2Fb`), `a_listed_item_id_is_decoded` (`My%20Event.ics` lists as `My Event.ics`, `a%2Fb.ics` as served). All four failed on io-webdav 0.9.1 (inf-it namespace, `connection reset`, `My%20Cal` listed, `My%20Event.ics` listed); `update_calendar("My%20Cal")` fails `calendar-not-found` on 0.10 with the string comparison. The fixtures serve Apple's colour.
- [x] CHANGELOG (Changed, Fixed), MIGRATION (JSON output).
- [x] Fold the delta into backends.md, write the log entry, set `landed`.
