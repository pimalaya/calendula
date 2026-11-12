---
cairn: change
id: the-sample-is-tested-and-shipped
status: landed
created: 2026-11-12
---

# The sample is tested and shipped

## Why

Change 5 of the-family-standards. The first-run welcome (the-shared-kit) creates a configuration from config.sample.toml, copying it from share/doc/calendula/ when one ships beside the binary and downloading it otherwise. No calendula package ships it, so every welcome downloads, and the file it hands over is 196 lines of prose paragraphs that nothing checks: a renamed key or a stale value in it goes unnoticed. The file as copied holds an active CalDAV account pointed at `caldav.example.org` with `pass show example`, beside an active vdir home, and its pimdir root names ~/.local/state/neverest/example, a layout neverest left (Himalaya fixed the same line on 2026-11-11). Its shared options sit at the top level, before the account, where the shape the family converged on keeps one account table.

Ortie, Himalaya and comodoro ship their sample as one flat account with a section per provider or preset, parsed by their test suites once per variant (ortie 2026-11-10-providers-in-the-sample and 2026-11-10-sample-shipped-in-share, himalaya 2026-11-11-the-sample-is-tested-and-shipped, comodoro 2026-11-12-the-sample-is-tested-and-shipped), through pimalaya-config's `sample::variants` (feature `sample`, released in 0.4.0).

## What

- config.sample.toml in Himalaya's shape: no header comment, one `[accounts.example]` table with dotted keys; the options every account shares (`default`, `calendar.default`, `event.list.page-size`, `item.list.page-size`, `item.composer`), the default active and the alternatives commented, each line ending in one tab-aligned comment; then one commented section per way in, each a working account once uncommented, its own alternatives commented twice: CalDAV (the `server`, `discover` and `home` routes, HTTP Basic with an `exit 127` password command, ALPN, TLS), Google Calendar (`gcal`), Google CalDAV (the principal `https://apidata.googleusercontent.com/caldav/v2/` with a bearer token), Microsoft Graph (`msgraph`), Fastmail (CalDAV at `https://caldav.fastmail.com/dav/`, HTTP Basic with an app password), vdir, pimdir (neverest's shared store, ~/.local/state/neverest). OAuth 2.0 tokens come from `["ortie", "token", "show", "-a", "example"]`, Ortie's sample holding the matching Google and Microsoft Graph sections. Arrays stay on one line, `sample::variants` reading no inline-table element of a multi-line one.
- src/config.rs: `the_sample_configuration_parses_with_each_variant`, parsing every variant through `Config::load`, the loader `-c` uses, resolving `example` as the default account and requiring a backend block; the sections whose backend the build leaves out are skipped, their block being ignored by the loader. Cargo.toml: pimalaya-config with `sample` as a dev-dependency.
- package.nix installs config.sample.toml as share/doc/calendula/config.sample.toml, which the release archives carry with the build's share/ (pimalaya/nix's `build-with-share` job, already called by .github/workflows/releases.yml); install.sh copies it under its prefix when the archive holds it.
- README Configuration: the sample, its sections, Ortie, the shipped path and the welcome first, then the paths and `-c`; the Google, Microsoft and Fastmail snippets agree with the sections. CHANGELOG.

## Scope / non-goals

The CalDAV authentication is an enum keyed by name (`caldav.auth.basic`, `caldav.auth.bearer`, `caldav.auth = "none"`), which the variant helper cannot swap one line at a time: each section shows the one its provider takes as code, and the others are named in a comment. No configuration type changes. The test cannot see a misspelled first-level account key (`evnt.list.page-size`), the account block denying no unknown key so a build without a backend still loads a file naming it (config, Unknown fields).
