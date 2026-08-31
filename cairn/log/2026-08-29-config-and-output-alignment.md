---
cairn: log
date: 2026-08-29
change: config-and-output-alignment
---

# calendula diverged from the family on config style and output types

Landed from the group B items of CLI_ALIGNMENT_PLAN.md, the cross-repo audit of the nine Pimalaya binaries.

## What landed

**[src/config.rs](../../src/config.rs) expands every path at deserialize.** `vdir.home-dir` and `pimdir.root` carry pimalaya-config's `shell_expanded_path`, and `tls.cert` a local `opt_shell_expanded_path` standing in for the optional variant pimalaya-config does not ship yet, with a TODO naming it.

`tls.cert` was expanded nowhere, so a `cert = "~/ca.pem"` looked for a literal `./~/ca.pem`. The other two were expanded by hand at five call sites ([src/vdir/client.rs](../../src/vdir/client.rs), [src/vdir/backend.rs](../../src/vdir/backend.rs), [src/account/check.rs](../../src/account/check.rs), [src/pimdir/client.rs](../../src/pimdir/client.rs)), each of which now reads the field as it stands.

**`TlsConfig::into_tls(alpn)` replaces `impl From<TlsConfig> for Tls`.** The conversion hardcoded an empty ALPN list which both clients patched back afterwards, so a third call site would have negotiated none. `gcal.alpn` is the new key letting a user override the `http/1.1` default; the `caldav` block deliberately gains none, its client pinning HTTP/1.1 as cardamum's `carddav` does.

**`caldav.server` became a string parsed at use**, accepting a full URL, a bare domain or `domain:port` with `https` assumed, which is cardamum's shape. `parse_server` in [src/caldav/client.rs](../../src/caldav/client.rs) is a copy of its `parse_carddav_server`, so the two products read one spelling the same way.

**`downloads-dir` is gone**, along with its merge in [src/account/context.rs](../../src/account/context.rs) and the accessor nothing called. calendula downloads no attachment.

**`default` is skipped when false**, through an `is_default` helper, so a wizard-generated account stops writing a line the rest of the family omits.

**Every data command returns a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`.** Eighteen types were renamed from three competing conventions (`Events`, `CalendarsTable`, `DiscoveryReport`), the four `read` commands stopped answering data through the printer's `Message`, and every payload key is kebab-case now.

**[src/json_schema.rs](../../src/json_schema.rs) registers 23 commands** and the `json-schema` subcommand prints one or writes them all under `--dir`, feature-gated so the registry stays coherent under any build.

## Verification

`cargo fmt`, `cargo check`, `cargo test` and `cargo clippy` clean under `--all-features` and across the backend feature matrix. The live `~/.calendularc`, four accounts including a CalDAV one and a gcal one, loads unchanged, and `account check` opens its tilde-rooted pimdir store.

## Capabilities moved

- config: where a path is expanded, what `caldav.server` accepts, the `alpn` key, the defaulted scalar, and `downloads-dir` gone
- commands: what a data command returns, and the published schema
