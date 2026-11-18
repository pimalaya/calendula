---
cairn: change
change: a-restamped-event-is-no-conflict
---

# Delta

## MODIFIED Requirements

### backends.md, msgraph

An update naming an etag is refused when the master's `changeKey` moved, unless the etag is calendula's own read (no `--if-match`) and the body read again differs from the one read only in `DTSTAMP` and `LAST-MODIFIED` lines, unfolded: then it proceeds. A caller's `--if-match` keeps the strict comparison.
