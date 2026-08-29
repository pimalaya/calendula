---
cairn: change
id: first-run-wizard-frame
status: active
created: 2026-08-14
---

# Adopt Comodoro's first-run wizard frame

## Why

Comodoro's `first-run-wizard` landed the shape a Pimalaya CLI meets a newcomer with. Himalaya and Ortie have taken it, Cardamum is next, and Calendula is the last of the five.

Calendula already reasoned about this once and stopped in a defensible place: `load_config` fails with a message naming the wizard rather than running it, because a wizard that prints a document cannot hand a configuration back to a command already underway.

That is `The wizard does not serve a running command`, and it is correct given a printing wizard.

The frame changes the premise. Once the wizard writes to the resolved configuration path, it can hand a configuration back, and the two products that took the frame handle the remaining doubt by re-reading the configuration after the offer rather than assuming it landed.

A wizard that printed instead of writing leaves nothing, the re-read finds nothing, and the command fails the ordinary way. The safety the requirement protects is kept by the re-read, not by refusing to offer.

What is left is the same list as everywhere else. There is no way to run the wizard by name, so a second account means running the binary bare, and someone already set up who types `calendula` gets a wizard instead of the help.

Nothing guards interactivity, so a cron job hitting a missing configuration gets prompts. The welcome names no path, so a mistyped `-c` reads as a first run.

An existing configuration can only be overwritten, never appended to. And the account resolution failures do not list the accounts that exist or name the two ways to pick a default.

## What

Discovery is untouched. The input prompt, the typed-server-URL path, the per-service flows, the auth prompts, the connection test and the local backend detection all stay exactly as they are.

A `configure` command (alias `wizard`) runs the wizard by name, with no welcome. The welcome belongs to the offer, and gains the configuration path that was looked for.

The offer becomes a hook raised from a bare `calendula` and from any command needing a configuration. It never exits, and the configuration is re-read afterwards rather than assumed. A bare invocation falls back to the help when the offer is declined, which is also what someone already configured gets.

Nothing prompts when stdin is not a terminal or `--json` is set.

The target path stops being prompted and comes from `Config::target_path`. A configuration already there is appended to as plain text rather than overwritten, under the two rules Comodoro established: a free account name, suffixed until it is, and a single default.

The three resolution failures each name what is missing and what to do.

The rendering gains Himalaya's ordering pass: the serializer still decides what is written, but the groups run most-defining first, a group's `server` reads before the credentials qualifying it, and a blank line separates them.

## Scope / non-goals

`CALENDULA_CONFIG` already exists and already splits on `:`, so nothing to do there.

`The wizard covers the discoverable backends only` is untouched, including the build that carries none of the three wizard-capable backends. That build has no wizard to offer, so the offer is compiled out with it and the bare invocation keeps saying so by name.

gcal stays out of the wizard, written by hand from the sample, for the reasons that requirement already gives.
