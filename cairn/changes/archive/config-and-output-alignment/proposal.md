---
cairn: change
id: config-and-output-alignment
status: landed
created: 2026-08-29
---

# calendula diverged from the family on config style and output types

A three-way audit of himalaya, cardamum and calendula (CLI_ALIGNMENT_PLAN.md, group B) found calendula the most divergent of the three on how a configuration is read and how a command answers.

Paths were expanded at five call sites rather than at deserialize, so `tls.cert` was simply never expanded and a `cert = "~/ca.pem"` looked for a literal directory under the working one. The TLS conversion was a `From` impl hardcoding an empty ALPN list, patched back at each call site, so a new call site would silently negotiate none. The `gcal` block exposed no `alpn` key where himalaya's Google backends do. `downloads-dir` was declared, merged and accessed by nothing. `caldav.server` took a `url::Url`, so the bare authority that works in cardamum failed here. A wizard-generated account wrote a `default = false` line the rest of the family omits.

On the output side calendula derived `JsonSchema` nowhere, shipped no schema registry, and named the value a data command returns three different ways: `Events`, `CalendarsTable`, `DiscoveryReport`. Four `read` commands returned data through `Message`, which is the printer's confirmation shape.

## What changes

Every configured path is expanded by the deserializer rather than by a caller, `vdir.home-dir`, `pimdir.root` and `tls.cert` alike, the last through a hand-rolled optional variant pimalaya-config does not ship yet.

`TlsConfig::into_tls(alpn)` replaces the `From` impl, so no caller can build a TLS profile without saying what it negotiates, and `gcal.alpn` lets a user override the `http/1.1` default the Calendar API needs.

`caldav.server` becomes `Option<String>`, parsed at use, accepting a full URL, a bare domain or `domain:port` with `https` assumed. `downloads-dir` goes. `default` is skipped when false.

Every data command returns a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, registered in a `json-schema` command that prints or writes the schema of any `--json` payload.

## What does not change

Every spelling a live configuration carries still parses, `caldav.server` full URLs included. The plain-text rendering of every command is untouched, and the `caldav` block deliberately gains no `alpn` key: the DAV backends of the family pin HTTP/1.1 in the client, and cardamum's `carddav` block has none either.
