---
cairn: delta
change: a-proxy-per-account
---

# Delta

## ADDED Requirements

### Requirement: A proxy is set per account and per backend
(config) An account's `proxy` SHALL apply to every network backend of that account (CalDAV, Google Calendar, Microsoft Graph) whose own block names no `proxy`, and a backend's own `proxy` SHALL win over it. With neither, the connection SHALL read the `all_proxy` and `https_proxy` environment variables, honouring `no_proxy`.

`proxy.url` SHALL take `socks5://`, `socks5h://` or `http://`. `proxy.username` and `proxy.password` SHALL override the URL's user info, the password being a secret like any credential; a password without a username SHALL be rejected.

## MODIFIED Requirements

### Requirement: Backend blocks
(config) An account MAY carry a `proxy`, and `caldav`, `gcal` and `msgraph` each one of their own ("A proxy is set per account and per backend").

### Requirement: Unknown fields
(config) The account `proxy` of a build with no network backend SHALL load unread, as the block of a backend the build leaves out does.

## REMOVED Requirements
