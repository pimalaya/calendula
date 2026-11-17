---
cairn: delta
change: anonymous-dav-and-punycode-hosts
---

# Delta

## ADDED Requirements

### Requirement: A CalDAV server may ask for no credentials (config)

`caldav.auth = "none"` sends no `Authorization` header and `auth` is optional, absent reading as `none`; a 401 to such a block says the account configures no `auth`, its code unchanged. Two scenarios.

## MODIFIED Requirements

### Requirement: Hosts are ASCII (packaging)

The refusal names the host and its Punycode form; the scenario says so.

### Requirement: Dependencies name their features (packaging)

url takes no `serde`, `caldav.home` being parsed by calendula.

## REMOVED Requirements

None.
