---
cairn: change
id: a-read-is-printable
status: landed
created: 2026-11-12
---

# A read is printable

## Why

The cross-check of the-family-standards with cardamum and Himalaya on 2026-11-12 found `event read` and `item read` printing the stored iCalendar raw in text mode, as the spec asked ("The raw iCalendar a command is asked for ... SHALL print as it is stored"). A server, a sync or a file writes those bytes, so an event whose summary carries an OSC 52 clipboard write or a bidi override drives the terminal of whoever reads it. Cardamum fixed the same in `card read` (every-text-output-is-printable, eb79dff) and Himalaya's `message read` defuses its bodies the same way (`sanitize_body`, himalaya/src/shared/message/read.rs); pimalaya-cli's `sanitize::text` is for one-line fields, replacing tabs and line feeds too.

## What

- `event read` and `item read` print the iCalendar with CRLF read as LF and every other control character but tab and line feed (C0, DEL, C1) and the bidi embeddings, overrides and isolates replaced with U+FFFD, as Himalaya's `sanitize_body` and cardamum's `card read` do; one `printable` in src/shared/item/read.rs serves both. `--json` keeps the stored text.
- A `build` stays byte-exact, on stdout as in the file `-o` names: its iCalendar is the one the caller gave or composed, printed to be piped into a write.
- The sweep for other text outputs printing a server or store string raw: the descriptions (`calendar list`, `caldav list`, `gcal calendars`), `gcal settings`, `pimdir status`, `pimdir search`, `account check` and every receipt already go through `sanitize::text`. Two did not: the line naming the index zone under `pimdir search` and `pimdir count`, and the days of the coverage line under them, both read from the store; they do now.
- The help of both reads, the spec (commands), CHANGELOG, log.

## Out

The failure a failed command prints (pimalaya-cli `ErrorReport`) carries its message and sources as written, a server's error text included; defusing it is pimalaya-cli's. A shared `sanitize` for multi-line text in pimalaya-cli would replace the three copies (Himalaya, cardamum, calendula).
