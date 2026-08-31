---
cairn: tasks
change: json-keys-are-camel-case
---

# Tasks

- [x] Switch `rename_all` to camelCase on the 22 `*Output` types carrying one
- [x] Switch it on the nested row and payload types reaching the printer
- [x] Leave every `rename_all` in src/config.rs and every clap `rename_all` alone
- [x] Confirm `EventAgendaOutput` needs no recasing, its hand-written `Serialize` and `JsonSchema` naming no field
- [x] Update the `JSON output:` help lines spelling a changed key
- [x] Regenerate the schemas and check all 23 commands for camelCase properties
- [x] Update the CHANGELOG entry the kebab-case pass wrote
- [x] Fold the delta, log
