---
cairn: change
change: the-releases-land
---

# Delta

## MODIFIED Requirements

(backends.md, jmap) `calendar list` SHALL read `Calendar/get`, and the calendar writes SHALL go through io-jmap's `Calendar/set` (`JmapClientStd::calendar_set`), addressed to the calendar account, one object per call.

(backends.md, pimdir refuses an online meeting no source can create) A body the user did not write as it stands, the iCalendar a raw `item create` or `item update` is given (a file, raw contents or stdin), SHALL be queued without its `X-PIMDIR-` properties (pimdir STORAGE §18, io-pimdir's `enqueue_with_body`), so it asks for nothing and no gate refuses it. A body the user wrote, through the `event` commands or composed by `item create -i` or `item update -i` from the blank or the stored item, SHALL be queued whole (`enqueue_user_body`), its `X-PIMDIR-ONLINE-MEETING` their ask. Scenario added: an imported file asks for nothing.

(backends.md, a pimdir store failure carries io-pimdir's code) A store of another format version SHALL be worded by io-pimdir's `PimdirFormatRefusal` naming calendula, ordering the versions by pimdir STORAGE §4.2: an earlier one, a draft store under a release included, synced with its engine until nothing is queued then moved aside for a new store; a later one asking for a later calendula; never to delete the store.

(backends.md, pimdir intents) The receipt SHALL be `{row, handle, id, calendarId, recurrenceId, partstat, comment, source}`, `handle` the row under the store's identity, which `pimdir queue show` and `pimdir queue cancel` take.

(backends.md, pimdir reads and retracts its queue) `<ROW>` SHALL be a bare row id, followed unchecked, or a handle, checked against the store: a handle another store or incarnation answered SHALL read `reset` under `queue show` and be refused `store-reset` by `queue cancel`, nothing dropped.

(commands.md, the `store-version` row) An earlier sync engine's store is drained by it then set aside.

(commands.md, an item is edited through a command) The composer's file SHALL be created afresh with pimalaya-cli's `file::private_options` and `create_new`: on Unix readable by its owner alone, never written through a planted file or link.

(projection.md) The read conversion SHALL be io-jmap's `JmapCalendarEvent::to_ical`.

## ADDED Requirements

None.

## REMOVED Requirements

(backends.md) The stated departure from pimdir STORAGE §18 on the raw `item add`.
