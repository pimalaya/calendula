# Contributing guide

Thank you for investing your time in contributing to calendula.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation in [src/main.rs](./src/main.rs): it is the architecture document of this crate;
4. the [cairn](./cairn) folder for the living specification, the in-flight proposals and the landed history.

Everything below documents only what differs from the Pimalaya standards.

## Feature matrix

Each backend sits behind its own cargo feature, and exactly one TLS provider must be on. The default set is `caldav`, `vdir`, `pimdir` and `rustls-ring`.

| Feature       | What it pulls in                                                    |
|---------------|---------------------------------------------------------------------|
| `caldav`      | io-webdav, io-http and io-pim-discovery: the CalDAV backend         |
| `vdir`        | io-vdir: the local vdir home                                        |
| `pimdir`      | io-pimdir: the local pimdir store                                   |
| `rustls-ring` | the default TLS provider                                            |
| `rustls-aws`  | Rustls with the aws-lc crypto provider                              |
| `native-tls`  | the platform TLS stack                                              |

A build with no backend compiles too: it serves `account list`, `completion`, `manual` and `json-schema`, the commands needing a backend failing.

When touching a feature gate or an import, run clippy over the matrix the `features` job of .github/workflows/tests.yml checks, so no backend-only code leaks across a disabled gate:

```sh
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features --features rustls-ring -- -D warnings
cargo clippy --all-targets --no-default-features --features rustls-ring,vdir -- -D warnings
```

and so on for `rustls-ring` with each of `caldav`, `gcal`, `msgraph` and `pimdir`, and for `native-tls,caldav`, `rustls-aws,caldav` and `rustls-ring,vdir,pimdir`.

## Unreleased dependencies

`[patch.crates-io]` carries the crates whose needed changes ride unreleased upstream, each with a note saying what it waits on. An entry is a temporary state, not a convention: drop it as its crate publishes, and do not add one for a change that is already released.

## Cairn

This repository follows [Cairn](https://github.com/pimalaya/cairn): a change that affects behaviour is not done until the specification is updated and a log entry is written. See [AGENTS.md](./AGENTS.md) for the full stanza, and run `cairn/verify.sh` to check the structure.
