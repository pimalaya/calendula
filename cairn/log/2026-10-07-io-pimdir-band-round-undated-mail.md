---
cairn: log
change: io-pimdir-band-round-undated-mail
landed: 2026-10-07
---

# io-pimdir pinned at ff28408, a band round keeps undated mail

io-pimdir moves from `f9b13f8` to `ff28408`, the fix for undated mail in a band round (pimdir `00535c1`). A band round lists by a date filter that never returns undated mail, so its last page no longer infers the delete of an undated member. `PimdirWriteOp::OpenRound` and `PimdirRound` carry `band`, recorded in the new `sources.round_band` column, which the hub adds when it opens an older store. calendula's code is unchanged, as it never opens a round and the fix changes no API it uses. Tests run with the default features and with `--no-default-features --features pimdir,rustls-ring`, as MOA builds it.

Capabilities moved: none.
