---
cairn: tasks
change: the-kit-0-5-1-is-adopted
---

# Tasks

- [x] `cargo update -p pimalaya-cli -p pimalaya-config`, the lock diff kept to the two crates (the resolver's windows-sys churn put back).
- [x] src/account/list.rs and src/account/check.rs on `pimalaya_cli::account`; src/json_schema.rs registers the shared types.
- [x] src/repl.rs without its nested-session arm; src/cli.rs versions the `Command` enum.
- [x] src/mcp.rs answers a refused call through `Grammar::usage_error`; tests/mcp.rs reads the new wording.
- [x] src/shared/build.rs on `pimalaya_cli::output::SavedOutput`; `IcalFamily::label` and the local type go.
- [x] src/config.rs: the eight refusals through `removed_key!`; its tests read the family sentence.
- [x] Tests failing on the code before: tests/account.rs `the_account_list_is_the_shared_text` (header row), tests/repl.rs nested lines coded `usage-invalid` and `a_version_line_prints_the_version` (refused), tests/mcp.rs (clap's `'someday'`), tests/write.rs `a_build_written_to_a_file_names_its_size`, the config tests (old wording).
- [x] CHANGELOG; fold the delta into commands, repl, mcp and config; log.
