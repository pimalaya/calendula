---
cairn: log
change: io-pimdir-acknowledge-receipts
landed: 2026-10-04
---

# io-pimdir pinned with neverest and himalaya

io-pimdir moves from `3a9ffb8` to `8c83c04`, the commit neverest and himalaya pin, so the set shares one io-pimdir. That commit adds `acknowledge_action`, the receipt a sync engine records when it acknowledges an intent it performed (pimdir STORAGE §15.5): a `calendar-reply` or `calendar-cancel` calendula queued reads applied afterwards rather than unknown. calendula's code is unchanged; tests run with `--no-default-features --features pimdir`, as MOA builds it.

Capabilities moved: none.
