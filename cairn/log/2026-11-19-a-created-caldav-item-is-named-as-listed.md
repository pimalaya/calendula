---
cairn: log
change: a-created-caldav-item-is-named-as-listed
landed: 2026-11-19
---

# A created CalDAV item is named as listed

Landed without a proposal: a defect from the iCloud live run of 2026-11-19, where `item add` printed `live-cal9-1@pimalaya.test.ics` and `item list` and `event list` printed the same item as `live-cal9-1%40pimalaya.test.ics`, the spelling iCloud serves; `item read` took both.

`new_resource_name` (src/caldav/backend.rs) mints the UID through io-webdav's `rfc4918::encode_id`, so the PUT goes to `e1%40example.org.ics` and the id io-webdav returns, read from the URL sent as a listed segment is, keeps the `%40`: io-webdav's `segment_id` keeps `%40` as served, since `@` encodes back raw. The id a create prints is the one a listing gives, on `item add`, `event add` and every unified create. `is_safe_segment` is unchanged: `@` is the one byte it lets through that `encode_id` escapes. An update prints the id it was given, either spelling addressing the item.

Test `a_created_item_is_named_as_the_listing_names_it` (src/caldav/backend.rs), a scripted PUT then REPORT, which fails before the change (`e1@example.org.ics` against `e1%40example.org.ics`); `a_resource_name_comes_from_the_uid_when_it_is_url_safe` now expects `event-1%40example.org`.

Moved: backends.md ("CalDAV backend", the create's resource name). CHANGELOG (Fixed).
