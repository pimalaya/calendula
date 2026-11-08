---
cairn: tasks
change: the-wizard-picks-and-asks-once
---

# Tasks

- [x] Cargo.toml: io-pim-discovery 0.11 (`rfc9728` left out: its lookups ignore the deadline); pimalaya-cli patched to the local checkout (6d8daa7, now f976e40)
- [x] Switch the pimalaya-cli patch to a git rev at push (lead): f976e40
- [x] src/wizard/search.rs: discovery for CalDAV, `Gcal` and `MsgraphCalendar`; routes (protocol, server, sign-in), deduped, ranked into Auto
- [x] src/wizard/plan.rs: the review row, the change list (Auto, routes, custom server URL), the server URL parser (encrypted only)
- [x] src/wizard/sign_in.rs: password and API token typed once and stored, OAuth through Ortie or the broker list, each tested, the refusal menu
- [x] src/wizard/discover.rs: the name suffixed before the sign-in, nothing discovered asks `Server URL:`, a typed URL through the same steps
- [x] src/wizard/caldav.rs removed, src/wizard/secret.rs trimmed to the converters and the broker list
- [x] src/config.rs: `msgraph.user-id` not rendered at its `me` default
- [x] Tests: dedupe, ranking, review sentences, change labels, server URL parsing, login rule, the gcal, msgraph and CalDAV blocks written
- [x] README, config.sample.toml, CHANGELOG
- [x] Fold the delta into cairn/spec/wizard.md, write the log, archive the change
