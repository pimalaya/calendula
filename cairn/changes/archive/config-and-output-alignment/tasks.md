---
cairn: tasks
change: config-and-output-alignment
---

# Tasks

- [x] Expand `vdir.home-dir`, `pimdir.root` and `tls.cert` at deserialize, dropping the five call-site expansions
- [x] Replace `impl From<TlsConfig> for Tls` with `into_tls(self, alpn)`
- [x] Add `gcal.alpn`, defaulting to `http/1.1`
- [x] Skip `default` when false, through an `is_default` helper
- [x] Remove `downloads-dir`, its merge and its accessor
- [x] Reshape `caldav.server` to a string parsed at use
- [x] Replace the inline `url::Url` and the remaining inline qualified paths with `use` imports
- [x] Name every data output `*Output`, derive `JsonSchema` on it, kebab-case its keys
- [x] Add src/json_schema.rs and the `json-schema` command
- [x] Update config.sample.toml, README and CHANGELOG
- [x] Fold the delta, log
