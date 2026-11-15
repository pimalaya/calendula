---
cairn: change
id: vdir-refuses-an-unknown-calendar
status: landed
created: 2026-11-15
---

# vdir refuses an unknown calendar

> Line K2 (finding F31) of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15).

## Why

The vdir backend built a calendar's path without looking (src/vdir/backend.rs:122-127), so a calendar the home does not hold read as an empty one or failed uncoded. On a scratch vdir, 2026-11-15:

- `calendula --json event list -k nope` and `item list -k nope` exited 0 with `{"events":[],"nextPage":null,"errors":[]}`;
- `event create -k nope` created the directory `nope` and wrote the event into it;
- `calendar delete -k nope`, `vdir delete nope` and `vdir rename nope x` failed uncoded with `No such file or directory (os error 2)`;
- an id that is not one directory name reached out of the home: `calendar delete -k ../sibling` removed a directory beside the home, and `calendar delete -k ""` the home itself.

cardamum answers `addressbook-not-found` (cardamum/src/vdir/backend.rs, `addressbook_path`), Himalaya `mailbox-not-found`, and calendula's pimdir backend `calendar-not-found` ("pimdir refuses an unknown calendar" in backends), each exiting 1.

## What

- `vdir::client::collection_dir` resolves a calendar id to its directory only when the id is one path segment, neither `.` nor `..` (one `Component::Normal`), naming a directory that exists under the home.
- `VdirBackend::calendar_path` refuses any other id with `calendar-not-found`, ``Calendar `<id>` not found`` (cardamum's ``Addressbook `<id>` not found``), before anything is read or written: `calendar update`, `calendar delete`, and every item list, read, create, update and delete, the `event` and `item` families alike.
- `VdirClient::collection_path` refuses `vdir delete` and `vdir rename` the same way, ``Collection `<id>` not found``, as cardamum words its protocol commands.
- `calendar create` and `vdir create` name a calendar to make and keep their behaviour.
- pimdir has no such gap: `PimdirBackend::known_collection` refuses an unknown calendar on every item read and write, `pimdir search -k` and `pimdir count -k` included, and its calendar writes are refused as unsupported before the store opens.
- Spec: backends ("vdir refuses an unknown calendar", added).
