---
cairn: change
id: agenda-reads-a-lone-month
status: landed
created: 2026-10-20
---

# The agenda reads a lone month name, and refuses a date it cannot read

## Why

`event agenda` follows cal(1), but a single argument is always read as a year, so `event agenda noviembre` draws the whole current year where `cal noviembre` draws November. util-linux cal documents `cal [timestamp|monthname]`: a lone month name gives that month of the current year, a lone number is still a year.

The same place reads the month and the year with a fallback: `parse_month` returns January when no name matches, and an unreadable year becomes the current one, so a typo (`novembre` on a Spanish system, `2O26`) silently draws another month or year. cal refuses both.

## What

- One argument: a number is a year, as today; anything else is read as a month name, in English or in the time locale, and draws that month of the current year (one month unless `-3`, `-n` or `-y` say otherwise).
- A month that matches no name, or a number outside 1-12, is an error naming the value. A year that is not an integer is an error naming the value.
- The `--json` payload is unchanged.
