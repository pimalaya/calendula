---
cairn: tasks
change: series-exceptions-follow-the-listed-range
---

- [x] Take io-msgraph 0.9.1 (`instance_windows`)
- [x] `exceptions` takes the range to read; `list_items` passes `query.range`, the others today ± five years
- [x] Drop `series_window` and `OPEN_SERIES_YEARS`
- [x] A refused window makes the event an item error, never the listing's failure
- [x] Tests (scripted Graph): an open-ended series started six years ago lists this week's moved occurrence; no request spans five years or more; a refused window lists the other events beside the error
- [ ] Live check on the Microsoft test account: `event list` over one week with an open-ended series
- [x] CHANGELOG (Fixed, naming #31), fold into cairn/spec/backends.md, log, archive
- [ ] Reply on #31
