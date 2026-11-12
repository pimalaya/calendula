---
cairn: log
change: the-api-is-unified
landed: 2026-11-12
---

# The API is unified

The cross-protocol API is called the unified API, its commands, writes, operations, outputs, surface and command families following, in the code, the `--help` text of `calendar`, `event` and `item`, README.md, config.sample.toml, the spec and the active jmap-backend change, as Himalaya renamed its own the same day (owner, 2026-11-12). No behaviour changed: the least-common-denominator rule stands under the new name until capability-shared-api. "Shared" stays for code shared by the backends (src/shared, the shared types, client and `paginate`). Capabilities moved: commands (requirement renamed "The unified API is a strict least common denominator"), backends ("Unified operation set", "Unified outputs carry no backend details"), config (wording only).
