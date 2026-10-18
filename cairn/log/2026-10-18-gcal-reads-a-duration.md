---
cairn: log
change: gcal-reads-a-duration
landed: 2026-10-18
---

# A Google event may give its end as a DURATION

`[patch.crates-io]` pins io-gcal at `9053761`, which carries pimalaya/io-gcal#1: `GcalEvent::from_ical` turns a positive DURATION into the end Google requires, the DTSTART shifted by it on its wall clock in its zone and shape, and drops the DURATION, so the event reads back with a DTEND. `event create --duration` and an invitation carrying a DURATION now reach a Google account instead of being refused. The patch goes once io-gcal is released. No calendula code moves; the other commits since io-gcal 0.2.0 are its rustls 0.23.46 bump, which calendula already holds, and a live test.

Verification: `cargo test` and `cargo clippy --all-targets -- -D warnings` with all features.

Capabilities moved: **projection** (a DURATION gives the end rather than being refused, modified).
