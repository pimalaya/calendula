---
cairn: change
id: the-namespaces-follow-the-twin
status: landed
created: 2026-11-15
---

# The namespaces follow the twin

> Line K6 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), finding F23 of its reports/audit.md. cardamum, the twin, is the model: its src/carddav, src/msgraph and src/vdir.

## Why

cardamum's protocol namespaces mirror their protocol: `carddav` is the WebDAV and CardDAV methods (mkcol, propfind, proppatch, report, get, put, delete), `msgraph` is nested by Graph resource, `vdir` has an `item` subtree. calendula's were three shapes of their own: `caldav` was create, delete, discover and list, the Graph backend had no namespace at all, `vdir` stopped at the collections, and `gcal` mixed resource nouns (`calendars`, `acl`, `colors`, `settings`) with bare event methods (`instances`, `move`, `quick-add`). The twins are meant to read alike, and a vendor command keeps the API's resource spelling, plural included (the noun rule: Himalaya's `gmail messages`, cardamum's `gpeople connections`).

## What

Every former command name stays, hidden, so no script breaks: an alias where the command keeps its depth (`caldav list` reads as `caldav propfind`), a hidden command where it moves deeper (`gcal instances` runs `gcal events instances`).

### caldav

| Before | After | cardamum |
|---|---|---|
| `caldav discover` | `caldav discover` | `carddav discover` |
| `caldav list` (`ls`) | `caldav propfind [CALENDAR]`, `list` and `ls` hidden | `carddav propfind [ADDRESSBOOK]` |
| | `caldav proppatch <ID> [-n] [-d] [-C] [--tz]` | `carddav proppatch <ID> [-n] [-d] [-C]` |
| `caldav create <ID>` | `caldav mkcalendar <ID> [-n] [-d] [-C] [--component] [--tz]`, `create` hidden | `carddav mkcol <ID> [-n] [-d] [-C]`, `create` visible |
| | `caldav report query <CALENDAR> [--component] [--from] [--to] [--match PROP VALUE] [--not-match PROP VALUE] [--defined] [--not-defined] [--collation]` | `carddav report query <ADDRESSBOOK> [--match PROP TYPE VALUE] [--not-match ...] [--defined] [--not-defined] [--test] [--collation] [--limit]` |
| | `caldav report multiget <CALENDAR> <ITEM-ID>...` | `carddav report multiget` |
| | `caldav report sync <CALENDAR> [--sync-token] [--fallback]` | `carddav report sync` |
| | `caldav report raw <CALENDAR> <XML> [--depth]` | `carddav report raw` |
| | `caldav get <CALENDAR> <ITEM-ID>` | `carddav get` |
| | `caldav put <CALENDAR> <ITEM-ID> <ICAL> [--if-match \| --if-none-match]` | `carddav put` |
| `caldav delete <ID>` | `caldav delete <CALENDAR> [ITEM-ID] [--if-match]` | `carddav delete` |

The outputs: `propfind` without a calendar prints `{"calendars": [{id, displayName, description, color, components, ctag, syncToken}]}` as `caldav list` did, with one `{"items": [{id, etag}], "truncated"}`; `report query` and `multiget` print `{"items": [{id, etag, contents, encoding}], "truncated", "errors"}`; `report sync` `{changed, vanished, syncToken, truncated}`; `report raw` `{responses: [{href, status, etag, dataBytes}], syncToken}`; `get` `{id, etag, contents, encoding}`; `put` `{id, etag}`; `discover` `{server, principal, calendarHomeSet}`, adding `server` and reading an endpoint it cannot resolve as `null` rather than failing; the writes `{id}`.

### msgraph (new)

| After | cardamum |
|---|---|
| `msgraph calendars list [-s N]` (`calendar` hidden) | `msgraph contact-folders list` |
| | `msgraph contact-folders child-folders`: no counterpart, Graph calendars do not nest |
| `msgraph calendars get <CALENDAR-ID>` | `msgraph contact-folders get` |
| `msgraph calendars create <NAME>` (`add`, `new`) | `msgraph contact-folders create` |
| `msgraph calendars rename <CALENDAR-ID> <NAME>` | `msgraph contact-folders rename` |
| `msgraph calendars delete <CALENDAR-ID>` (`del`, `rm`) | `msgraph contact-folders delete` |
| `msgraph events list [-k CALENDAR-ID] [-s N] [--select] [--filter]` (`event` hidden) | `msgraph contacts list [-f FOLDER-ID] ...` |
| `msgraph events get <EVENT-ID>` | `msgraph contacts get` |
| `msgraph events create [-k CALENDAR-ID] <JSON>` (`add`, `new`) | `msgraph contacts create [-f FOLDER-ID] <JSON>` |
| `msgraph events update <EVENT-ID> <JSON>` | `msgraph contacts update` |
| `msgraph events delete <EVENT-ID>` (`del`, `rm`) | `msgraph contacts delete` |
| `msgraph events delta [-k CALENDAR-ID] --from DATE --to DATE \| --delta-link URL` | `msgraph contacts delta [-f FOLDER-ID] [--select] \| --delta-link URL` |
| `msgraph profile get` | `msgraph profile get` |
| `msgraph request <METHOD> <PATH> [JSON]` | `msgraph request` |

The outputs are Graph's raw objects as cardamum prints its: `{"calendars": [...], "@odata.nextLink"}`, one calendar as Graph sends it, `{"events": [...], "@odata.nextLink"}`, one event, `{"events": [...], "@odata.nextLink", "@odata.deltaLink"}` for a delta page, the user, `{"response"}` for a request, `{id}` for a delete. A request naming one event Graph does not hold is `item-not-found`, one naming a calendar `calendar-not-found`, as the unified commands code them.

### vdir

| Before | After | cardamum |
|---|---|---|
| `vdir list` (`ls`), `create`, `rename`, `delete` | unchanged | `vdir list`, `create`, `rename`, `delete` |
| | `vdir item list <COLLECTION>` (`items`; `ls`) | `vdir item list` |
| | `vdir item get <COLLECTION> <ID>` | `vdir item get` |
| | `vdir item create <COLLECTION> [--kind] <INPUT>` (`new`) | `vdir item create` |
| | `vdir item update <COLLECTION> <ID> <INPUT>` | `vdir item update` |
| | `vdir item delete <COLLECTION> <ID>` (`rm`) | `vdir item delete` |
| | `vdir item copy <SOURCE> <TARGET> <ID>` | `vdir item copy` |
| | `vdir item move <SOURCE> <TARGET> <ID>` | `vdir item move` |

Outputs as cardamum's: `{"items": [{id, kind, size, path}]}`, `{id, kind, contents, encoding}`, `{id}`.

### gcal

| Before | After |
|---|---|
| `gcal instances <EVENT-ID>` (`instance`) | `gcal events instances <EVENT-ID>`, `gcal instances` and `gcal instance` hidden |
| `gcal move <EVENT-ID> <DESTINATION-ID>` | `gcal events move`, `gcal move` hidden |
| `gcal quick-add <TEXT>` | `gcal events quick-add`, `gcal quick-add` hidden |
| `gcal calendars`, `acl`, `free-busy`, `colors`, `settings` | unchanged |

`gcal events` takes `event` as a hidden alias, as `calendars`, `colors` and `settings` take their singular. `acl` and `free-busy` keep their spelling, which is the API's (its `Acl` and `Freebusy` resources).

### Where calendula departs from cardamum

- `caldav report query`: RFC 4791's `text-match` is a substring match with no match type (cardamum's `--match PROP TYPE VALUE` is RFC 6352's), so `--match PROP VALUE`; its prop-filters within a component all apply, so no `--test`; `calendar-query` defines no limit, so no `--limit`. A prop-filter sits in a component filter, so `--component` (default `VEVENT` when a filter or a window needs one) and the window `--from`/`--to`, inclusive days as everywhere in calendula.
- `caldav get`, `report query`, `report multiget` and `vdir item get` print `encoding` beside `contents` and base64 for bytes that are not UTF-8, calendula's rule for an item's bytes (commands, "An item's bytes are text or base64"); cardamum fails on such a body.
- `caldav mkcalendar` takes `--component` and `--tz` (RFC 4791 §5.2.3, §5.2.2), `proppatch` `--tz`; CardDAV has neither property. The `create` alias is hidden, where cardamum shows its `create` beside `mkcol`: the family rule hides a former name, so cardamum is the one to align.
- `caldav propfind` keeps the record text `caldav list` printed (calendula's text rules: a field every item carries prints even when empty); cardamum prints only the fields a book holds.
- `caldav discover` keeps calendula's `Fields` text; cardamum prints bare `key: value` lines.
- `msgraph events` takes the calendar as `-k/--calendar`, calendula's letter for a calendar, where cardamum's contacts take `-f/--folder`; `events delta` takes the window Graph's calendar view delta needs (`--from`, `--to`) where cardamum's takes `--select`.
- `vdir list` keeps its visible `ls`, which cardamum's lacks.
