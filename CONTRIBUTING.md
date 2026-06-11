# Contributing guide

Thank you for investing your time in contributing to Calendula CLI.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) for the conventions every repository shares (layering, `no_std`, modules, errors, code style, licensing, notes for AI agents);
3. this guide, for how to build, test and submit changes here;
4. the repo [ARCHITECTURE](./ARCHITECTURE.md) for how Calendula CLI in particular is designed.

This document stays operational; the design lives in [ARCHITECTURE.md](./ARCHITECTURE.md).

## Development environment

The environment is managed by [Nix](https://nixos.org/download.html). `nix develop` spawns a shell with the right toolchain; every cargo command below assumes it (or prefix them with `nix develop --command`).

Without Nix, install a recent stable toolchain via [rustup](https://rust-lang.github.io/rustup/) (`rustup update`); the crate needs Rust matching the `rust-version` in [Cargo.toml](./Cargo.toml) (currently 1.87, edition 2024).

## Build

Calendula is a CLI application built on the Pimalaya sans-I/O libraries. Backends and the TLS provider are cargo features; the default set is `rustls-ring`, `caldav` and `vdir`:

```sh
cargo build                                                   # default features
cargo build --no-default-features --features vdir,rustls-ring # vdir only, no CalDAV
cargo build --release
```

`caldav` and `vdir` toggle the two backends; exactly one TLS provider must be on (`rustls-ring`, `rustls-aws`, or `native-tls`). When touching feature gates or imports, build at least the default set and a single-backend set, so no backend-only code leaks across a disabled gate.

## Lint, test, audit

```sh
cargo test                       # unit + doc tests
cargo clippy --all-targets       # keep clean for every feature set you touch
cargo fmt                        # CI checks `cargo fmt --check`
cargo deny check                 # licenses, advisories, sources
```

Before opening a PR, make sure `cargo test`, `cargo clippy`, `cargo fmt --check` and `cargo deny check` pass.

## Commit style

Calendula CLI follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary). Keep the subject imperative and scoped; describe the *why* in the body when it is not obvious.
