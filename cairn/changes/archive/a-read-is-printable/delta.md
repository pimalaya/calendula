---
cairn: delta
change: a-read-is-printable
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: Text output is plain text
(commands; its first paragraph, and one scenario added after `An event list`.) A text output SHALL be plain text: no border, no colour, no terminal escape, nothing sized to or truncated for the terminal, and it SHALL end with one newline; no command takes a width. Every string a server, a store or a file gave (an id, a name, a summary, a description, an error a record carries, the zone and the days the lines under `pimdir search` and `pimdir count` name) SHALL have its control and bidi characters replaced before it prints as text (pimalaya-cli `sanitize::text`); the JSON output SHALL carry it unchanged. The iCalendar `event read` and `item read` print SHALL keep its layout, CRLF read as LF, and have every other control character but tab and line feed (C0, DEL, C1) and every bidi embedding, override and isolate replaced with U+FFFD; their JSON `contents` SHALL carry the stored text unchanged. The iCalendar a `build` prints, or writes to the file `-o` names, SHALL be the bytes it built.

#### Scenario: A read of a body carrying controls
- GIVEN a vdir event stored with CRLF line ends, `ESC ]52;c;AAAA BEL` in its summary, U+202E in its location and a description folded on a tab
- WHEN `calendula event read` or `calendula item read` prints it
- THEN its lines end in LF, the ESC, the BEL and the U+202E print as U+FFFD, and the folded line keeps its tab
- AND under `--json` its `contents` is the stored text, CRLF and controls included

## REMOVED Requirements

None.
