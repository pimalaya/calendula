---
cairn: log
change: single-reads-print-raw
landed: 2026-11-18
---

# Single reads print raw

Landed without a proposal: the owner's ruling of 2026-11-18 that a single protocol read prints the provider's payload raw, as D18 made the protocol listings raw, which named calendula's `gcal colors` and `gcal free-busy`; himalaya landed its half in e440b4e4.

This corrects the log of a-protocol-refusal-is-a-result of the same day, which said these reads already printed raw: `gcal colors` printed a flattened `{colors}`, `gcal free-busy` a designed `{calendars: [{id, busy, queryErrors}]}`, and the `msgraph` gets io-msgraph's typed structs serialized again, dropping what they do not model.

- `shared::raw_json::RawRead`: the value as received under `--json`, the existing text form on a terminal.
- `GcalClient::raw_get` and `raw_post`, `MsgraphClient::raw_get` (the owner's path, each segment percent-encoded). `gcal colors` reads `colors`, `gcal free-busy` posts `freeBusy`, `msgraph calendars get`, `events get` and `profile get` read their resource raw, the typed value read from it for the text alone. A Graph 404 keeps its `calendar-not-found` or `item-not-found` code (`msgraph::client::coded`).
- src/json_schema.rs: the five keys describe a JSON object, inside `Refusable`.
- Test `a_single_read_prints_what_graph_sent` (an unmodelled `transactionId` kept); `the_api_errors_print_as_reasons` keeps the free-busy text.

Capabilities moved: commands.
