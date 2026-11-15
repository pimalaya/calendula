---
cairn: tasks
change: dependencies-trimmed
---

# Tasks

- [x] Cargo.toml: `default-features = false` and named features on every dependency, each checked against the item it gates (schemars without `std` also checked by compiling against a pimalaya-cli copy leaving it off); serde_json a dev-dependency; io-vdir without `serde`; url's `serde` under `caldav`; getrandom and humansize kept, the UID minting and the `item list` size column naming them.
- [x] `idna_adapter = "=1.0.0"`; `a_non_ascii_host_is_refused` (src/caldav/client.rs) seen failing on the ICU4X back end, then on the old `parse_server` fallback.
- [x] `parse_server` refuses a URL that fails to parse; `a_malformed_url_is_refused_rather_than_prefixed` seen failing on the old fallback.
- [x] `SimdCpu::install_detected()` first in `main`.
- [x] deny.toml: `[graph] targets`, licences trimmed to what the tree carries; `cargo deny check` warnings named.
- [x] `CaldavConfig::server` and `home` docs: the ASCII host.
- [x] fmt; clippy over the feature matrix (the no-backend build failing on dead code, as on b6875e0); `cargo test --all-features`; JSON schemas unchanged; crate counts, lock packages and release size before and after.
- [x] CHANGELOG; fold the delta into the spec (packaging, config); log; tick the-family-standards; status landed; `sh cairn/verify.sh`.
