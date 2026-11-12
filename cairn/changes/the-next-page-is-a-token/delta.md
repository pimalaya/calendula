---
cairn: delta
change: the-next-page-is-a-token
---

# Delta

## ADDED Requirements

### Requirement: A page names the next one
(commands, after `Text output is plain text`) `event list` and `item list` SHALL print `nextPage` beside their rows: the number of the next page, as a string, while one follows, `null` otherwise, exact on every backend as the footer's `Next page:` is. They SHALL take it back through pimalaya-cli's `--page-token`, which wins over `--page`, refusing a token that does not read as a page number before the backend is reached, `0` reading as 1 as `--page 0` does. An unpaged listing (`-s 0`, or `event list` over a window with no `-s`) SHALL ignore the token as it ignores `--page`. `event find`, which reads the calendar whole, SHALL print `nextPage` as `null` and takes no token.

#### Scenario: A listing resumed from its token
- GIVEN the vdir account `perso` whose calendar `personal` holds the events `a`, `b` and `c`
- WHEN `calendula --json event list -s 2` runs, then `calendula --json event list -s 2 --page-token 2`
- THEN the first prints `a` and `b` and `"nextPage":"2"`, the second `c` and `"nextPage":null`
- AND `calendula event list -s 2 -p 1 --page-token 2` prints `c`, the token winning, and `--page-token x2` fails naming `x2`

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
