---
cairn: log
change: the-draft-path-is-quoted-by-the-kit
landed: 2026-11-19
---

# The draft path is quoted by the kit

Landed without a proposal, the CAL-13 / CAR-14 class found again by the cardamum pass of 2026-11-19: src/shared/composer.rs quoted the draft's path with a local POSIX-only `quote` (single quotes), which `cmd /C` on Windows does not read, so a shell composer could not open its draft there. It now uses pimalaya-cli 0.6.1's `shell::quote`, double quotes on Windows and single quotes elsewhere, as cardamum does; the local helper and its test `a_quoted_path_survives_a_quote_of_its_own` are gone, the kit testing its own `quote`.

No new test: the change is the Windows branch of the kit's function, which no Linux run reaches. A Windows run of a shell composer is owed.

Moved: commands.md (the composer's path on a shell line).
