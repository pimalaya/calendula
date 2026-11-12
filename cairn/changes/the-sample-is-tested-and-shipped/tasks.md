---
cairn: tasks
change: the-sample-is-tested-and-shipped
---

# Tasks

- [x] config.sample.toml rewritten in Himalaya's shape, every key of the old sample kept or accounted for, every key checked against src/config.rs.
- [x] Cargo.toml: pimalaya-config with `sample` as a dev-dependency.
- [x] src/config.rs: the sample test over every variant of the compiled backends, seen failing on broken copies.
- [x] package.nix: share/doc/calendula/config.sample.toml in the package; `nix build` tried.
- [x] install.sh: the sample copied under the prefix when the archive holds it.
- [x] README Configuration, CHANGELOG.
- [x] `cargo fmt`, clippy (default and all features), `cargo test --all-features` and with `rustls-ring,vdir` alone, the sample run on a scratch vdir home.
- [x] Fold the delta into the spec (config, packaging), log, tick the-family-standards, set status landed, `sh cairn/verify.sh`.
