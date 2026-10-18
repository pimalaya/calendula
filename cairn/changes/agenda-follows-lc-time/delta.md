---
cairn: change
change: agenda-follows-lc-time
---

# Delta

## ADDED Requirements

### Requirement: An agenda names months and weekdays in the time locale
The agenda SHALL draw month and weekday names, and stamp the events under its grid, in the locale `LC_ALL`, then `LC_TIME`, then `LANG` resolves to, as cal does. A locale that is unset, `C`, `POSIX` or unknown SHALL fall back to the English names of the POSIX locale. A month argument SHALL be read in English or in the language of that locale.

The grid is a port of cal, so the two drawing the same month in different languages on one system would disagree.

#### Scenario: A Spanish system
- GIVEN `LANG=es_ES.UTF-8` and no `LC_ALL` or `LC_TIME`
- WHEN `calendula event agenda -m` draws October 2026
- THEN the header reads `octubre 2026`, the weekdays `lu ma mi ju vi sá do`, and an event on the 19th is stamped `oct 19`

#### Scenario: The POSIX locale
- GIVEN `LANG=C`
- WHEN `calendula event agenda -m` draws October 2026
- THEN the header reads `October 2026` and the weekdays `Mo Tu We Th Fr Sa Su`
