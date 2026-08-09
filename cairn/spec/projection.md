---
cairn: spec
capability: projection
status: current
---

# iCalendar projection

CalDAV, vdir and pimdir speak iCalendar natively. Google Calendar does not: for that backend the shared `CalendarItem.contents` is an iCalendar document of record calendula synthesizes from the Google event and re-projects on the way back ([gcal](../../src/gcal/project.rs)).

The policy is ported from cardamum's spec of the same name, so both products treat provider quirks identically.

### Requirement: Native backends do not project
CalDAV, vdir and pimdir speak iCalendar natively and SHALL store and return the bytes verbatim. Only a backend with no native iCalendar representation projects.

### Requirement: gcal synthesizes the document of record
The gcal backend SHALL synthesize the shared `CalendarItem.contents` from the Google event on read and re-project it on write. The shared type's documentation SHALL name the exception, so a reader of `CalendarItem` learns that one backend's bytes are generated rather than stored.

The synthesized document SHALL be a VCALENDAR wrapping exactly one VEVENT, carrying calendula's own PRODID.

### Requirement: Only well-slotted fields are managed
A Google event field SHALL be *managed*, read into the iCalendar on the way out and written back on the way in, only when it has a well-defined iCalendar slot: `iCalUID`, `summary`, `description`, `location`, `start`, `end`, `recurrence`, `status`, `transparency`, `visibility`, `attendees`, `organizer`, `sequence` and the reminder overrides. A managed field is authoritative in both directions, so clearing the property clears the field on the next update.

`created` and `updated` are managed on the way out only: they project onto CREATED and LAST-MODIFIED (and stand in for the mandatory DTSTAMP, which Google has no field for), but Google stamps them itself, so an incoming CREATED, LAST-MODIFIED or DTSTAMP SHALL be consumed rather than stashed or written.

### Requirement: Provider-only fields are left alone
A Google field with no iCalendar equivalent (`colorId`, `eventLabelId`, `eventType`, `anyoneCanAddSelf`, `guestsCanInviteOthers`, `guestsCanModify`, `guestsCanSeeOtherGuests`, `birthdayProperties`, `focusTimeProperties`, `outOfOfficeProperties`, `workingLocationProperties`, `gadget`, `source`, `attachments`) SHALL stay out of every write and survive an update untouched.

Since Google's event write replaces the whole resource, an update SHALL read the current server event first and merge the projection onto it, so the provider-only fields carry over from the server copy while the managed ones stay authoritative.

### Requirement: Provider-scoped fields are minted read-only
A Google field that means nothing outside the account (`htmlLink`, `hangoutLink`, `conferenceData`) SHALL be minted as a read-only `X-GOOGLE-*` property on read and consumed on write, the server value staying authoritative. A minted property is neither managed nor part of the stash remainder.

### Requirement: The remainder is stashed verbatim
Every iCalendar line the projection neither manages nor mints SHALL be stashed verbatim in `extendedProperties.private` and spliced back on read, so a property no Google field models survives a round-trip instead of being dropped on the next write. The remainder SHALL be recomputed from the incoming iCalendar on every write, so the stash never drifts.

The VEVENT remainder and the VCALENDAR-level remainder SHALL ride separate key families, since a calendar-level property cannot legally be spliced inside a VEVENT. A VTIMEZONE the event's TZID references belongs to the calendar-level family, so dropping the stash would never leave a TZID reference dangling.

An extended property another client owns SHALL survive a stash rewrite: only the chunks under calendula's own key prefixes are replaced.

### Requirement: The stash is chunked to the provider limit
Google caps an extended property value at 1024 characters, so the stash SHALL be split across numbered keys rather than written as one value. A single line too long to fit a chunk SHALL stay in the local document only, never sent, so an oversized property cannot fail the whole write.

### Requirement: Boundaries carry a zone or nothing
An all-day boundary SHALL project as a `VALUE=DATE` property and back; a timed one anchored in a named zone SHALL carry a TZID parameter and Google's `timeZone`; a timed one in UTC SHALL carry the `Z` suffix.

A floating DTSTART (no `Z` suffix and no TZID) has no Google form, since the API needs either a UTC offset or a named zone, so it SHALL be refused by name rather than resolved to a guessed zone. A VEVENT expressing its length as a DURATION rather than a DTEND SHALL likewise be refused by name, since Google requires an end.

A recurring event whose boundaries are UTC SHALL be given the `UTC` time zone explicitly, since Google expands a recurrence in the zone of its start and a UTC offset names none. That zone SHALL project back as the `Z` suffix rather than as a TZID, so the projection is idempotent: projecting a document a second time reproduces it byte for byte.

### Requirement: An instant outranks a display zone
Google returns a boundary as an absolute instant plus the calendar's display zone, a pair no iCalendar boundary expresses. Only the offset SHALL decide the instant: a `Z`-stamped boundary SHALL project as a UTC stamp even when a zone is named alongside it, since reading the literal time as that zone's wall time would shift the event by the zone's offset, and deriving the real wall time would need a time zone database the stack does not carry.

The display zone SHALL therefore be treated as a provider-only field and carried over from the server copy on update, so a zoned recurring series keeps expanding where it did instead of falling back to UTC and drifting by an hour after a daylight-saving change. Only a UTC-stamped boundary SHALL take the server's zone: an offset-less one is wall time in the zone it names, and relabelling it would move the event.

### Requirement: Alarms project only where Google can model them
A VALARM whose action is DISPLAY or EMAIL and whose trigger is a lead time in whole minutes within Google's four-week ceiling SHALL project onto a reminder override, and back. Any other alarm SHALL stay in the stash rather than be flattened into one Google can hold.

A document carrying no VALARM SHALL inherit the calendar's default reminders rather than silence the event, since an empty override list with the defaults turned off means no reminder at all.
