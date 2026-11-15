---
cairn: change
change: shared-output-free-of-pimdir
---

# Delta

## ADDED Requirements

### Requirement: Shared outputs carry no backend details
The output of a shared command SHALL NOT carry a field that only one backend fills. A backend detail worth showing SHALL be logged by that backend's adapter or shown by its own namespace.
