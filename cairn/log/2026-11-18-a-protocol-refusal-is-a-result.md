---
cairn: log
change: a-protocol-refusal-is-a-result
landed: 2026-11-18
---

# A protocol refusal is a result

Landed without a proposal: COH-5 with the `request` half of D26, ruled by the owner on 2026-11-18, the shape himalaya landed in 002bf012.

- src/shared/refusal.rs, himalaya's module: `RefusalOutput`, `Refusal`, `Refusable<T>` for the schemas, and `print`, which ends a protocol command by printing the refusal a reader finds in its error chain as the result, exit 3 (`PartialOutcome`), any other error staying the failure.
- Readers: `caldav::client::refusal` (the status and body of a WebDAV refusal, no reason), `gcal::client::refusal` (the status, the first reason, the message and `retryAfter` of `GcalApiError`), `msgraph::backend::refusal` (the status, `error.code`, `error.message` and `retryAfter`). `code` is the code the failure would carry, through `error::code`.
- The `caldav`, `gcal` and `msgraph` dispatchers end through `refusal::print`; src/json_schema.rs registers each key of the three namespaces as `Refusable<T>`.
- Tests: `msgraph::cli::tests::a_refused_request_is_the_result` and the module's own `a_refusal_is_the_result`.

Not ported: himalaya's `{ids, errors}` for a set applied in part, which concerns its JMAP sets; calendula's protocol writes act on one resource. A single protocol read printing raw (the other half of the ruling) is already what `gcal colors`, `gcal settings` and the `msgraph ... get` commands print.

Capabilities moved: commands.
