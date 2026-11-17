---
cairn: tasks
change: anonymous-dav-and-punycode-hosts
---

# Tasks

- [x] src/config.rs: `caldav.auth` defaults to `none`; `parse_url` refuses a non-ASCII host naming its Punycode form, `caldav.home` deserialized through it.
- [x] src/caldav/client.rs: `parse_server` through `parse_url`; `unauthenticated` adds the missing-`auth` sentence to a 401.
- [x] src/shared/client.rs `CalendarClient::explain` over src/session.rs `run_unified`, src/cli.rs `caldav` namespace, src/account/check.rs `check_caldav`.
- [x] Cargo.toml: `caldav` drops `url/serde`.
- [x] Tests `a_caldav_block_without_auth_loads` (src/config.rs), `a_non_ascii_host_names_its_punycode_form` (src/caldav/client.rs), `a_401_without_auth_names_the_missing_auth` (src/account/check.rs), each seen failing on the HEAD sources.
- [x] config.sample.toml, spec (config, packaging), CHANGELOG (Changed).
