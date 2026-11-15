---
cairn: delta
change: series-exceptions-follow-the-listed-range
---

# Delta

## MODIFIED Requirements

### Requirement: msgraph backend
msgraph SHALL adapt io-msgraph's calendars and events. A calendar is a Graph calendar, its colour the read-only `hexColor`, and an item a lone event or a series master, keyed by the event id Graph returned, its `changeKey` as etag, its iCalendar contents synthesized by io-msgraph's `ical` projection with the exceptions of a series folded in. The exceptions SHALL be read from the series' instances within the range the command lists, a day wider either side for the zones, or within five years either side of today when it lists none, cut by io-msgraph's `instance_windows` into windows Graph accepts: no `instances` request SHALL span five years or more. A window Graph refuses SHALL make its event an item error, never fail the listing. An update SHALL read its base over the range a read of the item does. Graph keeps no calendar description and names colours from a palette, so msgraph SHALL declare `calendar.description` and `calendar.color` `none`, a create or update naming either refused before connecting.

#### Scenario: An open-ended series older than five years
- GIVEN a Microsoft 365 calendar holding a weekly series started six years ago with no end, one occurrence this week moved by a day
- WHEN `event list` lists this week
- THEN it succeeds, and lists the moved occurrence on its new day
