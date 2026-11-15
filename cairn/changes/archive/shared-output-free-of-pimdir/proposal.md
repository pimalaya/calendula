---
cairn: change
id: shared-output-free-of-pimdir
status: landed
created: 2026-10-04
---

# Keep pimdir details out of the shared outputs

## Why

The shared commands are a protocol-agnostic API, and pimdir is one backend among several. The event, todo, journal and item writes had gained a `notes` array, and some shared docs named the pimdir store, both describing one backend rather than the operation.

## What

The shared write outputs, and `pimdir reply` and `pimdir cancel` with them, drop `notes`; the pimdir backend logs a capability its source supports only in part as a warning. The shared docs stop naming pimdir.
