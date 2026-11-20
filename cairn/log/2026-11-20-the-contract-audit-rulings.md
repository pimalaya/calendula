---
cairn: log
change: the-contract-audit-rulings
date: 2026-11-20
---

# The contract audit rulings

The owner's rulings on the CLI contract audit of 2026-11-20 (A1, A4, A6, section C), done in one pass with cardamum. Capabilities **commands**, **backends**, **config**, **mcp** moved.

- **pimalaya-cli working tree** (unreleased 0.8): `account check` prints `{accounts: [{account, backends: [{backend, ok, code, error, tls}]}]}`; a bare `--json` run fails `usage-invalid` exit 2 through `clap::meet_bare_invocation`, replacing calendula's copy; `completion`, `manual` and `json-schema` without argument are clap refusals, exit 2. Cargo.toml patches pimalaya-cli to ../cli until the release.
- **TLS (A1)**: as cardamum's, `client` and the account `tls` inherited field by field; the check row's `tls`, `null` over `http` or `jmap://`.
- **Capabilities (CAL-X1)**: `calendar.update`, `calendar.update.description`, `calendar.update.color`, `item.delete`, `item.delete.if-match`, `event.add.online-meeting`, `event.update.send-updates`; `event.search`, `full` on pimdir alone. pimdir's own `calendar.online-meeting` refusal keeps the store's name.
- **Queue (A4, CAL-X3, TWN-X1)**: `pending_rows`, `listed_row`, `uncancellable` and `row_not_found` mirror cardamum's; io-pimdir 0.18.1 fails a calendar's whole pending read on one undecodable row, so the item error names the calendar. `store_error` now prints its context as the error and the store's reason in `sources`, as cardamum's `refusal`, and the staging refusals too.
- **caldav (CAL-X8)**: `delete` takes a required item id; `rmcol` deletes a calendar, as cardamum's.
- **Search (A6, TWN-X3)**: `event search` takes the query language through the pimdir search refactored into `search_page`; built with pimdir alone, since io-pimdir has no event matcher for the other backends (cardamum matches cards with `match_cards`). The UID lookup moved to `event read --uid`; `find` is gone with it, never released.
- **Rows (TWN-X4, X5)**: `Event::calendar_id`, filled from the item; an empty `id` or `calendarId` serializes `null`. The pimdir search hit carries it through the event.
- **Codes and wording (TWN-X2, X6, X7, CAL-X4)**: `page-token-invalid` on every page token; `stdin-taken`, `item-invalid` and `sending-closed`; `Vdir home`; `Invalid item id <id> (expected a number)`; `vdir list` records and `pimdir status` columns as cardamum's.
- **Evidence**: `a_row_names_its_tls_unless_cleartext`, `a_client_certificate_needs_its_key`, `an_account_tls_fills_each_field_a_backend_leaves_unset`, `a_bare_json_invocation_is_a_refused_command_line`, `every_pending_row_lists_and_shows_as_listed`, `the_queue_refusals_carry_their_codes`, `an_undecodable_pending_row_is_an_item_error`, `an_existing_calendar_is_a_duplicate_before_a_missing_capability`, `an_item_delete_requires_its_id_and_rmcol_deletes_the_collection`, `a_uid_is_read_by_event_read_and_a_search_takes_a_query`, `an_event_search_answers_the_query_as_event_rows`, `a_hit_prints_on_one_line_and_an_error_last` (`--page-token 2`), `every_protocol_command_documents_exit_status_3`.

CHANGELOG (Added, Changed, Removed, Fixed). Spec: commands, backends, config, mcp.
