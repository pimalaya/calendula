---
cairn: log
change: shared-output-free-of-pimdir
landed: 2026-10-04
---

# Shared outputs drop the pimdir notes

The event, todo, journal and item writes, and `pimdir reply` and `pimdir cancel`, no longer end with notes or carry a `notes` array, a field pimdir alone filled. The pimdir backend logs a capability its source supports only in part as a warning. The shared docs no longer name pimdir.

Capabilities moved: **backends** (shared outputs carry no backend details).
