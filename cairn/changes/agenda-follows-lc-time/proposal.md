---
cairn: change
id: agenda-follows-lc-time
status: landed
created: 2026-10-18
---

# The agenda names months and weekdays in the time locale

## Why

`event agenda` follows cal(1), but its month and weekday names are hardcoded in English. On a system set to `es_ES.UTF-8`, cal draws `octubre 2026 / lu ma mi ju vi sá do` while the agenda draws `October 2026 / Mo Tu We Th Fr Sa Su`, and stamps the events under the grid `Oct 19`. The agenda is the one that is out of step with the tool it ports.

## What

- The locale is picked as setlocale(3) does for `LC_TIME`: `LC_ALL`, then `LC_TIME`, then `LANG`, the first one set and not empty winning; its codeset and modifier are dropped (`es_ES.UTF-8@euro` reads as `es_ES`).
- The names come from chrono's locale tables (the `unstable-locales` feature, which pulls `pure-rust-locales`): `%B` for the months, `%a` cut to two characters for the weekdays, as cal does (`mié` draws `mi`).
- `C`, `POSIX`, no variable set or a locale chrono does not know fall back to POSIX, whose names are the English ones the agenda draws today: such a system sees no change.
- The event stamps under the grid use the same locale (`oct 19, 20:50`); their pattern is unchanged.
- A month argument is read in English or in the language of the locale (`event agenda enero 2027`).
- Headers are centred by characters rather than bytes, so a name with an accent (`sá`, `août`) stays aligned.
- The `--json` payload is unchanged.
