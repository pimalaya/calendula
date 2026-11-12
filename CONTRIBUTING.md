# Contributing guide

Thank you for investing your time in contributing to calendula.

Whether you are a human or an AI agent, read these in order before touching the code:

1. the [Pimalaya README](https://github.com/pimalaya) for what the project is and how its repositories stack;
2. the [Pimalaya CONTRIBUTING](https://github.com/pimalaya/.github/blob/master/CONTRIBUTING.md) guide, which chains to the shared architecture and guidelines;
3. the inline header documentation in [src/main.rs](./src/main.rs): it is the architecture document of this crate;
4. the [cairn](./cairn) folder for the living specification, the in-flight proposals and the landed history.

Everything below documents only what differs from the Pimalaya standards.

## Feature matrix

Each backend sits behind its own cargo feature, and a network backend needs one of the TLS providers `rustls-ring`, `rustls-aws` or `native-tls`. Several may be built in, a backend block's `tls.provider` choosing among them. The default set is every backend, `rustls-ring` and `vendored`.

| Feature       | What it pulls in                                                          |
|---------------|---------------------------------------------------------------------------|
| `caldav`      | io-webdav, io-http and io-pim-discovery: the CalDAV backend and `caldav.discover` |
| `gcal`        | io-gcal: the Google Calendar API v3 backend and its iCalendar projection  |
| `msgraph`     | io-msgraph: the Microsoft Graph calendar backend and its iCalendar projection |
| `vdir`        | io-vdir: the local vdir home                                              |
| `pimdir`      | io-pimdir with its search index: the local pimdir store                   |
| `rustls-ring` | Rustls with the ring crypto provider, the default TLS provider            |
| `rustls-aws`  | Rustls with the aws-lc crypto provider                                    |
| `native-tls`  | the platform TLS stack                                                    |
| `vendored`    | SQLite, and OpenSSL under `native-tls`, built from source rather than linked |

A build with no backend compiles too: it serves `account list`, `completion`, `manual` and `json-schema`, the commands needing a backend failing.

The unified client is cfg-gated per backend, so the default build hides what a single-backend build breaks. Before a change lands, run clippy and the tests over the eleven sets the `features` job of .github/workflows/tests.yml checks:

```sh
for set in "" "--all-features" \
  "--no-default-features --features rustls-ring" \
  "--no-default-features --features rustls-ring,caldav" \
  "--no-default-features --features rustls-ring,gcal" \
  "--no-default-features --features rustls-ring,msgraph" \
  "--no-default-features --features rustls-ring,vdir" \
  "--no-default-features --features rustls-ring,pimdir" \
  "--no-default-features --features rustls-ring,vdir,pimdir" \
  "--no-default-features --features native-tls,caldav" \
  "--no-default-features --features rustls-aws,caldav"
do
  nix develop --command cargo clippy --locked --all-targets $set -- -D warnings
  nix develop --command cargo test --locked $set
done
```

## Testing against a real provider

tests/google.rs, tests/microsoft.rs and tests/fastmail.rs run the built binary against Google (Calendar API and CalDAV), Microsoft Graph and Fastmail CalDAV, each account taken from a provider section of config.sample.toml. Every test is ignored unless `--ignored` is given, and CI runs them on every push.

The `anonymous` tests use a made-up credential and touch no account:

```sh
nix develop --command cargo test --test google -- --ignored anonymous::
```

The `credentialed` tests only read the Pimalaya test accounts (`account check`, `calendar list`, `event list`), minting their own token from `CALENDULA_GOOGLE_SERVICE_ACCOUNT_KEY` (or `CALENDULA_GOOGLE_SERVICE_ACCOUNT_KEY_FILE`) for Google, and from `CALENDULA_MICROSOFT_TENANT_ID`, `CALENDULA_MICROSOFT_CLIENT_ID` and `CALENDULA_MICROSOFT_CLIENT_SECRET` for Microsoft; a test missing one fails naming it. Google CalDAV and Fastmail have no credentialed suite. Load a secret into the variable from your secret store, never paste it on the command line, and keep `RUST_LOG` at `debug` at most, trace logs carrying tokens.

## Unreleased dependencies

`[patch.crates-io]` carries the crates whose needed changes ride unreleased upstream, the cairn change adding an entry saying what it waits on, Cargo.toml carrying no comment there. An entry is a temporary state, not a convention: drop it as its crate publishes, and do not add one for a change that is already released.

## Cairn

This repository follows [Cairn](https://github.com/pimalaya/cairn): a change that affects behaviour is not done until the specification is updated and a log entry is written. See [AGENTS.md](./AGENTS.md) for the full stanza, and run `sh cairn/verify.sh` to check the structure.
