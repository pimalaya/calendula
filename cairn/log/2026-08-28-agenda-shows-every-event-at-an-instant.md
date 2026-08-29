---
cairn: log
date: 2026-08-28
change: agenda-shows-every-event-at-an-instant
---

# The agenda stopped keeping one event per instant

The agenda collected its labels into a map keyed by the start instant, one label per key, so the second of two events starting at the same moment overwrote the first and nothing said so. Two unrelated meetings at 09:00 were enough, which made it a defect the calendar already had.

What made it systematic is the change landed beside it: a collection may hold two resources under one `UID` (pimdir SPEC 9), the store now keeps both instead of freezing one, and the verified Posteo pair "Pre demo woonies" and "Pre demo MINIS" start at the same instant.

A storage change whose whole purpose is to stop losing the second copy would have handed it to a view that dropped it again.

## What landed

**An instant holds every event, in src/shared/event/agenda.rs.** `CalControl.events` and `Agenda.events` are a `BTreeMap<NaiveDateTime, Vec<AgendaEvent>>` rather than a `HashMap<NaiveDateTime, String>`, and `collect_events` pushes into the bucket for the instant instead of overwriting it.

`AgendaEvent` carries the item id beside the label, which is what tells two copies of one event apart when the label cannot.

**The order is stated rather than incidental.** `Agenda::new` sorts each bucket by label, then by item id, so the order is total and the same calendar renders the same way twice.

The map being a `BTreeMap` also fixes the instants themselves in chronological order in both outputs, where the old `HashMap` was sorted for the text and emitted unordered for JSON.

**The time column prints once.** The first event at an instant carries the stamp, the rest line up under it, so a duplicated pair reads as two events rather than as two rows that happen to repeat a time.

**The JSON payload changed shape.** It was an object mapping each start datetime to one label:

```json
{"2026-08-14T09:00:00": "Pre demo MINIS"}
```

It is now an object mapping each start datetime to the list of the labels starting at it, in the order the text renders them:

```json
{"2026-08-14T09:00:00": ["Pre demo MINIS", "Pre demo woonies"]}
```

The top level and its keys are untouched, which is why this shape was chosen over the flat list of (instant, label) pairs the proposal weighed: a consumer indexes the payload by the same key it always did, and what it reads back is a list where it used to read a string.

A consumer that wants the old single value takes the first element, but doing so is choosing to drop an event, which is the behaviour this change exists to remove. No second shape is kept beside it, for the same reason.

The flag is `--json`, not the `--output json` the proposal misnamed.

**The CHANGELOG carries one `### Added` entry rather than a fix and a break.** `event agenda` landed on 2025-10-30, after v0.1.0, and has never shipped, so there is no released payload to break and no released version to fix.

`[Unreleased]` is the net diff against the last release, so both collapse into the command's own entry, which now states the multiplicity and the payload shape.

## Capabilities moved

- commands: an agenda shows every event at an instant, and the `--json` payload carries the same multiplicity

## Verification

86 tests green, `cargo clippy --all-targets` clean, `cargo fmt` applied, all through the nix devshell with the default feature set.

Four of them are the agenda's own: `two_events_at_one_instant_both_render_under_one_time` (which also pins the chronological order between instants), `two_events_at_one_instant_both_reach_the_json_payload`, `two_events_sharing_a_label_are_ordered_by_their_ids` and `a_lone_event_renders_as_one_line`.

The last is the guard that a calendar with nothing duplicated in it renders exactly as it did.
