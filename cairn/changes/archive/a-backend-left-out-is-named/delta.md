---
cairn: delta
change: a-backend-left-out-is-named
---

# Delta

## ADDED Requirements

### Requirement: An account names the backends left out of the build
(commands) `account list` SHALL print, per account, `{"name", "default", "backends", "notBuilt"}`: `backends` the backends the account carries a block for and the build serves, `notBuilt` those it carries a block for and the build leaves out, each in the order `auto` tries them. The BACKENDS column SHALL list the first, then each of the second as `<name> (not built)`, comma-separated.

#### Scenario: A gcal block in a build without gcal
- GIVEN a build without `gcal` and the account `test` carrying a `gcal` block alone
- WHEN `calendula --json account list` runs
- THEN `test` reads `"backends": []` and `"notBuilt": ["gcal"]`, and the text row reads `gcal (not built)`

## MODIFIED Requirements

### Requirement: Backend selection order
(backends) `auto` takes the first configured backend, built or not. When that backend is one the build leaves out, its block loaded unread, `auto` SHALL NOT fall through to the next: the command SHALL fail with `backend-not-built` before anything is read or connected, the message naming the backend and the cargo feature bringing it back and, when the account configures a backend the build serves, the first of them and the `--backend` reaching it. A `--backend` value naming a backend the build leaves out SHALL be refused by the command line (exit 2), naming the cargo feature.

#### Scenario: The first backend is left out of the build
- GIVEN a build with `msgraph` and without `gcal`, and an account carrying a `gcal` and a `msgraph` block
- WHEN `calendula --json calendar list` runs
- THEN it exits 1 with the failure object coded `backend-not-built`, naming `--features gcal` and `--backend msgraph`, and nothing is connected

### Requirement: Unknown fields
(config) The block of a backend the build leaves out SHALL load unread, so one file serves every build, and a command reaching it SHALL fail with `backend-not-built`. Its scenario ends: a unified command on that account fails coded `backend-not-built`, naming `--features gcal`.

### Requirement: A JSON error carries a stable code
(commands) The code table gains `backend-not-built`.

### Requirement: account check fails when a backend fails
(commands) Under `auto`, a block of a backend the build leaves out SHALL be a failed verdict of that backend, after those of the built backends, coded `backend-not-built`, its error naming the cargo feature bringing it back, so the command exits 3. A named `--backend` reaches built backends alone.

### Requirement: An account lists what it serves
(commands) When the backend the unified commands resolve to is one the build leaves out, the command SHALL fail coded `backend-not-built` (exit 1), with the message a unified command gives (see "Backend selection order" in backends).

### Requirement: One cargo feature per backend
(packaging) A build with no backend fails `account check` and the unified commands with `backend-not-built` on an account carrying the block of a backend it leaves out, else no backend matching being configured.

### Requirement: Every backend builds alone
(packaging) A build loads the block of a backend it leaves out unread; in the scenario "A block for a compiled-out backend", the account is listed with vdir, and caldav as not built.

## REMOVED Requirements
