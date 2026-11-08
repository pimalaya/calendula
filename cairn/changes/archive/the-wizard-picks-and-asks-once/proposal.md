---
cairn: change
id: the-wizard-picks-and-asks-once
status: landed
created: 2026-11-08
---

# The wizard picks, and asks once

## Why

The wizard lists every context root discovery found, then asks the HTTP authentication scheme, the username, how the secret is read (a keyring entry the user must have filled beforehand, a command, or a raw value typed twice), and only then tests the account. A first run reads as a protocol questionnaire, and a password lands in a keyring only if the user put it there by hand first. A provider whose calendars are reached through its own API (the Google Calendar API, Microsoft Graph) is left out, the token story being the user's to settle, and an address discovery knows nothing about is a dead end.

The Pimalaya Android app's onboarding settled these (android cairn/spec/onboarding.md, change one-onboarding-path), and the owner approved transposing it to the CLIs on 2026-11-08. pimalaya-cli 6d8daa7 ships the shared steps: `keyring::prompt_password` (typed once, naming where it goes), `keyring::prompt_store` (stored by the wizard, read back), `review::prompt`, and `ortie::sign_in` (OAuth handed off to Ortie). himalaya and cardamum take the same change in parallel.

## What

1. One input, as today: an address or bare domain, a server URL, or a folder. Nothing is asked before discovery; the account name is derived and suffixed before any sign-in, the Ortie account and the keyring entry being named after it.
2. io-pim-discovery 0.9 to 0.11 (RFC 6764 context roots, the MX provider fallback, RFC 9728 grants, the feature `rfc9728` on). Discovery asks for CalDAV, the Google Calendar API (`Gcal`) and Microsoft Graph calendars (`MsgraphCalendar`).
3. Routes: every discovered way to the calendars is a protocol, a server and a sign-in (password, API token, OAuth). Same protocol, server and sign-in are one; several OAuth grants are one OAuth route; a server advertising nothing gets a password route.
4. Auto: the routes are ranked, a provider API before CalDAV, servers in discovery order, and within one server OAuth, token, password with `ortie` on the `PATH`, password, token, OAuth without. The first is picked without a question.
5. The review: one `Calendar` row in plain words (`CalDAV at dav.example.org, with your password`), `Continue` or `Change calendar` (Auto, every route, `Custom server URL`).
6. Sign-in: a password or an API token through `prompt_password` (the server named, flagged when off the address's domain), the login being the advertised address, else the typed one, asked only when neither exists; OAuth through `ortie::sign_in` (`calendula-<account>`, service `caldav`, `gcal` or `msgraphCalendar`), or the broker list of `prompt_token` when Ortie is missing.
7. Each sign-in tested as typed with `account check`'s test; a refusal offers `Try again`, `Change the login` (password) and `Back to the review`.
8. A tested password or token is stored through `prompt_store` at `calendula-<account>-caldav`.
9. Nothing discovered: `Server URL:` (encrypted only, `caldav(s)://` as `https`, `http://` refused and asked again), then Password or API token; empty stops as today. A server URL typed at the first prompt goes through the same steps.
10. gcal and msgraph written as config.sample.toml documents them: the token alone, `msgraph.user-id` left to its `me` default (no longer rendered).

## Decided differently from the brief

- A typed server URL, at the first prompt too, is now encrypted only: `http://` is refused, and `caldav://` reads as `https` instead of `http`. The flow makes the `Server URL:` prompt encrypted only (as the Android requirement "Setup accepts encrypted servers only"), and one parser for both prompts keeps one rule. Reverting is one match arm in `plan::server_url`.
- The network wizard stays gated on the caldav feature: a typed server and `Server URL:` are CalDAV, so gcal and msgraph are offered when compiled with it. The 18 wizard gates (`any(caldav, vdir, pimdir)`) are unchanged.
- discovery runs without secure-only mode, as before: a discovered `http://` root is still offered. The Android app drops them; the flow says nothing for the CLIs.

## Left

- Settings last: none for calendula.
- Ortie's `configure [INPUT] --service --name` is built in parallel; this side codes against the pimalaya-cli contract and was not run against a real Ortie.
