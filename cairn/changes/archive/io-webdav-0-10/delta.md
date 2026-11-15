---
cairn: delta
change: io-webdav-0-10
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: CalDAV backend
CalDAV SHALL adapt io-webdav's RFC 4791 surface over a connected client whose calendar home-set is already resolved.

The client SHALL connect from `caldav.server` or `caldav.home` alone ("CalDAV locates its home-set two ways" in config), and SHALL NOT resolve a domain through DNS (RFC 6764 SRV records), so the backend links no resolver.

Calendar and item ids SHALL be the last segments of the hrefs the server serves, read as io-webdav reads a listed segment: percent-decoded when the decoding encodes back to that very segment, as served otherwise (a `%2F`, an escape of a byte sent raw such as `%40`, lowercase hex). A calendar outside the home set SHALL be named by its path, as served. io-webdav neither appends nor strips a file extension, so an id a listing showed addresses the same resource on every verb, as does the served segment itself. A calendar a command names SHALL be found among the listed ones by the path its id addresses under the home set (`rfc4918::collection_path`), never by comparing the strings, so either spelling finds it and a segment kept as served is never found by its decoding.

A create SHALL propose a resource name derived from the UID of the item's first VEVENT, VTODO or VJOURNAL when that UID is URL-safe, falling back to a content digest, and SHALL keep the id the server reports in its `Location` header when it names the resource itself, else the last segment of the URL io-webdav sent, both read as a listed segment is, which is the form a listing returns.

A listing the server answers with anything but one whole multistatus (an HTML page, a cut document, an empty body) SHALL fail, never read as a shorter listing, and a delete answered by a multistatus naming failed members SHALL fail.

A listing SHALL name, in its `errors`, each row the server answered without the item's data and a listing the server truncated ("Each backend names the items it could not read"), on the first page only: the whole calendar is read before the page is cut, and such a row holds no place on a page.

A calendar's colour SHALL be written as Apple's `calendar-color` (`http://apple.com/ns/ical/`), by a create and an update alike, and read from it, the inf-it one when it is absent.

`update_calendar` SHALL write only the properties the patch names, setting a given one and removing (`DAV:remove`) a cleared one, through io-webdav's calendar update (`CaldavCalendarPatch`), which sends a request again on a fresh connection when the server closed the one it was sent on: a property the server refuses, or answers nothing of (RFC 4918 §9.2.1), fails the update. A colour SHALL go first, in a PROPPATCH of its own, so a server keeping the rest of a PROPPATCH while it ignores a colour leaves the calendar as it was: the update fails ``Write the colour of calendar `<id>`, nothing else written``. A calendar no listed id addresses SHALL fail `calendar-not-found` before anything is written. `create_calendar` given a colour SHALL read the calendars back and log a warning naming the colour when the server kept none.

#### Scenario: A colour the server ignores
- GIVEN a CalDAV server ignoring `calendar-color` in a PROPPATCH
- WHEN `calendula calendar update cal -n New -d Text -C '#123456'` runs
- THEN it exits 1 naming `calendar-color`, after one PROPPATCH holding the colour alone, and the name and the description are not written

#### Scenario: A calendar named with a space
- GIVEN a CalDAV calendar the server serves at `<home>/My%20Cal/`
- WHEN `calendula calendar list` runs, then `calendula calendar update "My Cal" -n New` and `calendula calendar update My%20Cal -n New`
- THEN the calendar lists as `My Cal`, and each update sends its PROPPATCH to `<home>/My%20Cal/`

### Requirement: A CalDAV id addresses its own resource
A calendar or item id a CalDAV command names SHALL be refused, before the account's secret is resolved or anything connects, when io-webdav refuses it (`rfc4918::check_collection_id`, `check_resource_id`: empty, or holding an empty, `.` or `..` segment once split on `/` and `\` and percent-decoded, a calendar id's trailing slashes and an absolute id's leading one aside), such an id addressing the home set, its parent or the server root: a calendar id coded `calendar-not-found`, ``Calendar `<id>` not found``, an item id coded `item-not-found`, ``Item `<id>` not found``, io-webdav's reason in `sources`, as vdir refuses an id that is not one directory name. Every unified command naming one SHALL refuse it so (`calendar update` and `delete`, `event list`, `agenda`, `search`, `read`, `add`, `update` and `delete`, `item list`, `read`, `add`, `update` and `delete`), `event delete` in its own sentence for an item id (commands, "An event is named by its id alone"), and every `caldav` command naming one (`propfind`, `proppatch`, `report`, `get`, `put` and `delete`). The id `calendar create` or `caldav mkcalendar` gives a new calendar, its URL segment, SHALL be refused the same way, uncoded as vdir refuses a new id: ``Calendar id `<id>` names no CalDAV collection``. A name given alone takes a minted UUID.

io-webdav refuses such an id itself before sending anything, but as an I/O error no code reads, and only once the account is open.

#### Scenario: An id climbing out of the home set
- GIVEN a CalDAV account whose password command would leave a file behind
- WHEN `calendula --json calendar delete ..` or `calendula --json item read -k cal ..` runs
- THEN it exits 1 coded `calendar-not-found` (``Calendar `..` not found``) or `item-not-found` (``Item `..` not found``), io-webdav's reason in `sources`, the password command never ran and nothing connected

## REMOVED Requirements

None.
