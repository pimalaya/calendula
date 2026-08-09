---
cairn: change
id: gcal-cli
status: landed
created: 2026-08-09
---

# Google Calendar protocol-specific family

## Why

Every other backend has a family exposing what only it has: `caldav` discovers endpoints and lists collections with their ctag, sync token and accepted component kinds; `vdir` renames a collection; `pimdir` reports how much of a store is downloaded. gcal landed with a backend and no family, so the whole half of the Calendar API that iCalendar cannot express stayed unreachable, and the shared surface was the only way in.

That half is not marginal. Free/busy is how a calendar answers "when are you available", and it is a query, not a resource: no component family can carry it. ACL is how a Google calendar is shared, which is the difference between a personal calendar and a team one. Quick add is the fastest path from a sentence to an event. Instances expand a recurring series, which the shared API deliberately does not do (it returns the series, since that is what round-trips). None of these belongs in a least-common-denominator surface, and all of them are one io-gcal call away.

## What

A `gcal` family in the shape `caldav` already has: a connected client built from the account's `[gcal]` block, and one subcommand group per Calendar API resource the shared surface cannot reach.

- **`calendars`**: the richer listing, as `caldav list` is to `calendar list`. A Google calendar list entry carries the access role the user has on it, whether it is the primary one, its time zone and its default reminders, none of which fit the shared `Calendar`.
- **`acl`** (`list`, `create`, `update`, `delete`): the sharing rules, each granting a scope (a user, a group, a domain, or everyone) a role.
- **`free-busy`**: the availability of one or more calendars over a window, the query no component family can model.
- **`instances`**: the occurrences a recurring event expands to, which the shared listing does not return.
- **`move`**: moving an event to another calendar, which the shared API would have to emulate as a create plus a delete, losing the id.
- **`quick-add`**: an event from a sentence, parsed server-side.
- **`colors`**: the two palettes, so the colour ids the API talks in can be read.
- **`settings`**: the user's own settings, the time zone and week start a renderer would want.

## Scope

Out, with reasons rather than by omission:

- **`channels` / `watch`**: a push channel delivers to an HTTPS endpoint the caller must host. A CLI has none, and standing one up is the push platform's job, not this binary's.
- **`calendars.transferOwnership`**: an irreversible administrative act on a Workspace domain, and one a `--yes`-less CLI should not make easy.
- **`calendars.clear`**: empties the primary calendar. `calendar delete` covers the secondary case, and the primary one has no undo.

These are declined on purpose, and the spec records the reasons so a future reader does not read them as gaps.
