---
cairn: log
change: the-x14-dependency-pass
landed: 2026-11-15
---

# The X14 dependency pass

Task X14 of the CLI alignment round, with N31, K26, K22, the D28 twin and the request origins. Capabilities moved: commands, backends.

- **X14**: pimalaya-cli 0.6.0 (`Next page:` after the listing, D27), io-msgraph 0.9.2, io-gcal 0.5.1, io-pimdir 0.14.0 (pimdir draft-13); the listing snapshots and the store-version tests follow.
- **N31**: `pimdir queue show` prints the refusals standing on the event an applied row acted on, `pimdir status` every refusal standing in its calendars (pimdir STORAGE §15.4), in cardamum's shapes.
- **K26**: the Graph and Google backends write a series' cancelled and overridden occurrences through the instances the server lists, deleting a created series again when one fails, and warn of what the server cannot hold.
- **K22**: a Google series keeps its wall time across a daylight-saving change (io-gcal 0.5.1), a regression test holding it.
- **D28 twin**: `calendar.create.id`, `none` on Google, Graph and pimdir, refuses `--id` before connecting.
- **Request origins**: `gcal request` and `msgraph request` refuse a full URL off the API's origin before sending.
