---
cairn: log
change: io-pimdir-without-probes
landed: 2026-10-07
---

# io-pimdir pinned at 35a1c3f, stores without probes

io-pimdir moves from `ef8eae0` to `35a1c3f`, the commit neverest pins for scoped mail sync. That commit drops the `probes` table, `PimdirLevel::Probed` and `count_probes`, and adds coverage; a store neverest reconciles with it has no `probes` table, which the earlier pin required. calendula's code is unchanged, as it used none of the removed API; coverage is mail only and not surfaced here. Tests run with the default features and with `--no-default-features --features pimdir,rustls-ring`, as MOA builds it.

Capabilities moved: none.
