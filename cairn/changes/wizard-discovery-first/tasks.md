---
cairn: tasks
change: wizard-discovery-first
---

# Tasks

- [x] `wizard/search.rs`: bounded parallel discovery over io-pim-discovery, one entry per context root.
- [x] `wizard/caldav.rs`: discovered and manual entry points, scheme prompted from the advertised capabilities.
- [x] `wizard/secret.rs`: credentials through pimalaya-cli's keyring and token pickers.
- [x] `wizard/local.rs`: vdir against pimdir detected from on-disk markers, prompting only when ambiguous.
- [x] `wizard/discover.rs`: welcome banner, derived account name, test before emit, print or save on a terminal.
- [x] Bare `calendula` runs the wizard; a command finding no configuration fails pointing at it.
- [x] `account configure` removed; `Config::write` and the direct toml dependency removed with it.
- [x] Tests: account-name derivation, path against endpoint, scheme aliases and rejection, the generated document.
