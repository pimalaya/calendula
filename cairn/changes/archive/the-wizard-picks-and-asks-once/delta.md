---
cairn: delta
change: the-wizard-picks-and-asks-once
---

## ADDED Requirements

### Requirement: The Auto plan and the review
Every discovered way to reach the calendars SHALL be one route: a protocol, a server and a sign-in (a password, an API token or OAuth). Two routes with the same protocol, server and sign-in SHALL be one, a server offering OAuth through several grants SHALL be one OAuth route, and a server advertising no sign-in SHALL get a password route.

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

### Requirement: A password is typed once and stored
A password or an API token SHALL be asked once, masked, without confirmation, then tested. Once its test passed, the wizard SHALL offer where to keep it through pimalaya-cli's `keyring::prompt_store`, at the keyring entry `calendula-<account name>-caldav`, and SHALL write what reads it back: the keyring's read command, a custom command, or the raw value.

The login SHALL be the advertised username when it is an address, else the typed address, and SHALL be asked (`Login:`) only when neither is known.

#### Scenario: A password stored in pass
- GIVEN the account `example` signing in to CalDAV with a password, and pass installed
- WHEN the test passes and the user picks pass
- THEN the password is stored at `calendula-example-caldav` and the account reads `caldav.auth.basic.password.command = ["pass", "show", "calendula-example-caldav"]`

### Requirement: The refusal menu
A sign-in whose test fails SHALL name on stderr each backend that failed and why, then offer `Try again` (the secret typed again, or the sign-in run again), `Change the login` (a password only: the login asked, then the password), and `Back to the review`. calendula sets up one domain, so no entry leaves it out.

#### Scenario: A mistyped password
- GIVEN a CalDAV server refusing the typed password
- WHEN the test fails
- THEN stderr names `caldav` and why, and the menu offers `Try again`, `Change the login` and `Back to the review`

### Requirement: OAuth hands off to Ortie
calendula runs no OAuth 2.0 grant itself. An OAuth route SHALL be signed in through pimalaya-cli's `ortie::sign_in` with the typed address (or domain), the Ortie account `calendula-<account name>` and the route's service (`caldav`, `gcal` or `msgraphCalendar`), and SHALL write the token command it answers. When `ortie` is not on the `PATH`, the wizard SHALL print one line saying a token broker issues the token, Ortie being one, then offer the keyrings and token brokers of `keyring::prompt_token`.

#### Scenario: A calendar API signed in through Ortie
- GIVEN the account `example` for `jane@example.org`, whose Auto pick is the Google Calendar API, and `ortie` on the `PATH`
- WHEN the user continues the review
- THEN `ortie configure --name calendula-example --service gcal -- jane@example.org` then `ortie auth get -a calendula-example` run, and the account reads `gcal.auth.token.command = ["ortie", "token", "show", "-a", "calendula-example"]`

### Requirement: Nothing discovered asks for a server URL
When discovery yields no route for an address or a bare domain, the wizard SHALL say nothing could be discovered for its domain, then ask `Server URL:`, read as a typed server URL is, a refused one asked again, then how it signs in. An empty answer SHALL stop the wizard with a message pointing at the documented sample (linked), writing nothing.

#### Scenario: An undiscoverable domain
- GIVEN `jane@example.org` for which discovery finds nothing
- WHEN the lookup ends
- THEN `Nothing could be discovered for example.org` shows and `Server URL:` is asked

## MODIFIED Requirements

### Requirement: Input orients the flow
A single prompt SHALL accept an email address (or a bare domain), a `scheme://` server URL, or a local folder path, and nothing SHALL be asked before it. An email or bare domain runs io-pim-discovery's parallel discovery under a spinner; a server URL names the CalDAV context root outright; a folder is a local vdir home or pimdir store.

The wizard SHALL NOT ask which backend to configure, SHALL NOT prompt for an endpoint field it could derive, and SHALL NOT ask an HTTP authentication scheme: a password signs in with HTTP Basic, a token with HTTP Bearer.

The account name SHALL be derived, and suffixed when taken, before any sign-in, the Ortie account and the keyring entry being named after it.

#### Scenario: An address runs discovery
- GIVEN the input `jane@example.org`
- WHEN it is entered
- THEN discovery runs for `example.org`, and the next prompt is the review

### Requirement: A typed server URL is configured as given
An `https`, `caldav` or `caldavs` URL SHALL be taken as the context root, `caldav` and `caldavs` read as `https` since that is how a DAV endpoint is often written down. A plain `http` URL SHALL be refused, naming it, a password never going out unencrypted; any other scheme SHALL be rejected by name.

Nothing being advertised, the wizard SHALL ask how it signs in, a password (selected) or an API token, then go through the review and the sign-in like a discovered route, the login asked since no address was typed.

This is calendula's one deliberate deviation from himalaya's wizard, which takes no server URL at its first prompt. Mail providers are near-universally discoverable; CalDAV servers are not.

Radicale, Baikal and self-hosted Nextcloud routinely publish neither an SRV record nor a `.well-known` redirect, and refusing them would put the servers calendula's users most often run out of reach.

#### Scenario: A plain HTTP server
- GIVEN the input `http://dav.example.org/`
- WHEN it is entered
- THEN the wizard fails naming `http://dav.example.org/` and saying only `https` is accepted, and no password is asked

### Requirement: Connection tested before printing
Each sign-in SHALL be tested as soon as it is typed, before anything is stored or printed, so a bad credential or endpoint never yields a configuration that cannot connect. A local backend SHALL be tested before the account is printed.

The test is the same one `account check` runs, under a spinner. Its failure SHALL name each backend that failed and why: on stderr, followed by the refusal menu, for a sign-in; as the wizard's error for a local backend.

### Requirement: The wizard covers the discoverable backends only
The wizard SHALL configure CalDAV, the Google Calendar API (gcal), Microsoft Graph calendars (msgraph), vdir and pimdir. gcal and msgraph SHALL be offered only where a provider rule matched (by the address's domain or its MX hosts), signing in with a bearer token only, and SHALL be written as the sample configuration documents them: the token alone, the Graph user left to its `me` default.

Discovering needs the caldav feature, a typed server being a CalDAV one: gcal and msgraph are offered when compiled with it.

## REMOVED Requirements

### Requirement: One entry per service, then auth
Replaced by The Auto plan and the review: discovery picks a route instead of listing services, and the sign-in comes with the route.

### Requirement: OAuth folds into the API token
Replaced by OAuth hands off to Ortie.

### Requirement: Stop when nothing is discovered
Replaced by Nothing discovered asks for a server URL.
