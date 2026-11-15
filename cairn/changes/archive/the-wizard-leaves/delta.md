---
cairn: delta
change: the-wizard-leaves
---

# Delta

## ADDED Requirements

### Requirement: A missing configuration is an error
(config) When no configuration is found, a command needing one SHALL fail naming the path it looked at (the first `-c` or `CALENDULA_CONFIG` path, else the default location) and pointing at the documented sample, and a bare `calendula` SHALL print the help. calendula SHALL NOT prompt to create a configuration.

#### Scenario: A mistyped path
- GIVEN no file at `~/.config/calendula/confg.toml`
- WHEN `calendula -c ~/.config/calendula/confg.toml calendar list` runs
- THEN it fails with `No configuration found at <that path>, write one by hand from the sample: https://github.com/pimalaya/calendula/blob/master/config.sample.toml`

### Requirement: The configuration is never written
(config) calendula SHALL NOT write a configuration file: the user writes it, from the documented sample.

## MODIFIED Requirements

### Requirement: Protocol-specific families
(commands) `caldav`, `gcal`, `pimdir` and `vdir` SHALL each expose what only that backend has, gated behind its own cargo feature. CalDAV covers `discover`, its own calendar listing (carrying the ctag, the sync token and the accepted component kinds), `create` and `delete`.

gcal covers the half of the Calendar API iCalendar cannot express: sharing, availability, recurrence expansion, server-side parsing and the palettes. vdir covers its collection verbs including `rename`, which the unified API has no home for.

pimdir covers `status`, reporting the account being read, every account the store groups collections under, how much of each calendar is downloaded, how many creations are queued for the next sync, and every action the store's owner parked in those calendars, by its queue row (ROW), its calendar, its kind and the error that parked it; `reply` and `cancel`, the scheduling intents only its sync engine can perform; and `search` and `count`, the store's search index queried in the pimdir SEARCH language.

A backend MAY exist without a family where its protocol adds nothing the unified surface lacks; a family SHALL NOT push protocol-specific concepts into the unified API instead.

### Requirement: One cargo feature per backend
(packaging) Each backend SHALL sit behind its own cargo feature (`caldav`, `gcal`, `msgraph`, `vdir`, `pimdir`), pulling its io-* crates through `dep:`. A build SHALL compile at least one of them; a build with none is not a supported configuration, since the product would have nothing to talk to.

`caldav` SHALL pull io-pim-discovery, with its RFC 6764 resolution alone, for the `discover` route, so a build without that backend links no discovery code.

`gcal` SHALL additionally pull the time zone database its projection mints definitions from, so a build without that backend pays neither the dependency nor its embedded data.

## REMOVED Requirements

### Requirement: A defaulted scalar is not written back
(config) Nothing generates an account any more.

The wizard capability leaves whole, spec/wizard.md with it: calendula no longer generates a configuration.

### Requirement: The wizard runs when asked, or when nothing is configured
### Requirement: Input orients the flow
### Requirement: A typed server URL is configured as given
### Requirement: Discovery is time-bounded
### Requirement: Each domain gets its best option, listed in a review
### Requirement: The user sees where a password goes
### Requirement: A password is typed once, tested, then stored
### Requirement: OAuth hands off to Ortie
### Requirement: Account name derived, not prompted
### Requirement: Connection tested before printing
### Requirement: A refusal offers a way on
### Requirement: Saved, appended, or printed
### Requirement: Nothing discovered asks for a server URL
### Requirement: Local backend auto-detected
### Requirement: The wizard covers the discoverable backends only
### Requirement: The welcome introduces the wizard only where it was not asked for
