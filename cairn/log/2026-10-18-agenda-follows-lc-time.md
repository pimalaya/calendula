---
cairn: log
change: agenda-follows-lc-time
landed: 2026-10-18
---

# The agenda names months and weekdays in the time locale

`event agenda` in `src/shared/event/agenda.rs` no longer hardcodes English names: `time_locale` reads `LC_ALL`, `LC_TIME` and `LANG` as setlocale(3) does, and `Names` builds the month and two-character weekday names from chrono's locale tables (`unstable-locales`), falling back to POSIX. The event stamps use the same locale through `EventAgendaOutput::with_locale`, a month argument is read in English or in the locale, and headers are centred by characters rather than bytes.

Tested on locale parsing and fallback, the POSIX and Spanish names, month parsing, an accented header and a Spanish stamp, and against a Google calendar in `es_ES.UTF-8` and `C`.

Capabilities moved: **commands** (an agenda names months and weekdays in the time locale, added).
