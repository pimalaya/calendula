---
cairn: change
id: a-vdir-id-is-one-directory-name
status: landed
created: 2026-11-15
---

# A vdir id is one directory name

> Line K18 of the family's CLI alignment round (/home/soywod/code/pimalaya/.github/cairn/changes/cli-alignment-2026-11-15), a defect left by [vdir-refuses-an-unknown-calendar](../vdir-refuses-an-unknown-calendar/proposal.md) (2fc464f) and closed in cardamum by its twin 1d047d6.

## Why

`collection_dir` took an id for one directory name when `Path::components` yielded one `Component::Normal`, but that iterator drops a trailing separator and a non-leading `.`, and the creates checked nothing. On a scratch vdir whose home is <T>/outer/home, holding `personal`, beside <T>/outer/keep holding a file, 2026-11-15:

- `calendula --json calendar delete -k personal/` answered `{"id":"personal/","queued":false}`, exit 0, and removed `personal`;
- `calendar delete -k personal/.` and `vdir delete personal/.` removed every event of `personal`, then failed uncoded with `Invalid argument (os error 22)`;
- `calendar create .. Escape` answered `{"id":"..","queued":false}`, exit 0, and wrote a `displayname` file into <T>/outer, `calendar create ../keep Escape` into <T>/outer/keep, and `calendar create x/y Escape` made <T>/outer/home/x/y;
- `vdir create ..` and `vdir create ../keep` answered exit 0, `vdir create ../<id>` made a directory beside the home and `vdir create x/y` a nested one in it, as io-vdir 0.3 runs `create_dir_all` on whatever path it is handed.

io-vdir already refuses the new name of `vdir rename` (`invalid collection name "../escape"`).

## What

- `vdir::client::is_dir_name`, cardamum's rule: an id is one directory name when it is one normal path component and the whole of the id, so neither empty, `.` nor `..`, and holding no separator, a trailing one included. `collection_dir` resolves an id only when it is one, so `calendar update`, `calendar delete`, every `event` and `item` read and write, `vdir delete` and `vdir rename` refuse `personal/` and `personal/.` with `calendar-not-found`.
- `VdirBackend::create_calendar` refuses an id that is not one directory name, ``Calendar id `<id>` is not one directory name``, and `vdir create` the same, ``Collection id `<id>` is not one directory name``, both uncoded as in cardamum, before anything is written.
- Spec: backends ("vdir refuses an unknown calendar", modified).
