---
cairn: delta
change: io-family-releases
---

## MODIFIED Requirements

### Requirement: The stash is chunked to the provider limit (projection)
A line wider than a chunk SHALL run over as many keys as it takes, where it stayed out of the stash. A line, or a component whole, the stash has no room left for (io-gcal keeps the two stashes within 30,000 octets together) SHALL stay in the local document only: the event SHALL be written without it, on a create and on an update, and a warning SHALL name every line and component left out. Scenario added: more lines than Google stores.

### Requirement: Google Calendar backend (backends)
A calendar, event or rule id SHALL reach io-gcal as it stands, never percent-encoded by calendula: the crate encodes each as one path segment, so an id holding `#`, `?` or `/` names its own resource, and one encoded twice would name another.

### Requirement: CalDAV backend (backends)
A create SHALL keep the id of the `Location` header when the server names the resource itself, else the last segment of the URL io-webdav sent, percent-encoded, the form a listing returns. A listing the server answers with anything but one whole multistatus SHALL fail, never read as a shorter listing, and a delete answered by a multistatus naming failed members SHALL fail.

### Requirement: Microsoft Graph backend (backends)
A listing SHALL follow its pages through io-msgraph, which refuses a paging link leaving the Graph origin.

### Requirement: gcal family (commands)
An access role, a scope type or an event status the API returns and calendula does not know SHALL print as the API spells it, in text and in JSON, never fail the listing. The `acl` flags SHALL keep taking the values calendula knows alone.
