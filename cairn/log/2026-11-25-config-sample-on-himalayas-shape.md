---
cairn: log
change: config-sample-on-himalayas-shape
landed: 2026-11-25
---

# Put config.sample.toml on Himalaya's shape

## What landed

- config.sample.toml: every line commented but `default = true`, one key per line with a short tab-aligned note, other values in brackets (`[native-tls]`), no line commented twice, `proxy` and `tls` as flat dotted keys, the per-backend ALPN, TLS and proxy keys once under a Network heading, token and password commands as `exit 127`, no broker nor sync engine named.
- config.sample.toml: one section per backend (CalDAV, JMAP, Google Calendar, Microsoft Graph, vdir, pimdir); the Google CalDAV and Fastmail sections removed, the README keeping the Google API and Fastmail recipes.
- src/config.rs: `the_sample_configuration_parses_with_each_variant` ported from Himalaya (as written, then once per rank fully uncommented, replacing paragraphs swapped from rank 2), each variant carrying a block of every backend; the pimalaya-config `sample` dev-dependency dropped.
- tests/common.rs: Himalaya's `sample_section` (first line per field, a secret's later alternative skipped); tests/google.rs and tests/fastmail.rs take the CalDAV section with the provider's server and auth set over it.
- Capabilities moved: config (the sample requirement), packaging (the live suites' accounts).
