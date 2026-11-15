---
cairn: change
change: config-and-output-alignment
---

# Delta

## ADDED Requirements

### Requirement: A backend names the ALPN it offers
A backend whose protocol pins an HTTP version SHALL expose an `alpn` key, a list of identifiers offered during the TLS handshake. Unset offers the protocol default, an explicit empty list skips ALPN negotiation and a non-empty list replaces the default.

`gcal.alpn` defaults to `http/1.1`, the only version io-http speaks. The `caldav` block exposes none: its client pins HTTP/1.1 itself, as every DAV backend of the family does.

The runtime TLS profile SHALL be built through `TlsConfig::into_tls(alpn)`, never through a conversion assuming a list, so no caller can negotiate a protocol it did not name.

### Requirement: A defaulted scalar is not written back
A generated account SHALL omit a scalar equal to its type's default, `default = false` first among them, so what the wizard writes is what the user chose rather than the whole schema.

### Requirement: Every data command returns a named output type
A command answering with data SHALL hand the printer a `*Output` type deriving `Display`, `Serialize` and `JsonSchema`, named after its command path. `Message` carries a confirmation and never data.

Every key of such a payload SHALL be kebab-case, as the configuration schema already is.

### Requirement: The output schema is published
`calendula json-schema` SHALL print the JSON Schema of a named command's `--json` payload, or write one file per command under `--dir`. The registry SHALL be gated by the same cargo features as the commands it names, so it stays coherent under any feature combination.

## MODIFIED Requirements

### Requirement: Paths are shell-expanded
Every configured filesystem path SHALL be shell-expanded by the deserializer that reads it, so `~` and environment variables both work and no call site can forget. A path used raw would resolve against the working directory, which for a store root silently creates an empty one.

`vdir.home-dir`, `pimdir.root` and `tls.cert` are the paths concerned; the optional one goes through a local deserializer until pimalaya-config ships an optional variant of its own.

### Requirement: CalDAV locates its home-set three ways
The `caldav` block SHALL offer exactly three mutually exclusive routes, from most to least discovery. `discover` resolves a bare domain through RFC 6764 SRV records and the `.well-known` path.

`server` names the context root the principal and home-set walk starts from, taking a full URL, a bare domain or `domain:port` with `https` assumed for a bare authority, and is parsed when it is used rather than when it is read. `home` pins the home-set outright.

A block carrying none of the three SHALL be rejected by name.

## REMOVED Requirements

(none)
