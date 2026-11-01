---
cairn: change
change: agenda-reads-a-lone-month
---

# Delta

## ADDED Requirements

### Requirement: An agenda reads its date as cal does
The agenda SHALL read one argument that is a number as a year and drawn whole, and one argument that is not a number as a month name, drawing that month of the current year. A day that is not a number of the month, a month that matches no name in English or in the time locale, a month number outside 1-12, or a year that is not an integer SHALL be refused, naming the value, never replaced by the 1st, by January or by the current year.

The grid is a port of cal, and a typo silently drawing another month reads as an empty one.

#### Scenario: A lone month name
- GIVEN `LANG=es_ES.UTF-8` in 2026
- WHEN `calendula event agenda noviembre` runs
- THEN it draws November 2026 alone, today highlighted when it falls in November, as `cal noviembre` does

#### Scenario: An unknown month name
- GIVEN `LANG=es_ES.UTF-8`
- WHEN `calendula event agenda novembre 2026` runs
- THEN it fails with `Unknown month name: novembre` and draws nothing
