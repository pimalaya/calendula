---
cairn: spec
capability: packaging
status: current
---

# Packaging

### Requirement: calendula is an application, not a library
The crate SHALL ship a binary only, with no library target. Its architecture document is the `src/main.rs` header, not a separate file, and the manifest carries no `documentation` field and no docs.rs metadata.

### Requirement: One cargo feature per backend
Each backend SHALL sit behind its own cargo feature (`caldav`, `vdir`, `pimdir`), pulling its io-* crates through `dep:`. A build SHALL compile at least one of them; a build with none is not a supported configuration, since the product would have nothing to talk to.

### Requirement: TLS providers are orthogonal
`rustls-ring` (the default), `rustls-aws` and `native-tls` SHALL each select a pimalaya-stream provider and forward it to every network dependency, weakly where that dependency is optional. `vendored` SHALL forward the same way.

### Requirement: Released dependencies, patched only while unreleased
Dependencies SHALL name released versions. A `[patch.crates-io]` entry is allowed only while a needed change rides unreleased upstream, SHALL carry a note saying what it waits on, and SHALL be dropped as that crate publishes.

### Requirement: No aggregator dependency
calendula SHALL depend on the protocol crates directly and own the abstraction over them. The retired per-domain aggregators (io-calendar and its siblings) SHALL NOT be reintroduced.
