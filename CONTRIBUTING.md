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
| `caldav`      | io-webdav and io-http: the CalDAV backend and its discovery          |
| `vdir`        | io-vdir: the local vdir home                                        |
| `pimdir`      | io-pimdir and io-replica: the local pimdir store                     |
| `rustls-ring` | the default TLS provider                                            |
| `rustls-aws`  | Rustls with the aws-lc crypto provider                              |
| `native-tls`  | the platform TLS stack                                              |

A build must compile at least one backend: a calendula with none has nothing to talk to, and that combination is not supported.

When touching a feature gate or an import, build the default set plus at least one single-backend set, so no backend-only code leaks across a disabled gate:

```sh
cargo build
cargo build --no-default-features --features rustls-ring,vdir
cargo build --no-default-features --features rustls-ring,pimdir
cargo build --no-default-features --features rustls-ring,caldav
```

## Unreleased dependencies

`[patch.crates-io]` carries the crates whose needed changes ride unreleased upstream, each with a note saying what it waits on. An entry is a temporary state, not a convention: drop it as its crate publishes, and do not add one for a change that is already released.

## Cairn

This repository follows [Cairn](https://github.com/pimalaya/cairn): a change that affects behaviour is not done until the specification is updated and a log entry is written. See [AGENTS.md](./AGENTS.md) for the full stanza, and run `cairn/verify.sh` to check the structure.
