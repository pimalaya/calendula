---
cairn: log
change: the-unit-tests-need-no-network
landed: 2026-11-26
---

# The unit tests need no network

The unit tests (`cargo test --bins`, the ones the nixpkgs derivation runs) bound or connected loopback sockets, which the macOS build sandbox refuses, so the derivation needed `__darwinAllowLocalNetworking = true`. Ortie fixed the same on 2026-11-21 (its log the-port-rule-is-tested-without-io). Three places did:

- src/account/check.rs `caldav_tests`: fake CalDAV servers bound on `127.0.0.1:0`.
- src/config.rs `proxy_tests`: a fake HTTP proxy bound on loopback, each backend's connect sent through it.
- src/shared/client.rs `a_reopened_backend_runs_its_secret_command_once`: an open failing on `http://127.0.0.1:1/`, which the macOS sandbox answers with a permission error rather than a refused connection.

## What landed

- src/caldav/client.rs: `connect` is now `open` (the credential, the route, the stream) then `discover` (the home-set pinned or walked to), behaviour unchanged; `build_auth` is `pub(crate)`.
- src/account/check.rs: `check_caldav` opens the client and hands it to `check_caldav_client`, which runs discovery and the PROPFIND Depth 0 on a pinned `home`. The five CalDAV check tests run it over the shared scripted stream (src/script.rs) and prove what they proved: a refused credential fails both routes alike (now also checking the `Authorization` header is sent), a 401 without `auth` names the missing `auth`, a home answering its PROPFIND passes, and a connection closed before the answer gets the PROPFIND again on the fresh stream `set_reconnect` hands over.
- src/shared/client.rs: the reopened-backend test fails its open on a block naming both `server` and `home`, refused after the credential is resolved and before anything connects.
- The four proxy tests only make sense against a real proxy, so they moved to tests/proxy.rs, the built binary running `account check` per backend through a local refusing proxy; the nixpkgs derivation skips tests/.

## Spec

No capability moved: no behaviour changed.

## Verified

`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (454 unit tests, 4 in tests/proxy.rs, every suite green), the unit test binary under `strace -f -e trace=socket,connect,bind` (no socket syscall at all), and the unit test binary in a network namespace whose loopback is down (`unshare --user --map-user=$(id -u) --map-group=$(id -g) --net`): 454 passed. Under `unshare -rn` (mapped to root) two pimdir tests fail because root reads a file made unreadable; that is the uid mapping, not the network.
