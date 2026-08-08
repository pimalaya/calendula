---
cairn: change
change: pimdir-cache-backend
---

# Delta

## ADDED Requirements

Folded into the backends capability: the pimdir backend, its short public id, its availability-aware reads, its staged and source-guarded writes, its shell-expanded store path, its auto-sourced writes, and the `text/calendar` summary convention. The backend selection order gained pimdir between vdir and CalDAV, and the config capability gained the pimdir block.
