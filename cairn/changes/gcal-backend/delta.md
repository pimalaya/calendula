---
cairn: change
change: gcal-backend
---

# Delta

## ADDED Requirements

### Requirement: Google Calendar backend
gcal SHALL adapt io-gcal's Calendar API v3 client. A calendar is a calendar list entry and an item is an event of it. Item ids SHALL be the event ids the API returned, verbatim. `create_item` SHALL use `events.import` when the projected event carries an iCalendar UID, so the UID survives, and `events.insert` otherwise. `update_item` SHALL honour `if_match` as the API's `If-Match` header.

`create_calendar` SHALL insert a secondary calendar. Google mints the id of a calendar it creates, so the requested id SHALL be reported rather than honoured. `update_calendar` SHALL patch the calendar for name and description and the calendar list entry for colour, since Google splits those across the two resources; a Google calendar always carries a colour, so clearing one SHALL be refused by name.

### Requirement: gcal pushes the time range down
A [`CalendarTimeRange`](#requirement-time-range-filtering) SHALL reach gcal as the `timeMin` and `timeMax` parameters of `events.list`, so the server does the narrowing. The bounds SHALL be converted from their iCalendar UTC spelling to the RFC 3339 form the API takes.

### Requirement: gcal walks its own pagination
`list_items` SHALL follow `nextPageToken` until the requested window is covered, then apply the shared 1-indexed windowing. A page the caller never reaches SHALL NOT be fetched.

### Requirement: gcal refuses what Google cannot model
An item whose iCalendar carries no VEVENT SHALL be refused by component name rather than emulated, since Google models neither VTODO nor VJOURNAL.

## MODIFIED Requirements

### Requirement: Backend selection order
The shared commands SHALL target the backend the global `--backend` flag selects. Its default, `auto`, SHALL take the first configured-and-compiled backend in the order vdir, pimdir, CalDAV, gcal, preferring a local read to a network round-trip and a protocol-standard server to a vendor API. A named value SHALL pin the command to that backend and bail when the account carries no matching configuration block. The protocol-specific commands SHALL ignore the flag.

### Requirement: One cargo feature per backend
Each backend SHALL sit behind its own cargo feature (`caldav`, `gcal`, `vdir`, `pimdir`), pulling its io-* crates through `dep:`. A build SHALL compile at least one of them; a build with none is not a supported configuration, since the product would have nothing to talk to.

### Requirement: Backend blocks
An account SHALL carry an optional block per compiled backend: `vdir` with a `home-dir`, `pimdir` with a `root` and an optional `source`, `caldav` with its endpoint, TLS and authentication, `gcal` with a `token` and TLS. An account MAY carry several; which one a shared command uses is the backend capability's business.

### Requirement: Secrets are read, never written
A CalDAV password, a CalDAV token or a gcal token SHALL be a secret read from the configuration itself or from the standard output of a command. calendula SHALL NOT write a secret anywhere: an OAuth 2.0 token broker is a command like any other, and a missing value surfaces when the account is tested. Google expires an access token within the hour, so a token broker is the practical answer there rather than a stored value.

### Requirement: Protocol-specific families
`caldav`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`. vdir covers its collection verbs including `rename`, which the shared API has no home for. pimdir covers `status`, reporting the source writes are attributed to, every source the store has been synced as, and how much of each calendar is downloaded.

A backend MAY exist without a family: gcal has none yet, so what only the Calendar API offers (free/busy, colours, ACL, quick add, push channels) is not reachable. A backend without a family SHALL NOT push those concepts into the shared API instead.

### Requirement: Projections never rewrite bytes
Projecting a VEVENT out of an item SHALL be read-only and lossy by design: calendula returns what the backend holds. An item whose bytes do not parse SHALL project no event rather than fail the listing, so one malformed resource cannot hide a whole calendar.

"What the backend holds" is verbatim for the backends that store iCalendar, and synthesized for the one that does not; the projection capability governs the difference, and no command layer SHALL rewrite bytes of its own.

## ADDED Requirements

The requirements below open a new `projection` capability, ported from cardamum's spec of the same name so both products treat provider quirks identically. They fold into `cairn/spec/projection.md`; the requirements above fold into `cairn/spec/backends.md`, `cairn/spec/commands.md`, `cairn/spec/config.md`, `cairn/spec/packaging.md` and `cairn/spec/wizard.md`, each naming its file where it is not the backends one.

### Requirement: gcal has nothing to discover
The Calendar API lives at one fixed base URL, so the `gcal` block SHALL name no server and offer no discovery route: a bearer token and a TLS profile are the whole block. This folds into `cairn/spec/config.md`.

### Requirement: Credentials nest under `auth`
Every backend that authenticates SHALL carry its credentials under an `auth` sub-block, so the same concept is spelled the same way across the Pimalaya CLIs: `caldav.auth.basic.password`, `gcal.auth.token`, as cardamum spells `people.auth.token` and `msgraph.auth.token`. A backend accepting exactly one kind SHALL still nest, as a struct rather than an enumeration of kinds, so gaining a second kind later is not a breaking change to the first. This folds into `cairn/spec/config.md`.

### Requirement: The wizard covers the discoverable backends only
The wizard SHALL configure CalDAV, vdir and pimdir. gcal is out: it needs no discovery and its token broker story is the user's to settle, so a Google account is written by hand from the sample configuration. A build carrying none of the three wizard-capable backends SHALL compile without the wizard and say so by name when bare `calendula` runs, rather than offering an empty flow. This folds into `cairn/spec/wizard.md`.

### Requirement: Native backends do not project
CalDAV, vdir and pimdir speak iCalendar natively and SHALL store and return the bytes verbatim. Only a backend with no native iCalendar representation projects.

### Requirement: gcal synthesizes the document of record
The gcal backend SHALL synthesize the shared `CalendarItem.contents` from the Google event on read and re-project it on write. The shared type's documentation SHALL name the exception, so a reader of `CalendarItem` learns that one backend's bytes are generated rather than stored.

### Requirement: Only well-slotted fields are managed
A Google event field SHALL be *managed*, read into the iCalendar on the way out and written back on the way in, only when it has a well-defined iCalendar slot: `iCalUID`, `summary`, `description`, `location`, `start`, `end`, `recurrence`, `status`, `transparency`, `visibility`, `attendees`, `organizer`, `sequence` and the reminder overrides. A managed field is authoritative in both directions, so clearing the property clears the field on the next update.

`created` and `updated` are managed on the way out only: they project onto CREATED and LAST-MODIFIED (and stand in for the mandatory DTSTAMP), but Google stamps them itself, so an incoming CREATED, LAST-MODIFIED or DTSTAMP SHALL be consumed rather than stashed or written.

### Requirement: Provider-only fields are left alone
A Google field with no iCalendar equivalent (`colorId`, `eventLabelId`, `eventType`, `anyoneCanAddSelf`, `guestsCanInviteOthers`, `guestsCanModify`, `guestsCanSeeOtherGuests`, `birthdayProperties`, `focusTimeProperties`, `outOfOfficeProperties`, `workingLocationProperties`, `gadget`, `source`, `attachments`) SHALL stay out of every write and survive an update untouched.

Since Google's event write replaces the whole resource, an update SHALL read the current server event first and merge the projection onto it, so the provider-only fields carry over from the server copy while the managed ones stay authoritative.

### Requirement: Provider-scoped fields are minted read-only
A Google field that means nothing outside the account (`htmlLink`, `hangoutLink`, `conferenceData`) SHALL be minted as a read-only `X-GOOGLE-*` property on read and consumed on write, the server value staying authoritative. A minted property is neither managed nor part of the stash remainder.

### Requirement: The remainder is stashed verbatim
Every iCalendar line the projection neither manages nor mints SHALL be stashed verbatim in `extendedProperties.private` and spliced back on read, so a property no Google field models survives a round-trip instead of being dropped on the next write. The remainder SHALL be recomputed from the incoming iCalendar on every write, so the stash never drifts.

The VEVENT remainder and the VCALENDAR-level remainder SHALL ride separate key families, since a calendar-level property cannot legally be spliced inside a VEVENT; a VTIMEZONE the event's TZID references belongs to the calendar-level family. An extended property another client owns SHALL survive a stash rewrite.

### Requirement: The stash is chunked to the provider limit
Google caps an extended property value at 1024 characters, so the stash SHALL be split across numbered keys rather than written as one value. A single line too long to fit a chunk SHALL stay in the local document only, never sent, so an oversized property cannot fail the whole write.

### Requirement: A create reports the identifier it was given
`create_calendar` and `create_item` SHALL return the identifier the backend actually assigned, and the command SHALL report that one. It is the requested id on every backend that lets a client name a collection, and a server-minted one where it does not, so a create never names a resource that does not exist. This folds into `cairn/spec/backends.md`.

### Requirement: An instant outranks a display zone
Google returns a boundary as an absolute instant plus the calendar's display zone, a pair no iCalendar boundary expresses. Only the offset SHALL decide the instant: a `Z`-stamped boundary SHALL project as a UTC stamp even when a zone is named alongside it. The display zone SHALL be treated as a provider-only field and carried over from the server copy on update, so a zoned recurring series keeps expanding where it did instead of drifting by an hour after a daylight-saving change. Only a UTC-stamped boundary SHALL take the server's zone.

### Requirement: Boundaries carry a zone or nothing
An all-day boundary SHALL project as a `VALUE=DATE` property and back; a timed one anchored in a named zone SHALL carry a TZID parameter and Google's `timeZone`; a timed one in UTC SHALL carry the `Z` suffix. A floating DTSTART, and a VEVENT expressing its length as a DURATION rather than a DTEND, SHALL be refused by name rather than resolved to a guess. A recurring event whose boundaries are UTC SHALL be given the `UTC` time zone explicitly, since Google expands a recurrence in the zone of its start.

### Requirement: Alarms project only where Google can model them
A VALARM whose action is DISPLAY or EMAIL and whose trigger is a lead time in whole minutes within Google's four-week ceiling SHALL project onto a reminder override, and back. Any other alarm SHALL stay in the stash. A document carrying no VALARM SHALL inherit the calendar's default reminders rather than silence the event.
