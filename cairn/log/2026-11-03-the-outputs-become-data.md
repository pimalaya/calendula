---
cairn: log
change: the-outputs-become-data
date: 2026-11-03
---

# The outputs become data

The user bumped pimalaya-cli and pimalaya-config to 0.3 and io-pim-discovery to 0.9. pimalaya-cli 0.3 removes `Message`, which 17 writes printed and the `build -o` path too, and requires `Printer::event` and `out_bytes`.

src/shared/receipt.rs holds `WriteReceipt` (`{id, queued}`, its sentence worded by the write's kind, so a queued delete reads `Event <id> deletion queued for the next sync` as the create and the update already did) and `IdReceipt` (`{id}`). The builds' `SavedOutput` is `{path, size}`. The registry moves to `JsonSchemaRegistry::new`, registers every receipt and each build's two shapes through `OneOf`, and `every_command_is_described` checks it over the clap tree: it listed exactly the 17 writes before they were registered. pimdir's `a_write_is_confirmed_as_queued` now expects the delete sentence to name the item.

Spec: commands (Every data command returns a named output type).

`cargo test --all-features` 254 passed, clippy clean. Left: the rest of calendula's line in pimalaya-cli's ledger, listed in the change's tasks.md.
