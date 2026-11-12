---
cairn: log
change: capability-shared-api
landed: 2026-11-12
---

# The unified API serves the union of the backends

Change 9 of the-family-standards (owner, 2026-11-12): the unified API is now the union of what the backends serve, held coherent by one meaning per verb, no longer a least common denominator, as Himalaya (capability-shared-api, 2026-11-02) and cardamum (2026-11-12) have it.

src/capability.rs names fifteen capabilities in pimdir STORAGE Annex B's grammar without the domain word, the collection read as the calendar and the calendar resource as the item: `calendar.list|create|rename|delete`, `calendar.description`, `calendar.color`, `item.list|read|add|update|remove`, `item.update.if-match`, `item.remove.if-match`, `online-meeting` and `scheduling.send-updates`. Five tables, keyed by the name `--backend` takes and compiled into every build, declare each at `full`, `partial`, `server` or `none`, seeded from the adapters' refusals and limits:

| Capability | vdir | pimdir | caldav | gcal | msgraph |
|---|---|---|---|---|---|
| `calendar.list` | full | full | full | full | full |
| `calendar.create` | full | none | server | partial (id minted) | partial (id minted) |
| `calendar.rename` | full | none | full | full | full |
| `calendar.delete` | full | none | server | partial (not the primary) | partial (not the default) |
| `calendar.description` | full | none | full | full | none |
| `calendar.color` | full | none | full | partial (never cleared) | none |
| `item.list`, `item.read` | full | full | full | full | full |
| `item.add` | full | partial (queued) | full | partial (events only) | partial (events only) |
| `item.update` | full | partial (queued) | full | partial (events only) | partial (events only, master alone) |
| `item.remove` | full | partial (queued) | full | full | full |
| `item.update.if-match`, `item.remove.if-match` | none | full | full | full | partial (`changeKey` read first) |
| `online-meeting` | none | partial (a source declaring it) | none | none | none |
| `scheduling.send-updates` | none | none | none | partial (not on an import) | none |

Every unified command but `build` requires the capabilities its verb and its flags reach through `CalendarClient::require` before anything is read, opened or connected; one the backend declares `none` fails with the new code `capability-missing`, naming the backend, the capability, why, and the first other backend of the account serving it with the `--backend` reaching it. `SendUpdatesArg::check_backend` and the online-meeting half of `EventFieldsArgs::check_backend` are gone, their wording kept in the details (`` `--online-meeting` needs the pimdir backend, whose sync engine creates the meeting``, `` `--send-updates` needs the gcal backend``); the adapters keep their own refusals as a second guard, and `check_backend` keeps the alarm leads. `--backend auto` still takes the first configured backend, skipping none for a capability it lacks, as Himalaya's id-exchanging verbs do: every unified verb trades calendar or item ids the others take.

`account capability` (alias `capabilities`) prints, for each backend the account configures that `--backend` allows, in the order `auto` tries them, every capability with its level and detail, connecting nowhere: `Columns` (CAPABILITY, BACKEND, LEVEL, DETAIL), `{account, backends: [{backend, capabilities: [{name, level, detail}]}]}` under `--json`, schema `calendula-account-capability`. Each unified leaf's `--help` ends with a generated `Backends:` footer (`after_long_help`), per capability the backends by level then each detail; `build` carries none, the top-level help keeps pimalaya-cli's `footer!`. `AccountConfig::backends` now lists an account's blocks for `account list`, the client and `account capability`.

What changes for a user: every refusal the tables cover carries `capability-missing` and a new message; a Graph calendar description or colour is refused before its token command runs, and a pimdir calendar write before the store is opened, where it failed with the store's own error first (`store-uncreated`); `--if-match` on a vdir `event update` or `item update`, which was ignored, is refused. Every verb a backend serves prints what it printed.

Decided: `calendar.rename`, not a `calendar.update`, Annex B having `collection.rename` and no `collection.update`, as cardamum's `addressbook.rename`; `--send-updates` is `scheduling.send-updates`, an option of Annex B's `calendar.scheduling`, since a bare `scheduling` declared `none` on an RFC 6638 server or on Graph, which notify the attendees without a per-write choice, would give the name a second meaning; `scheduling` alone, `reply` and `cancel` reach no unified verb and are left out. pimdir's rows are static, as cardamum's, its store's runtime checks refining the `partial` ones. gcal and msgraph `online-meeting` stay `none` though io-gcal 0.3.1 and io-msgraph 0.7.1 project the ask: the gcal backend sends no `conferenceDataVersion`, and serving them is its own change.

Tests: tests/capability.rs runs the built binary: a vdir `item delete --if-match` and `event update --if-match` refused with the event file untouched; a vdir and CalDAV account naming `--backend caldav`, its password command never run; CalDAV on 127.0.0.1:1 refusing `--online-meeting` and `--send-updates` with its password command never run; pimdir refusing `calendar create` and `calendar delete` with the store root never created; gcal refusing `--online-meeting` and msgraph `calendar create -d` with their token commands never run; `account capability` byte-exact for a vdir account and per backend in JSON for a vdir and pimdir one; the `item delete --help` footer byte-exact and none on `item build`. All nine failed on the code before this change: seven on `"code":null` (or `"code":"store-uncreated"` for pimdir), `account capability` an unknown subcommand, the help without its footer. The two unit tests of src/capability.rs check that every backend declares every capability once and the refusal's code and named alternative. 265 unit tests and the 23 integration tests pass with all features, the existing ones unchanged.

Verified: `cargo fmt`; `cargo clippy --locked --all-targets -D warnings` and `cargo test --locked` over the eleven feature sets of the `features` job; the anonymous live suites (`cargo test --all-features --test google --test microsoft --test fastmail -- --ignored anonymous`, 8 tests, no account touched). `json-schema -d` writes 49 schemas from 48: `calendula-account-capability.json` is new, `calendula-error.json` and `calendula-account-check.json` gain `capability-missing` in the code vocabulary with its description and the vocabulary's reworded one, and every other schema is byte-identical.

Spec: commands (opening rewritten; "The unified API is a strict least common denominator" removed; "The unified API is the union of the backends", "Capability registry", "An unserved verb is refused before connecting", "A served verb keeps its output", "An account lists what it serves", "The help names what each backend serves" added; "A JSON error carries a stable code", "A delete can be gated on a version", "An online meeting can be asked for", "A gcal write chooses who is mailed", "Text output is plain text" modified), backends ("Unified operation set", "Backend selection order", "Microsoft Graph backend", "vdir backend", "pimdir backend" modified).

Left: the MCP server listing a tool only where the account's capabilities serve it (the-shared-repl-and-mcp); pimdir rows read from what the store's sources declare, as Himalaya does; `online-meeting` on gcal and msgraph.
