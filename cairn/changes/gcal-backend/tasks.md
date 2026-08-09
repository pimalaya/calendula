---
cairn: tasks
change: gcal-backend
---

# Tasks

- [x] io-gcal: optional `If-Match` on the event write methods, so the backend can honour `if_match` instead of dropping it.
- [x] `Cargo.toml`: `gcal` feature over io-gcal, TLS features threaded through, git patch until io-gcal publishes.
- [x] `config.rs`: `GcalConfig` with the bearer `Secret` and the TLS block, wired into `AccountConfig` and the account check.
- [x] `gcal/client.rs`: connected `GcalClientStd` built from the configuration.
- [x] `gcal/project.rs`: `to_ical` and `to_event`, with the managed set, the minted `X-GOOGLE-*` properties, and the chunked `extendedProperties.private` stash. Added `merge`, which the write side needs: Google's event write replaces the whole resource, so the provider-only fields have to carry over from the server copy rather than simply be omitted.
- [x] `gcal/backend.rs`: the nine shared operations, the time range pushed down as `timeMin`/`timeMax`, pagination walked over `nextPageToken`, VTODO and VJOURNAL refused by name.
- [x] `backend.rs` and `shared/client.rs`: the `Gcal` variant, `allows_gcal`, and the selection order extended to vdir, pimdir, CalDAV, gcal.
- [x] Tests: projection round-trip per managed field, the stash surviving a write, an oversized line staying local, a non-VEVENT refused, the time range reaching the query, pagination across a page boundary.
- [x] `config.sample.toml` and the README backend list.
- [x] Live-verified against a real Google account: the whole shared surface, both create paths, the `If-Match` guard (fresh accepted, stale 412) and the VTODO refusal. Two defects it surfaced, both fixed and covered by tests:
  - a `Z`-stamped boundary returned alongside a display `timeZone` was read as that zone's wall time, shifting every event by the zone's offset;
  - the display zone was then lost on write, which would have drifted a zoned recurring series by an hour after a daylight-saving change. It is carried over from the server copy like any other provider-only field.
- [x] `create_calendar` returns the assigned id, so a create never reports one that does not exist. Google mints its own, and a warn-level log is not a report.
- [x] Credentials nest under `auth`: `gcal.auth.token`, aligning with `caldav.auth.*` here and with cardamum's `people.auth.token` / `msgraph.auth.token`.
- [x] Gate the wizard on the backends it configures: a gcal-only build compiles without it and says so, rather than offering a flow with no entries.
- [x] Fold the delta into `cairn/spec/backends.md`, `cairn/spec/config.md` and the new `cairn/spec/projection.md`, plus `commands.md`, `packaging.md` and `wizard.md`; write `cairn/log/2026-08-09-gcal-backend.md`.
