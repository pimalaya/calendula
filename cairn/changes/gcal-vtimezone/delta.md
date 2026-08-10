---
cairn: change
change: gcal-vtimezone
---

# Delta

## ADDED Requirements

### Requirement: A named zone arrives with its definition
Every zone the synthesized document names in a TZID SHALL be defined by a VTIMEZONE the same document carries, as RFC 5545 3.2.19 requires. The Calendar API carries the IANA name alone, so the definition SHALL be minted from a time zone database the gcal feature carries rather than read from the resource.

The zones owed a definition SHALL be read from the finished document, after the recurrence lines and the stash have been spliced in, since a TZID reaches the document through those too and not only through the boundaries. The reading SHALL resolve RFC 5545 3.1 folds first, and SHALL consider only the parameter section of a line, so free text spelling `TZID=` conjures nothing. A zone the document already defines SHALL NOT be minted again.

A definition SHALL describe the era the item falls in rather than the zone's whole record: the observances in force at the item's start, as a standard and daylight pair, each carrying its offsets, a DTSTART that is an occurrence of its own rule, and a yearly RRULE where the following transitions agree on one. A zone that never shifts, and one whose nearest shift is more than two years from the item, SHALL state one observance and no rule, so a rule a zone abandoned decades ago is not shipped with a present-day item.

The definitions SHALL precede the VEVENT that references them, and a zone named twice SHALL be defined once. A name the database does not know SHALL mint nothing, leaving the TZID as it was rather than inventing a zone.

## MODIFIED Requirements

### Requirement: The remainder is stashed verbatim
Every iCalendar line the projection neither manages nor mints SHALL be stashed verbatim in `extendedProperties.private` and spliced back on read, so a property no Google field models survives a round-trip instead of being dropped on the next write. The remainder SHALL be recomputed from the incoming iCalendar on every write, so the stash never drifts.

The VEVENT remainder and the VCALENDAR-level remainder SHALL ride separate key families, since a calendar-level property cannot legally be spliced inside a VEVENT.

A VTIMEZONE naming a zone the database knows SHALL NOT be stashed, since the projection mints it again on every read: a definition runs to a dozen lines or more, and the stash budget is better spent on lines nothing could rebuild. One naming a zone the database does not know SHALL be stashed like any other remainder. A definition already stashed SHALL be spliced back as it stands and SHALL NOT be doubled by a minted one.

An extended property another client owns SHALL survive a stash rewrite: only the chunks under calendula's own key prefixes are replaced.

### Requirement: An instant outranks a display zone
Google returns a boundary as an absolute instant plus the calendar's display zone, a pair no iCalendar boundary expresses. Only the offset SHALL decide the instant: a `Z`-stamped boundary SHALL project as a UTC stamp even when a zone is named alongside it, since reading the literal time as that zone's wall time would shift the event by the zone's offset.

The display zone SHALL therefore be treated as a provider-only field and carried over from the server copy on update, so a zoned recurring series keeps expanding where it did instead of falling back to UTC and drifting by an hour after a daylight-saving change. Only a UTC-stamped boundary SHALL take the server's zone: an offset-less one is wall time in the zone it names, and relabelling it would move the event.

The rule stands whatever database the projection carries. Deriving a wall time from an instant and a zone name is a question about the instant, which the stamp already answers; minting a VTIMEZONE is a question about the zone, which the name answers. The second is now available and does not license the first.

### Requirement: One cargo feature per backend
Each backend SHALL sit behind its own cargo feature (`caldav`, `gcal`, `vdir`, `pimdir`), pulling its io-* crates through `dep:`. A build SHALL compile at least one of them; a build with none is not a supported configuration, since the product would have nothing to talk to.

`gcal` SHALL additionally pull the time zone database its projection mints definitions from, so a build without that backend pays neither the dependency nor its embedded data.
