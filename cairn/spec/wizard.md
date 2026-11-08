---
cairn: spec
capability: wizard
status: current
---

# Wizard

`calendula configure` (alias `wizard`) runs the interactive wizard. It discovers one account, picks the best way to its calendars and lets the user review it, signs it in and tests it, then saves it, appends it to the configuration already there, or prints it.

Prompts render on stderr, so redirecting stdout into a configuration file works directly.

### Requirement: The wizard runs when asked, or when nothing is configured
The wizard SHALL run from `calendula configure`, and from an offer raised only where nothing can happen without a configuration: a bare `calendula` finding none, and a command needing an account finding none.

A bare `calendula` finding one SHALL print the help instead, and so SHALL one carrying `--account`, which names an account to act on and reads as a half-typed command rather than a first run.

A file that exists but fails to parse counts as a configuration, so the offer never proposes to write over a broken one.

The offer SHALL be skipped in JSON mode and whenever stdin is not a terminal, neither a script nor a JSON consumer being able to answer a prompt: both get the help or the ordinary failure.

A command whose offer was declined or skipped SHALL fail naming the path it looked at and `calendula configure`.

The offer is a hook rather than a gate: the wizard may print the account instead of writing it, so having run it proves nothing, and the caller SHALL look the configuration up again before carrying on.

### Requirement: Input orients the flow
A single prompt SHALL accept an email address (or a bare domain), a `scheme://` server URL, or a local folder path, and nothing SHALL be asked before it. An email or bare domain runs io-pim-discovery's parallel discovery under a spinner; a server URL names the CalDAV context root outright; a folder is a local vdir home or pimdir store.

The wizard SHALL NOT ask which backend to configure, SHALL NOT prompt for an endpoint field it could derive, and SHALL NOT ask an HTTP authentication scheme: a password signs in with HTTP Basic, a token with HTTP Bearer.

The account name SHALL be derived, and suffixed when taken, before any sign-in, the Ortie account and the keyring entry being named after it.

#### Scenario: An address runs discovery
- GIVEN the input `jane@example.org`
- WHEN it is entered
- THEN discovery runs for `example.org`, and the next prompt is the review

### Requirement: A typed server URL is configured as given
An `https`, `caldav` or `caldavs` URL SHALL be taken as the context root, `caldav` and `caldavs` read as `https` since that is how a DAV endpoint is often written down. A plain `http` URL SHALL be refused, naming it, unless its host is this machine (`localhost` or a loopback address), a password never crossing a network unencrypted; any other scheme SHALL be rejected by name.

Nothing being advertised, the wizard SHALL ask how it signs in, a password (selected) or an API token, then go through the review and the sign-in like a discovered route, the login asked since no address was typed.

This is calendula's one deliberate deviation from himalaya's wizard, which takes no server URL at its first prompt. Mail providers are near-universally discoverable; CalDAV servers are not.

Radicale, Baikal and self-hosted Nextcloud routinely publish neither an SRV record nor a `.well-known` redirect, and refusing them would put the servers calendula's users most often run out of reach.

#### Scenario: A plain HTTP server
- GIVEN the input `http://dav.example.org/`
- WHEN it is entered
- THEN the wizard fails naming `http://dav.example.org/` and saying only `https` is accepted, and no password is asked

### Requirement: Discovery is time-bounded
The parallel discovery run SHALL be bounded by a short deadline, so a single unreachable endpoint (a firewalled port, a black-hole host) cannot stall the interactive wizard.

Each mechanism runs independently; any that has not reported by the deadline is abandoned, and only what completed in time is offered.

### Requirement: Each domain gets its best option, listed in a review
Every discovered way to reach the calendars SHALL be one route: a protocol, a server and a sign-in (a password, an API token or OAuth). Two routes with the same protocol, server and sign-in SHALL be one, a server offering OAuth through several grants SHALL be one OAuth route, and a server advertising no sign-in SHALL get a password route. A discovered server that is not `https` SHALL be dropped.

The routes SHALL be ranked, the first being Auto: a provider's own calendar API before CalDAV, servers in the order discovery found them, and within one server OAuth, then an API token, then a password when `ortie` is on the `PATH`, and a password, then an API token, then OAuth when it is not.

Auto SHALL be picked without a question and shown through pimalaya-cli's review as one `Calendar` row in plain words: `<Protocol> at <host>, with your password`, `, with an API token`, `, through your browser` (OAuth through Ortie) or `, with a token from a broker` (OAuth without Ortie), a provider API named without its host. `Continue`, selected, SHALL go to the sign-in. `Change calendar` SHALL list `Auto (<what it is>)`, every route as `<Protocol> · <sign-in>`, the host added only where two would read the same and the whole URL where the host still would, then `Custom server URL`, and SHALL come back to the review. An empty custom server URL SHALL keep the pick.

#### Scenario: A server taking a password and OAuth
- GIVEN discovery offering one CalDAV server at `dav.example.org` with a password and with OAuth, and `ortie` not on the `PATH`
- WHEN the review shows
- THEN its row reads `Calendar  CalDAV at dav.example.org, with your password`, and `Change calendar` lists `Auto (CalDAV · Password)`, `CalDAV · Password`, `CalDAV · OAuth 2.0` and `Custom server URL`

#### Scenario: A provider rule matched
- GIVEN an address a provider rule matches, offering its calendar API and its CalDAV, both over OAuth, and `ortie` on the `PATH`
- WHEN the review shows
- THEN its row reads `Calendar  Google Calendar API, through your browser`

### Requirement: The user sees where a password goes
Before a password or an API token is typed, the wizard SHALL print, through pimalaya-cli's `keyring::prompt_password`, the server it goes to, saying so when its host is outside the typed address's domain.

#### Scenario: Calendars hosted elsewhere
- GIVEN `jane@example.org` whose discovery names `https://dav.example.net/`
- WHEN the password is asked
- THEN stderr reads `Your password goes to https://dav.example.net/, not on example.org`

### Requirement: A password is typed once, tested, then stored
A password or an API token SHALL be asked once, masked, without confirmation, then tested. Once its test passed, the wizard SHALL offer where to keep it through pimalaya-cli's `keyring::prompt_store`, at the keyring entry `calendula-<account name>-caldav`, and SHALL write what reads it back: the keyring's read command, a custom command, or the raw value.

The login SHALL be the advertised username when it is an address, else the typed address, and SHALL be asked (`Login:`) only when neither is known.

#### Scenario: A password stored in pass
- GIVEN the account `example` signing in to CalDAV with a password, and pass installed
- WHEN the test passes and the user picks pass
- THEN the password is stored at `calendula-example-caldav` and the account reads `caldav.auth.basic.password.command = ["pass", "show", "calendula-example-caldav"]`

### Requirement: OAuth hands off to Ortie
calendula runs no OAuth 2.0 grant itself. An OAuth route SHALL be signed in through pimalaya-cli's `ortie::sign_in` with the typed address (or domain), the Ortie account `calendula-<account name>` and the route's service (`caldav`, `gcal` or `msgraphCalendar`), and SHALL write the token command it answers. When `ortie` is not on the `PATH`, the wizard SHALL print one line saying a token broker issues the token, Ortie being one, then offer the keyrings and token brokers of `keyring::prompt_token`.

#### Scenario: A calendar API signed in through Ortie
- GIVEN the account `example` for `jane@example.org`, whose Auto pick is the Google Calendar API, and `ortie` on the `PATH`
- WHEN the user continues the review
- THEN `ortie configure --name calendula-example --service gcal -- jane@example.org` then `ortie auth get -a calendula-example` run, and the account reads `gcal.auth.token.command = ["ortie", "token", "show", "-a", "calendula-example"]`

### Requirement: Account name derived, not prompted
The wizard SHALL NOT prompt for an account name. It derives one from the input (the domain's first label, or the folder name) and uses it as the `[accounts.<name>]` table key; the user renames it by editing that key.

A name the configuration already holds SHALL be suffixed until it is free, two `[accounts.<name>]` tables making the whole document fail to parse and taking the working accounts down with it.

The generated account SHALL claim the default only when no account already does, since two `default = true` would make the account every command picks depend on map ordering.

### Requirement: Connection tested before printing
Each sign-in SHALL be tested as soon as it is typed, before anything is stored or printed, so a bad credential or endpoint never yields a configuration that cannot connect. A local backend SHALL be tested before the account is printed.

The test is the same one `account check` runs, under a spinner. Its failure SHALL name each backend that failed and why: on stderr, followed by the refusal menu, for a sign-in; as the wizard's error for a local backend.

### Requirement: A refusal offers a way on
A sign-in whose test fails SHALL name on stderr each backend that failed and why, then offer `Try again` (the secret typed again, or the sign-in run again), `Change the login` (a password only: the login asked, then the password), and `Back to the review`. calendula sets up one domain, so no entry leaves it out.

#### Scenario: A mistyped password
- GIVEN a CalDAV server refusing the typed password
- WHEN the test fails
- THEN stderr names `caldav` and why, and the menu offers `Try again`, `Change the login` and `Back to the review`

### Requirement: Saved, appended, or printed
The generated account SHALL be printed as a TOML document on stdout in JSON mode and whenever stdout is redirected, so `calendula configure > config.toml` and any script keep working.

`configure` SHALL refuse outright when stdin is not a terminal, there being no way to answer its prompts.

On a terminal the target is `--config` when one was given, else the platform configuration path. A target that does not exist SHALL be offered as a file to create; one that does SHALL be offered as a block to append, so the accounts already there survive.

Appending SHALL be a plain text append rather than a re-serialization, so comments, ordering and hand-written formatting come out untouched. A declined offer SHALL fall back to printing, so the generated document is never lost.

What was written SHALL be reported: the path, the account name, and how to reach it when it did not claim the default, the name never having been asked for.

The rendered account is compact: only the `[accounts.<name>]` table stays a section header, other tables flatten into dotted keys, and empty tables and defaulted values are dropped.

Its groups read in a fixed order, the endpoint first within each, since alphabetical order would file a server under the credential authenticating against it.

### Requirement: Nothing discovered asks for a server URL
When discovery yields no route for an address or a bare domain, the wizard SHALL say nothing could be discovered for its domain, then ask `Server URL:`, read as a typed server URL is, a refused one asked again, then how it signs in. An empty answer SHALL stop the wizard with a message pointing at the documented sample (linked), writing nothing.

#### Scenario: An undiscoverable domain
- GIVEN `jane@example.org` for which discovery finds nothing
- WHEN the lookup ends
- THEN `Nothing could be discovered for example.org` shows and `Server URL:` is asked

### Requirement: Local backend auto-detected
A typed folder path or `file://` URL SHALL configure a local backend, auto-detecting the kind from on-disk markers: a pimdir index file (`pimdir.db`) means pimdir, a directory holding at least one collection (a subdirectory carrying an `.ics` file or a vdir metadata marker) means vdir. A blob directory without its index SHALL NOT mean pimdir: no reader opens a store its sync engine has not created.

pimdir SHALL be tested first, since a store also holds subdirectories and testing vdir first would misread every store as a home.

The wizard SHALL prompt vdir-against-pimdir only when both backends are compiled in and detection is inconclusive, which an empty directory is.

### Requirement: The wizard covers the discoverable backends only
The wizard SHALL configure CalDAV, the Google Calendar API (gcal), Microsoft Graph calendars (msgraph), vdir and pimdir. gcal and msgraph SHALL be offered only where a provider rule matched (by the address's domain or its MX hosts), signing in with a bearer token only, and SHALL be written as the sample configuration documents them: the token alone, the Graph user left to its `me` default.

Discovering needs the caldav feature, a typed server being a CalDAV one: gcal and msgraph are offered when compiled with it.

### Requirement: The welcome introduces the wizard only where it was not asked for
The offer SHALL be preceded by a welcome naming what calendula is, the configuration path that is missing, the sample documenting every field, and `calendula configure` for later.

`configure` typed by name SHALL skip it, whoever typed it knowing what it does. It renders on stderr, so a redirected stdout holds the document alone.
