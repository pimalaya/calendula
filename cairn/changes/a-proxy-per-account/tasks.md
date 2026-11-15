---
cairn: tasks
change: a-proxy-per-account
---

# Tasks

- [x] src/config.rs: `ProxyConfig` (`resolve`, `try_into_proxy`), the account `proxy` handed to every network backend naming none (`inherit_proxy`, in `take_named_account`), `proxy` on `CaldavConfig`, `GcalConfig` and `MsgraphConfig`, the account key unread without a network backend.
- [x] src/caldav/client.rs, src/msgraph/backend.rs and src/gcal/client.rs connect through it.
- [x] config.sample.toml.
- [x] Tests failing on the code before (the `proxy` key unknown): `an_account_proxy_fills_every_backend_naming_none`, `a_proxy_password_needs_a_username`, and `proxy_tests` in src/config.rs, a local HTTP proxy reading the `CONNECT` each backend sends (`calendars.invalid:80`, `www.googleapis.com:443`, `graph.microsoft.com:443`) and refusing it, so nothing leaves the machine.
- [x] CHANGELOG; fold the delta into config; log.
