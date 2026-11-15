---
cairn: change
id: dependencies-trimmed
status: landed
created: 2026-11-12
---

# Dependencies trimmed

## Why

Change 6 of the-family-standards. Ortie (2026-11-09-dependencies-trimmed, 2026-11-09-deny-warnings-cleared), Himalaya (2026-11-11-dependencies-trimmed) and Cardamum (2026-11-12-dependencies-trimmed) name every dependency's features, run url's IDNA without its Unicode back end, install the CPU token the SIMD codecs read, and run `cargo deny check` over the release targets. calendula has not followed. anyhow, ariadne, getrandom, humansize, log, schemars, secrecy, serde, serde_json, unicode-width, url, tempfile and toml take their upstream defaults, so a default turned on upstream reaches the binary unasked; serde_json is a normal dependency though only the tests name it; io-vdir compiles its serde derives though calendula maps its types to rows of its own; url's `std` and `serde` are on in every build though only `caldav.home` deserializes a URL. url's IDNA compiles the 21 ICU4X crates, which only a host written in Unicode needs. The base64 and quoted-printable codecs io-http, io-msgraph, io-proxy, mime-rs (under io-pimdir's `search`) and pimalaya-cli run read a CPU token nothing installs, so they run the portable backend. deny.toml checks every target, Android's included, and allows 0BSD, `Apache-2.0 WITH LLVM-exception` and CDLA-Permissive-2.0, which no crate needs on the targets calendula ships.

`parse_server` (src/caldav/client.rs) prefixes `https://` to any `server` that fails to parse, not only to a bare authority: `https://dav.example.org:99999/` becomes host `https` with path `//dav.example.org:99999/`, and once the ICU4X back end leaves, `https://bücher.example/dav/` would go the same way instead of being refused.

## What

- Cargo.toml: every dependency, build and dev ones included, declares `default-features = false` and the features calendula uses, each checked against the item it gates; serde_json moves to the dev-dependencies; io-vdir loses `serde`; url's `serde` is turned on by `caldav`.
- `idna_adapter = "=1.0.0"`: a CalDAV `server` or `home` with a non-ASCII host is refused when parsed, its Punycode form working; `a_non_ascii_host_is_refused` seen failing on the ICU4X back end.
- `parse_server` prefixes `https://` only to a string without a scheme or one whose first label url read as a scheme, refusing any other parse error; `a_malformed_url_is_refused_rather_than_prefixed` seen failing first.
- `main` calls `SimdCpu::install_detected()` first (simd-cpu 0.1, `std`).
- deny.toml: Ortie's and Himalaya's `[graph] targets`; the allowed licences the ones the tree carries; `allow-org.github = ["pimalaya"]` kept while the pimalaya-cli git pin lives.
- The `server` and `home` docs name the ASCII host; CHANGELOG; spec (packaging, config); log.

## Scope / non-goals

The pimalaya-cli `[patch.crates-io]` git pin stays. Duplicates another crate imposes (getrandom 0.2 under ring, syn 2 under derive_more-impl, domain-macros and windows-core) stay visible warnings rather than skips. url and pimalaya-stream stay non-optional though only the network backends name them: making them optional changes the TLS forwarding "TLS providers are orthogonal" defines, so a vdir or pimdir build still compiles rustls. The composer's re-edit question keeps pimalaya-cli's `prompt`, inquire and crossterm in every build. The build with no backend failing clippy on dead code is every-feature-set-builds-clean's.
