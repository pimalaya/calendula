//! # Event projection and expansion
//!
//! Reads the VEVENTs of an item into [`Event`]s, and walks a recurring
//! one into the occurrences a window holds.
//!
//! Expansion is ical-rs's [`IcalRecurSet`]: `DTSTART` plus every `RRULE`
//! and `RDATE`, minus every `EXDATE`, with the `RECURRENCE-ID` overrides
//! of the same `UID` applied. It is civil, so the zone step comes after:
//! a `TZID` the time-zone database knows resolves through it, one the
//! calendar defines through its `VTIMEZONE`, a `Z` time is UTC, and a
//! floating time, or a zone nobody defines, is read in the local zone
//! (the `TZ` variable, then the system's).
//!
//! A local time a transition jumps over is read with the offset before
//! the gap, and one it repeats as its first occurrence (RFC 5545 3.3.5).
//!
//! An occurrence is addressed by its item id and its recurrence id, the
//! instance identity a `RECURRENCE-ID` would carry, spelled as the
//! series' `DTSTART` is (a date, a local time, or a `Z` time).

use std::collections::BTreeSet;

use ical::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    recur::{IcalRecurDateTime, set::IcalRecurSet},
    tree::cst::IcalCst,
    tz::{IcalTz, IcalTzOffset},
    value::IcalValue,
};
use jiff::{
    Timestamp,
    civil::DateTime,
    tz::{Offset, TimeZone},
};

use crate::shared::{
    event::{Event, EventAttendee, EventPerson},
    item::{CalendarItem, CalendarTimeRange},
};

/// How many instances a walk visits at most, whatever its window: a
/// `SECONDLY` rule from 1970 must not stall a listing.
const MAX_STEPS: usize = 200_000;

/// How far past a window's end an instance identity may sit and still
/// start inside it: a whole day of offsets either way.
const BOUND_SLACK: i64 = 2 * 86_400;

/// How far a window open on its end reaches: a year.
const OPEN_REACH: i64 = 366 * 86_400;

/// The properties an online meeting's join link rides in, in order of
/// preference: the RFC 7986 one, then the vendors' own.
const MEETING_PROPS: &[&str] = &[
    "CONFERENCE",
    "X-GOOGLE-CONFERENCE",
    "X-GOOGLE-HANGOUT-LINK",
    "X-MICROSOFT-SKYPETEAMSMEETINGURL",
    "X-MICROSOFT-ONLINEMEETINGCONFLINK",
];

/// Every VEVENT of an item, one [`Event`] each, unexpanded.
///
/// A series lists once, at its own `DTSTART`, and each override once at
/// its own. An item whose bytes do not parse yields nothing.
pub fn project(item: &CalendarItem) -> Vec<Event> {
    let Ok(cst) = IcalCst::parse(&item.contents) else {
        return Vec::new();
    };
    let ical = cst.decode();
    let context = Context::new(&ical);

    vevents(&ical)
        .map(|vevent| {
            let start = stamp(vevent, IcalPropKind::DtStart);
            let times = start
                .as_ref()
                .map(|start| context.times(start, start.civil, Span::of(&context, vevent, start)))
                .unwrap_or_default();
            let recurrence_id = prop(vevent, IcalPropKind::RecurrenceId).and_then(date_text);

            context.event(&item.id, vevent, times, recurrence_id, is_recurring(vevent))
        })
        .collect()
}

/// The occurrences of an item's events overlapping `range`, recurring
/// series expanded, in start order.
pub fn occurrences(item: &CalendarItem, range: Option<&CalendarTimeRange>) -> Vec<Event> {
    let Ok(cst) = IcalCst::parse(&item.contents) else {
        return Vec::new();
    };
    let ical = cst.decode();
    let context = Context::new(&ical);
    let window = range.map(Window::of).unwrap_or_default();

    let mut events: Vec<(i64, Event)> = Vec::new();

    for group in groups(&ical) {
        for (_, event, at) in context.walk(&item.id, &group, window.bound()) {
            if window.overlaps(&event) {
                events.push((at, event));
            }
        }
    }

    events.sort_by_key(|(at, _)| *at);
    events.into_iter().map(|(_, event)| event).collect()
}

/// The one occurrence of an item addressed by `recurrence_id`, as an
/// occurrence listing spells it.
pub fn occurrence(item: &CalendarItem, recurrence_id: &str) -> Option<Event> {
    let wanted = IcalRecurDateTime::parse(recurrence_id).ok()?;
    let cst = IcalCst::parse(&item.contents).ok()?;
    let ical = cst.decode();
    let context = Context::new(&ical);

    groups(&ical).iter().find_map(|group| {
        context
            .walk(&item.id, group, Some(wanted))
            .into_iter()
            .find(|(id, ..)| *id == Some(wanted))
            .map(|(_, event, _)| event)
    })
}

/// The events of one `UID`: its series, if the item carries it, and the
/// components overriding one of its instances.
struct Group<'a> {
    master: Option<&'a IcalComponent<'a>>,
    overrides: Vec<&'a IcalComponent<'a>>,
}

/// The VEVENTs of a calendar grouped by `UID`, in source order.
///
/// A component with no `UID` is a group of its own.
fn groups<'a>(ical: &'a Ical<'a>) -> Vec<Group<'a>> {
    let mut groups: Vec<(Option<String>, Group<'a>)> = Vec::new();

    for vevent in vevents(ical) {
        let uid = text(vevent, IcalPropKind::Uid);
        let index = match &uid {
            Some(uid) => groups
                .iter()
                .position(|(other, _)| other.as_deref() == Some(uid)),
            None => None,
        };
        let index = index.unwrap_or_else(|| {
            groups.push((
                uid,
                Group {
                    master: None,
                    overrides: Vec::new(),
                },
            ));
            groups.len() - 1
        });

        let group = &mut groups[index].1;
        if has(vevent, IcalPropKind::RecurrenceId) {
            group.overrides.push(vevent);
        } else if group.master.is_none() {
            group.master = Some(vevent);
        }
    }

    groups.into_iter().map(|(_, group)| group).collect()
}

/// The VEVENTs a calendar carries at its top level.
fn vevents<'a>(ical: &'a Ical<'a>) -> impl Iterator<Item = &'a IcalComponent<'a>> {
    ical.components.iter().filter(|component| {
        matches!(
            component.name,
            IcalComponentName::Kind(IcalComponentKind::VEvent)
        )
    })
}

/// Where a projection reads zones from: the calendar's own `VTIMEZONE`s
/// and the local zone floating times are read in.
struct Context<'a> {
    ical: &'a Ical<'a>,
    local: TimeZone,
}

/// A zone a time is read in.
#[derive(Clone)]
enum Zone {
    /// A `Z` time.
    Utc,
    /// A zone the time-zone database resolves, or the local one.
    Named(TimeZone),
    /// A zone only the calendar defines.
    Defined(IcalTz),
}

/// A time property, decoded.
#[derive(Clone, Debug)]
struct Stamp {
    /// The civil time it spells.
    civil: IcalRecurDateTime,
    /// Whether it is a `DATE`.
    date: bool,
    /// Whether it carries a trailing `Z`.
    utc: bool,
    /// Its `TZID` parameter.
    tzid: Option<String>,
}

/// The times of one occurrence, resolved.
#[derive(Clone, Debug, Default)]
pub(crate) struct Times {
    /// The start, spelled as iCalendar does.
    start: String,
    /// The end, spelled as iCalendar does.
    end: String,
    /// Whether the event spans whole days.
    all_day: bool,
    /// The start, RFC 3339 with its offset, or a date for a whole day.
    starts_at: Option<String>,
    /// The end, as `starts_at` is spelled; exclusive for a whole day.
    ends_at: Option<String>,
    /// The start in Unix seconds, a whole day's at midnight UTC.
    start_secs: Option<i64>,
    /// The end in Unix seconds, a whole day's at midnight UTC.
    end_secs: Option<i64>,
    /// The zone the start names.
    time_zone: Option<String>,
}

/// How long an event lasts, measured the way it adds to a start.
#[derive(Clone, Copy, Debug)]
enum Span {
    /// Whole days, for a `DATE` start.
    Days(i64),
    /// Seconds on the wall clock, added to the civil start.
    Civil(i64),
    /// Seconds on the timeline, added to the start instant.
    Exact(i64),
}

impl Span {
    /// The length of a component: `DTEND` minus `DTSTART`, else its
    /// `DURATION`, else a day for a date and nothing for a time.
    fn of(context: &Context<'_>, component: &IcalComponent<'_>, start: &Stamp) -> Self {
        let end = stamp(component, IcalPropKind::DtEnd);
        let duration = prop(component, IcalPropKind::Duration).and_then(|prop| match &prop.value {
            IcalValue::Duration(duration) => duration.seconds(),
            _ => None,
        });

        if start.date {
            let days = match (end, duration) {
                (Some(end), _) => (end.civil.seconds() - start.civil.seconds()) / 86_400,
                (None, Some(seconds)) => seconds / 86_400,
                (None, None) => 1,
            };
            return Self::Days(days.max(0));
        }

        match (end, duration) {
            (Some(end), _) if end.tzid == start.tzid && end.utc == start.utc => {
                Self::Civil(end.civil.seconds() - start.civil.seconds())
            }
            (Some(end), _) => {
                let from = context.resolve(start, start.civil).map(|(at, _)| at);
                let to = context.resolve(&end, end.civil).map(|(at, _)| at);
                match (from, to) {
                    (Some(from), Some(to)) => Self::Exact(to - from),
                    _ => Self::Civil(end.civil.seconds() - start.civil.seconds()),
                }
            }
            (None, Some(seconds)) => Self::Civil(seconds),
            (None, None) => Self::Civil(0),
        }
    }
}

/// What an occurrence listing narrows to, in Unix seconds and days.
#[derive(Clone, Copy, Debug, Default)]
struct Window {
    /// Inclusive start, as a UTC instant.
    start: Option<IcalRecurDateTime>,
    /// Exclusive end, as a UTC instant.
    end: Option<IcalRecurDateTime>,
}

impl Window {
    /// The window a day range names.
    fn of(range: &CalendarTimeRange) -> Self {
        let parse = |bound: &Option<String>| {
            bound
                .as_deref()
                .and_then(|bound| IcalRecurDateTime::parse(bound).ok())
        };

        Self {
            start: parse(&range.start),
            end: parse(&range.end),
        }
    }

    /// The latest identity an instance starting inside the window may
    /// carry.
    ///
    /// A window open on its end reaches [`OPEN_REACH`] past its start, or
    /// past now when it has none: an endless rule has no last instance.
    fn bound(&self) -> Option<IcalRecurDateTime> {
        let end = match self.end {
            Some(end) => end.seconds(),
            None => {
                let from = match self.start {
                    Some(start) => start.seconds(),
                    None => Timestamp::now().as_second(),
                };
                from + OPEN_REACH
            }
        };

        Some(IcalRecurDateTime::from_seconds(end + BOUND_SLACK))
    }

    /// Whether an occurrence overlaps the window.
    ///
    /// A whole day compares by date, every other by instant; one taking
    /// no time is in when its start is.
    fn overlaps(&self, event: &Event) -> bool {
        let (Some(start), Some(end)) = (event.start_secs, event.end_secs) else {
            return false;
        };

        if let Some(bound) = self.end
            && start >= bound.seconds()
        {
            return false;
        }

        if let Some(bound) = self.start.map(|bound| bound.seconds()) {
            let before = match start == end {
                true => start < bound,
                false => end <= bound,
            };
            if before {
                return false;
            }
        }

        true
    }
}

impl<'a> Context<'a> {
    fn new(ical: &'a Ical<'a>) -> Self {
        Self {
            ical,
            local: TimeZone::try_system().unwrap_or(TimeZone::UTC),
        }
    }

    /// The zone a time is read in.
    fn zone(&self, stamp: &Stamp) -> Zone {
        if stamp.utc {
            return Zone::Utc;
        }

        let Some(tzid) = &stamp.tzid else {
            return Zone::Named(self.local.clone());
        };

        if let Some(zone) = named_zone(tzid) {
            return Zone::Named(zone);
        }

        match IcalTz::of_calendar(self.ical, tzid) {
            Some(zone) => Zone::Defined(zone),
            None => Zone::Named(self.local.clone()),
        }
    }

    /// The instant a civil time names in a stamp's zone, with the offset
    /// it is shown at.
    fn resolve(&self, stamp: &Stamp, civil: IcalRecurDateTime) -> Option<(i64, i32)> {
        resolve(&self.zone(stamp), civil)
    }

    /// The times of an occurrence starting at `civil`, a start spelled
    /// like `start`, lasting `span`.
    fn times(&self, start: &Stamp, civil: IcalRecurDateTime, span: Span) -> Times {
        if start.date {
            let days = match span {
                Span::Days(days) => days,
                Span::Civil(seconds) | Span::Exact(seconds) => seconds / 86_400,
            };
            let end = IcalRecurDateTime::from_seconds(civil.seconds() + days * 86_400);

            return Times {
                start: wire(civil, true, false),
                end: wire(end, true, false),
                all_day: true,
                starts_at: Some(iso_date(civil)),
                ends_at: Some(iso_date(end)),
                start_secs: Some(civil.seconds()),
                end_secs: Some(end.seconds()),
                time_zone: None,
            };
        }

        let resolved_start = self.resolve(start, civil);
        let (end_civil, resolved_end) = match span {
            Span::Days(days) => {
                let end = IcalRecurDateTime::from_seconds(civil.seconds() + days * 86_400);
                (end, self.resolve(start, end))
            }
            Span::Civil(seconds) => {
                let end = IcalRecurDateTime::from_seconds(civil.seconds() + seconds);
                (end, self.resolve(start, end))
            }
            Span::Exact(seconds) => {
                let end = resolved_start.map(|(at, offset)| (at + seconds, offset));
                let civil = end
                    .map(|(at, offset)| IcalRecurDateTime::from_seconds(at + i64::from(offset)))
                    .unwrap_or(civil);
                (civil, end)
            }
        };

        Times {
            start: wire(civil, false, start.utc),
            end: wire(end_civil, false, start.utc),
            all_day: false,
            starts_at: resolved_start.and_then(|(at, offset)| rfc3339(at, offset)),
            ends_at: resolved_end.and_then(|(at, offset)| rfc3339(at, offset)),
            start_secs: resolved_start.map(|(at, _)| at),
            end_secs: resolved_end.map(|(at, _)| at),
            time_zone: match self.zone(start) {
                Zone::Utc => Some("UTC".into()),
                Zone::Named(zone) => start
                    .tzid
                    .clone()
                    .or_else(|| zone.iana_name().map(ToOwned::to_owned)),
                Zone::Defined(zone) => Some(zone.id),
            },
        }
    }

    /// Walks a group into its occurrences: `(identity, event, sort key)`,
    /// the identity `None` for an event that does not recur.
    ///
    /// Stops at the first instance whose identity passes `bound`, and
    /// still yields an override moved into reach from past it.
    fn walk(
        &self,
        item_id: &str,
        group: &Group<'_>,
        bound: Option<IcalRecurDateTime>,
    ) -> Vec<(Option<IcalRecurDateTime>, Event, i64)> {
        let mut out = Vec::new();

        let series = group.master.and_then(|master| {
            let start = stamp(master, IcalPropKind::DtStart)?;
            let set = IcalRecurSet::of_component(master);
            (!set.rules.is_empty() || !set.dates.is_empty()).then_some((master, start, set))
        });

        let Some((master, start, mut set)) = series else {
            // NOTE: a lone event, plus whatever overrides travel without
            // their series, an invitation to one instance being the
            // common case.
            for component in group.master.iter().chain(&group.overrides) {
                let event = self.lone(item_id, component);
                let at = event.start_secs.unwrap_or_default();
                out.push((None, event, at));
            }
            return out;
        };

        for component in &group.overrides {
            set.with_override(component);
        }

        self.localize_until(master, &start, &mut set);

        let span = Span::of(self, master, &start);
        let filter = match self.zone(&start) {
            Zone::Defined(zone) => Some(zone),
            _ => start
                .tzid
                .as_deref()
                .and_then(|tzid| IcalTz::of_calendar(self.ical, tzid)),
        };
        let walk: Box<dyn Iterator<Item = _>> = match &filter {
            Some(zone) => Box::new(set.expand_in_zone(zone)),
            None => Box::new(set.expand()),
        };

        let mut seen = BTreeSet::new();
        for occurrence in walk.take(MAX_STEPS) {
            if bound.is_some_and(|bound| occurrence.id > bound) {
                break;
            }
            seen.insert(occurrence.id);

            let recurrence_id = wire(occurrence.id, start.date, start.utc);
            let event = match occurrence
                .over
                .and_then(|index| overriding(&group.overrides, set.overrides[index].id))
            {
                Some(component) => {
                    let mut event = self.lone(item_id, component);
                    event.recurrence_id = Some(recurrence_id);
                    event
                }
                None => {
                    let times = self.times(&start, occurrence.start, span);
                    self.event(item_id, master, times, Some(recurrence_id), true)
                }
            };

            let at = event.start_secs.unwrap_or_default();
            out.push((Some(occurrence.id), event, at));
        }

        // NOTE: the walk runs in identity order, so an override moved
        // into the window from an identity past the bound was never
        // reached; it is still an occurrence of the series.
        for component in &group.overrides {
            let Some(id) = prop(component, IcalPropKind::RecurrenceId).and_then(civil_of) else {
                continue;
            };
            if seen.contains(&id) || set.exdates.binary_search(&id).is_ok() {
                continue;
            }
            let mut event = self.lone(item_id, component);
            event.recurrence_id = Some(wire(id, start.date, start.utc));
            let at = event.start_secs.unwrap_or_default();
            out.push((Some(id), event, at));
        }

        out
    }

    /// Moves each rule's UTC `UNTIL` onto the wall clock of a zoned
    /// `DTSTART`.
    ///
    /// RFC 5545 3.3.10 writes the bound in UTC whenever the start is
    /// zoned, while expansion compares civil times: unconverted, the last
    /// instance is lost or gained by the zone's offset.
    fn localize_until(&self, master: &IcalComponent<'_>, start: &Stamp, set: &mut IcalRecurSet) {
        if start.utc || start.date {
            return;
        }

        let zone = self.zone(start);
        let rules = master
            .props
            .iter()
            .filter(|prop| matches!(prop.name, IcalPropName::Kind(IcalPropKind::RRule)))
            .filter_map(|prop| match &prop.value {
                IcalValue::Recur(recur) => ical::recur::IcalRecurRule::parse(&recur.0)
                    .ok()
                    .map(|_| recur.0.to_string()),
                _ => None,
            });

        // NOTE: the set keeps the readable rules in source order, which
        // is the order they are read here.
        for (rule, raw) in set.rules.iter_mut().zip(rules) {
            let utc = raw.split(';').any(|part| {
                part.split_once('=').is_some_and(|(name, value)| {
                    name.eq_ignore_ascii_case("UNTIL") && value.ends_with(['Z', 'z'])
                })
            });
            if let (true, Some(until)) = (utc, rule.until) {
                rule.until = Some(local(&zone, until.seconds()));
            }
        }
    }

    /// One component read on its own times.
    fn lone(&self, item_id: &str, component: &IcalComponent<'_>) -> Event {
        let times = stamp(component, IcalPropKind::DtStart)
            .map(|start| self.times(&start, start.civil, Span::of(self, component, &start)))
            .unwrap_or_default();
        let recurrence_id = prop(component, IcalPropKind::RecurrenceId).and_then(date_text);

        self.event(
            item_id,
            component,
            times,
            recurrence_id,
            is_recurring(component),
        )
    }

    /// The event a component describes, at the given times.
    fn event(
        &self,
        item_id: &str,
        component: &IcalComponent<'_>,
        times: Times,
        recurrence_id: Option<String>,
        recurring: bool,
    ) -> Event {
        Event {
            id: item_id.to_owned(),
            etag: None,
            uid: text(component, IcalPropKind::Uid).unwrap_or_default(),
            recurrence_id,
            summary: text(component, IcalPropKind::Summary).unwrap_or_default(),
            description: text(component, IcalPropKind::Description).unwrap_or_default(),
            location: text(component, IcalPropKind::Location).unwrap_or_default(),
            start: times.start.clone(),
            end: times.end.clone(),
            all_day: times.all_day,
            starts_at: times.starts_at.clone(),
            ends_at: times.ends_at.clone(),
            time_zone: times.time_zone.clone(),
            recurring,
            status: text(component, IcalPropKind::Status).map(|status| status.to_uppercase()),
            transparency: text(component, IcalPropKind::Transp).map(|transp| transp.to_uppercase()),
            busy_status: named_text(component, "X-MICROSOFT-CDO-BUSYSTATUS")
                .map(|status| status.to_uppercase()),
            organizer: prop(component, IcalPropKind::Organizer).and_then(person),
            attendees: component
                .props
                .iter()
                .filter(|prop| matches!(prop.name, IcalPropName::Kind(IcalPropKind::Attendee)))
                .filter_map(attendee)
                .collect(),
            online_meeting_url: MEETING_PROPS
                .iter()
                .find_map(|name| named_text(component, name)),
            start_secs: times.start_secs,
            end_secs: times.end_secs,
        }
    }
}

/// The component among `overrides` whose `RECURRENCE-ID` names `id`.
fn overriding<'a>(
    overrides: &[&'a IcalComponent<'a>],
    id: IcalRecurDateTime,
) -> Option<&'a IcalComponent<'a>> {
    overrides.iter().copied().find(|component| {
        prop(component, IcalPropKind::RecurrenceId).and_then(civil_of) == Some(id)
    })
}

/// The zone the time-zone database answers to `tzid`.
///
/// Some writers prefix the IANA name with a path of their own
/// (`/mozilla.org/20050126_1/Europe/Paris`), so the trailing segments
/// are tried after the whole.
fn named_zone(tzid: &str) -> Option<TimeZone> {
    if let Ok(zone) = TimeZone::get(tzid) {
        return Some(zone);
    }

    let segments: Vec<&str> = tzid
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    (1..segments.len()).find_map(|skip| TimeZone::get(&segments[skip..].join("/")).ok())
}

/// The instant a civil time names in a zone, with the offset it shows.
///
/// A time a transition skips is read with the offset before it, and
/// shown with the one after; one it repeats is its first occurrence.
fn resolve(zone: &Zone, civil: IcalRecurDateTime) -> Option<(i64, i32)> {
    match zone {
        Zone::Utc => Some((civil.seconds(), 0)),
        Zone::Named(zone) => {
            let datetime = DateTime::new(
                i16::try_from(civil.year).ok()?,
                i8::try_from(civil.month).ok()?,
                i8::try_from(civil.day).ok()?,
                i8::try_from(civil.hour).ok()?,
                i8::try_from(civil.minute).ok()?,
                i8::try_from(civil.second.min(59)).ok()?,
                0,
            )
            .ok()?;
            let at = zone.to_ambiguous_timestamp(datetime).compatible().ok()?;
            Some((at.as_second(), zone.to_offset(at).seconds()))
        }
        Zone::Defined(zone) => {
            let local = civil.seconds();
            match zone.resolve(civil) {
                IcalTzOffset::One(offset) => Some((local - i64::from(offset), offset)),
                IcalTzOffset::Gap { before, after } => Some((local - i64::from(before), after)),
                IcalTzOffset::Fold { earlier, .. } => Some((local - i64::from(earlier), earlier)),
            }
        }
    }
}

/// The wall-clock time an instant shows in a zone.
fn local(zone: &Zone, at: i64) -> IcalRecurDateTime {
    let offset = match zone {
        Zone::Utc => 0,
        Zone::Named(zone) => Timestamp::from_second(at)
            .map(|at| zone.to_offset(at).seconds())
            .unwrap_or(0),
        Zone::Defined(_) => {
            // NOTE: a VTIMEZONE answers civil times only, so the offset is
            // read at the instant's UTC time, then once more at the local
            // time that gives, which settles it but within an hour of a
            // transition.
            let first = resolve(zone, IcalRecurDateTime::from_seconds(at)).map_or(0, |(_, o)| o);
            let civil = IcalRecurDateTime::from_seconds(at + i64::from(first));
            resolve(zone, civil).map_or(first, |(_, offset)| offset)
        }
    };

    IcalRecurDateTime::from_seconds(at + i64::from(offset))
}

/// An instant at an offset, as RFC 3339 spells it.
fn rfc3339(at: i64, offset: i32) -> Option<String> {
    let offset = Offset::from_seconds(offset).ok()?;
    let zoned = Timestamp::from_second(at)
        .ok()?
        .to_zoned(TimeZone::fixed(offset));
    Some(zoned.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string())
}

/// A civil date as ISO 8601 spells it.
fn iso_date(civil: IcalRecurDateTime) -> String {
    format!("{:04}-{:02}-{:02}", civil.year, civil.month, civil.day)
}

/// A civil time as iCalendar spells it: a date, a local time, or a UTC
/// time.
pub(crate) fn wire(civil: IcalRecurDateTime, date: bool, utc: bool) -> String {
    let day = format!("{:04}{:02}{:02}", civil.year, civil.month, civil.day);

    if date {
        return day;
    }

    format!(
        "{day}T{:02}{:02}{:02}{}",
        civil.hour,
        civil.minute,
        civil.second,
        if utc { "Z" } else { "" }
    )
}

/// The first property of a kind.
fn prop<'a>(component: &'a IcalComponent<'a>, kind: IcalPropKind) -> Option<&'a IcalProp<'a>> {
    component
        .props
        .iter()
        .find(|prop| matches!(prop.name, IcalPropName::Kind(found) if found == kind))
}

/// Whether a component carries a property of a kind.
fn has(component: &IcalComponent<'_>, kind: IcalPropKind) -> bool {
    prop(component, kind).is_some()
}

/// Whether a component recurs, or overrides an instance of a series.
fn is_recurring(component: &IcalComponent<'_>) -> bool {
    has(component, IcalPropKind::RRule)
        || has(component, IcalPropKind::RDate)
        || has(component, IcalPropKind::RecurrenceId)
}

/// The text of the first property of a kind, empty ones counting as none.
fn text(component: &IcalComponent<'_>, kind: IcalPropKind) -> Option<String> {
    prop(component, kind)
        .and_then(|prop| value_text(&prop.value))
        .filter(|text| !text.is_empty())
}

/// The text of the first property named `name`, a vendor one included.
fn named_text(component: &IcalComponent<'_>, name: &str) -> Option<String> {
    component
        .props
        .iter()
        .find(|prop| prop.name.eq_ignore_ascii_case(name))
        .and_then(|prop| value_text(&prop.value))
        .filter(|text| !text.is_empty())
}

/// A value read as text, whatever type it decoded as.
fn value_text(value: &IcalValue<'_>) -> Option<String> {
    match value {
        IcalValue::Text(text) => Some(text.0.to_string()),
        IcalValue::Uri(uri) => Some(uri.0.to_string()),
        IcalValue::CalAddress(address) => Some(address.0.to_string()),
        IcalValue::Unknown(unknown) => Some(
            unknown
                .components
                .iter()
                .map(|values| values.join(","))
                .collect::<Vec<_>>()
                .join(";"),
        ),
        _ => None,
    }
}

/// The date-ish text of a value, verbatim.
fn date_text(prop: &IcalProp<'_>) -> Option<String> {
    match &prop.value {
        IcalValue::Date(date) => Some(date.0.to_string()),
        IcalValue::DateTime(date) => Some(date.0.to_string()),
        IcalValue::DateTimeList(dates) => dates.0.first().map(ToString::to_string),
        _ => None,
    }
}

/// The civil time a date-ish property names.
fn civil_of(prop: &IcalProp<'_>) -> Option<IcalRecurDateTime> {
    IcalRecurDateTime::parse(&date_text(prop)?).ok()
}

/// A time property, decoded.
fn stamp(component: &IcalComponent<'_>, kind: IcalPropKind) -> Option<Stamp> {
    let prop = prop(component, kind)?;
    let raw = date_text(prop)?;
    let civil = IcalRecurDateTime::parse(&raw).ok()?;

    Some(Stamp {
        civil,
        date: matches!(prop.value, IcalValue::Date(_)) || raw.len() == 8,
        utc: raw.ends_with(['Z', 'z']),
        tzid: prop.params.iter().find_map(|param| match param {
            IcalParam::TzId(tzid) => Some(tzid.to_string()),
            _ => None,
        }),
    })
}

/// The address of a calendar user, `mailto:` removed.
fn address(prop: &IcalProp<'_>) -> Option<String> {
    let value = value_text(&prop.value)?;
    let value = value.trim();
    let address = match value.get(..7) {
        Some(scheme) if scheme.eq_ignore_ascii_case("mailto:") => &value[7..],
        _ => value,
    };
    (!address.is_empty()).then(|| address.to_owned())
}

/// The `CN` of a calendar user.
fn common_name(prop: &IcalProp<'_>) -> Option<String> {
    prop.params.iter().find_map(|param| match param {
        IcalParam::Cn(name) if !name.is_empty() => Some(name.to_string()),
        _ => None,
    })
}

/// An `ORGANIZER`.
fn person(prop: &IcalProp<'_>) -> Option<EventPerson> {
    Some(EventPerson {
        email: address(prop)?,
        name: common_name(prop),
    })
}

/// An `ATTENDEE`, its participation defaulting as RFC 5545 3.2.12 says.
fn attendee(prop: &IcalProp<'_>) -> Option<EventAttendee> {
    let param = |pick: fn(&IcalParam<'_>) -> Option<String>| prop.params.iter().find_map(pick);

    Some(EventAttendee {
        email: address(prop)?,
        name: common_name(prop),
        partstat: param(|param| match param {
            IcalParam::PartStat(value) => Some(value.to_uppercase()),
            _ => None,
        })
        .unwrap_or_else(|| "NEEDS-ACTION".into()),
        role: param(|param| match param {
            IcalParam::Role(value) => Some(value.to_uppercase()),
            _ => None,
        })
        .unwrap_or_else(|| "REQ-PARTICIPANT".into()),
        rsvp: param(|param| match param {
            IcalParam::Rsvp(value) => Some(value.to_uppercase()),
            _ => None,
        })
        .is_some_and(|rsvp| rsvp == "TRUE"),
        cutype: param(|param| match param {
            IcalParam::CuType(value) => Some(value.to_uppercase()),
            _ => None,
        })
        .unwrap_or_else(|| "INDIVIDUAL".into()),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// An item of one calendar holding `events`, `zones` before them.
    fn item(zones: &str, events: &str) -> CalendarItem {
        CalendarItem {
            id: "7".into(),
            calendar_id: "cal".into(),
            etag: None,
            contents: format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\n{zones}{events}END:VCALENDAR\r\n"
            )
            .into_bytes(),
        }
    }

    /// A VEVENT carrying `props`, CRLF-separated.
    fn vevent(props: &[&str]) -> String {
        let mut out = String::from("BEGIN:VEVENT\r\nDTSTAMP:20260101T000000Z\r\n");
        for prop in props {
            out.push_str(prop);
            out.push_str("\r\n");
        }
        out.push_str("END:VEVENT\r\n");
        out
    }

    /// A day window, both bounds inclusive.
    fn days(from: &str, to: &str) -> CalendarTimeRange {
        let day = |day: &str| chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").unwrap();
        CalendarTimeRange::from_days(Some(day(from)), Some(day(to)))
            .unwrap()
            .unwrap()
    }

    fn starts(events: &[Event]) -> Vec<&str> {
        events
            .iter()
            .map(|event| event.starts_at.as_deref().unwrap())
            .collect()
    }

    fn ids(events: &[Event]) -> Vec<&str> {
        events
            .iter()
            .map(|event| event.recurrence_id.as_deref().unwrap())
            .collect()
    }

    #[test]
    fn a_daily_series_stops_at_its_count() {
        let item = item(
            "",
            &vevent(&[
                "UID:daily@example.org",
                "DTSTART:20261005T090000Z",
                "DTEND:20261005T093000Z",
                "RRULE:FREQ=DAILY;COUNT=3",
                "SUMMARY:Stand-up",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-10-01", "2026-10-31")));

        assert_eq!(
            starts(&events),
            [
                "2026-10-05T09:00:00+00:00",
                "2026-10-06T09:00:00+00:00",
                "2026-10-07T09:00:00+00:00",
            ]
        );
        assert_eq!(
            ids(&events),
            ["20261005T090000Z", "20261006T090000Z", "20261007T090000Z"]
        );
        assert_eq!(
            events[2].ends_at.as_deref(),
            Some("2026-10-07T09:30:00+00:00")
        );
        assert!(
            events
                .iter()
                .all(|event| event.id == "7" && event.recurring)
        );
        assert!(events.iter().all(|event| event.summary == "Stand-up"));
    }

    #[test]
    fn a_weekly_series_on_three_days_stops_at_its_until_and_skips_its_exdate() {
        let item = item(
            "",
            &vevent(&[
                "UID:weekly@example.org",
                "DTSTART;TZID=Europe/Paris:20260601T140000",
                "DTEND;TZID=Europe/Paris:20260601T150000",
                "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR;UNTIL=20260612T120000Z",
                "EXDATE;TZID=Europe/Paris:20260603T140000",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-06-01", "2026-06-30")));

        assert_eq!(
            starts(&events),
            [
                "2026-06-01T14:00:00+02:00",
                "2026-06-05T14:00:00+02:00",
                "2026-06-08T14:00:00+02:00",
                "2026-06-10T14:00:00+02:00",
                "2026-06-12T14:00:00+02:00",
            ]
        );
        assert_eq!(events[0].time_zone.as_deref(), Some("Europe/Paris"));
        assert_eq!(events[0].recurrence_id.as_deref(), Some("20260601T140000"));
    }

    #[test]
    fn only_the_occurrences_overlapping_the_window_are_kept() {
        let item = item(
            "",
            &vevent(&[
                "UID:weekly@example.org",
                "DTSTART:20200106T090000Z",
                "DTEND:20200106T100000Z",
                "RRULE:FREQ=WEEKLY",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-10-05", "2026-10-11")));

        assert_eq!(starts(&events), ["2026-10-05T09:00:00+00:00"]);
    }

    #[test]
    fn an_event_started_before_the_window_overlaps_it() {
        let item = item(
            "",
            &vevent(&[
                "UID:night@example.org",
                "DTSTART:20261004T220000Z",
                "DTEND:20261005T020000Z",
            ]),
        );

        assert_eq!(
            Event::occurrences(&item, Some(&days("2026-10-05", "2026-10-05"))).len(),
            1
        );
        assert!(Event::occurrences(&item, Some(&days("2026-10-06", "2026-10-06"))).is_empty());
    }

    #[test]
    fn an_override_replaces_its_instance_and_keeps_its_identity() {
        let events = [
            vevent(&[
                "UID:sync@example.org",
                "DTSTART:20261005T090000Z",
                "DTEND:20261005T100000Z",
                "RRULE:FREQ=DAILY;COUNT=3",
                "SUMMARY:Sync",
            ]),
            vevent(&[
                "UID:sync@example.org",
                "RECURRENCE-ID:20261006T090000Z",
                "DTSTART:20261006T150000Z",
                "DTEND:20261006T153000Z",
                "SUMMARY:Sync (moved)",
            ]),
        ]
        .concat();
        let item = item("", &events);

        let events = Event::occurrences(&item, Some(&days("2026-10-01", "2026-10-31")));

        assert_eq!(events.len(), 3);
        assert_eq!(events[1].summary, "Sync (moved)");
        assert_eq!(
            events[1].starts_at.as_deref(),
            Some("2026-10-06T15:00:00+00:00")
        );
        assert_eq!(
            events[1].ends_at.as_deref(),
            Some("2026-10-06T15:30:00+00:00")
        );
        assert_eq!(events[1].recurrence_id.as_deref(), Some("20261006T090000Z"));

        let read = Event::occurrence(&item, "20261006T090000Z").unwrap();
        assert_eq!(read.summary, "Sync (moved)");
        assert!(Event::occurrence(&item, "20261009T090000Z").is_none());
    }

    #[test]
    fn an_override_moved_into_the_window_from_past_it_is_listed() {
        let events = [
            vevent(&[
                "UID:late@example.org",
                "DTSTART:20261001T090000Z",
                "RRULE:FREQ=MONTHLY;COUNT=6",
            ]),
            vevent(&[
                "UID:late@example.org",
                "RECURRENCE-ID:20270301T090000Z",
                "DTSTART:20261007T090000Z",
            ]),
        ]
        .concat();

        let events =
            Event::occurrences(&item("", &events), Some(&days("2026-10-05", "2026-10-10")));

        assert_eq!(ids(&events), ["20270301T090000Z"]);
        assert_eq!(starts(&events), ["2026-10-07T09:00:00+00:00"]);
    }

    #[test]
    fn an_all_day_series_lists_dates_with_an_exclusive_end() {
        let item = item(
            "",
            &vevent(&[
                "UID:birthday@example.org",
                "DTSTART;VALUE=DATE:20200310",
                "DTEND;VALUE=DATE:20200311",
                "RRULE:FREQ=YEARLY",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-03-10", "2026-03-10")));

        assert_eq!(events.len(), 1);
        assert!(events[0].all_day);
        assert_eq!(events[0].starts_at.as_deref(), Some("2026-03-10"));
        assert_eq!(events[0].ends_at.as_deref(), Some("2026-03-11"));
        assert_eq!(events[0].recurrence_id.as_deref(), Some("20260310"));
        assert_eq!(events[0].time_zone, None);
        assert!(Event::occurrences(&item, Some(&days("2026-03-11", "2026-03-11"))).is_empty());
    }

    #[test]
    fn a_series_crossing_the_autumn_change_in_paris_keeps_its_wall_time() {
        let item = item(
            "",
            &vevent(&[
                "UID:dst@example.org",
                "DTSTART;TZID=Europe/Paris:20261019T090000",
                "DTEND;TZID=Europe/Paris:20261019T100000",
                "RRULE:FREQ=WEEKLY;COUNT=3",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-10-01", "2026-11-30")));

        assert_eq!(
            starts(&events),
            [
                "2026-10-19T09:00:00+02:00",
                "2026-10-26T09:00:00+01:00",
                "2026-11-02T09:00:00+01:00",
            ]
        );
        assert_eq!(
            events[1].ends_at.as_deref(),
            Some("2026-10-26T10:00:00+01:00")
        );
    }

    #[test]
    fn a_zone_only_the_calendar_defines_resolves_through_its_vtimezone() {
        let zone = "BEGIN:VTIMEZONE\r\nTZID:Romance Standard Time\r\n\
            BEGIN:STANDARD\r\nDTSTART:16011028T030000\r\n\
            RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\n\
            TZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\n\
            BEGIN:DAYLIGHT\r\nDTSTART:16010325T020000\r\n\
            RRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\r\n\
            TZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nEND:DAYLIGHT\r\n\
            END:VTIMEZONE\r\n";
        let item = item(
            zone,
            &vevent(&[
                "UID:outlook@example.org",
                "DTSTART;TZID=Romance Standard Time:20261019T090000",
                "DTEND;TZID=Romance Standard Time:20261019T100000",
                "RRULE:FREQ=WEEKLY;COUNT=2",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-10-01", "2026-11-30")));

        assert_eq!(
            starts(&events),
            ["2026-10-19T09:00:00+02:00", "2026-10-26T09:00:00+01:00"]
        );
        assert_eq!(
            events[0].time_zone.as_deref(),
            Some("Romance Standard Time")
        );
    }

    #[test]
    fn a_lone_occurrence_without_its_series_lists_on_its_own() {
        let item = item(
            "",
            &vevent(&[
                "UID:one@example.org",
                "RECURRENCE-ID:20261006T090000Z",
                "DTSTART:20261006T090000Z",
                "DTEND:20261006T100000Z",
            ]),
        );

        let events = Event::occurrences(&item, Some(&days("2026-10-06", "2026-10-06")));

        assert_eq!(ids(&events), ["20261006T090000Z"]);
        assert!(events[0].recurring);
    }

    #[test]
    fn the_json_shape_carries_people_and_the_meeting_link() {
        let item = item(
            "",
            &vevent(&[
                "UID:meet@example.org",
                "DTSTART:20261005T090000Z",
                "DURATION:PT45M",
                "SUMMARY:Planning",
                "DESCRIPTION:Agenda\\nfirst",
                "LOCATION:Room 1",
                "STATUS:confirmed",
                "TRANSP:OPAQUE",
                "X-MICROSOFT-CDO-BUSYSTATUS:TENTATIVE",
                "ORGANIZER;CN=Alice:mailto:alice@example.org",
                "ATTENDEE;CN=Bob;PARTSTAT=ACCEPTED;ROLE=REQ-PARTICIPANT;RSVP=TRUE:mailto:bob@example.org",
                "ATTENDEE:MAILTO:carol@example.org",
                "ATTENDEE;CUTYPE=ROOM;PARTSTAT=ACCEPTED:mailto:room@example.org",
                "X-GOOGLE-CONFERENCE:https://meet.example.org/abc-defg-hij",
            ]),
        );

        let events = Event::project(&item);
        let value = serde_json::to_value(&events[0]).unwrap();

        assert_eq!(
            value,
            json!({
                "id": "7",
                "etag": null,
                "uid": "meet@example.org",
                "recurrenceId": null,
                "summary": "Planning",
                "description": "Agenda\nfirst",
                "location": "Room 1",
                "start": "20261005T090000Z",
                "end": "20261005T094500Z",
                "allDay": false,
                "startsAt": "2026-10-05T09:00:00+00:00",
                "endsAt": "2026-10-05T09:45:00+00:00",
                "timeZone": "UTC",
                "recurring": false,
                "status": "CONFIRMED",
                "transparency": "OPAQUE",
                "busyStatus": "TENTATIVE",
                "organizer": { "email": "alice@example.org", "name": "Alice" },
                "attendees": [
                    {
                        "email": "bob@example.org",
                        "name": "Bob",
                        "partstat": "ACCEPTED",
                        "role": "REQ-PARTICIPANT",
                        "rsvp": true,
                        "cutype": "INDIVIDUAL",
                    },
                    {
                        "email": "carol@example.org",
                        "name": null,
                        "partstat": "NEEDS-ACTION",
                        "role": "REQ-PARTICIPANT",
                        "rsvp": false,
                        "cutype": "INDIVIDUAL",
                    },
                    {
                        "email": "room@example.org",
                        "name": null,
                        "partstat": "ACCEPTED",
                        "role": "REQ-PARTICIPANT",
                        "rsvp": false,
                        "cutype": "ROOM",
                    },
                ],
                "onlineMeetingUrl": "https://meet.example.org/abc-defg-hij",
            })
        );
    }

    #[test]
    fn a_series_lists_once_unexpanded_without_a_window() {
        let item = item(
            "",
            &vevent(&[
                "UID:daily@example.org",
                "DTSTART:20261005T090000Z",
                "RRULE:FREQ=DAILY",
            ]),
        );

        let events = Event::project(&item);

        assert_eq!(events.len(), 1);
        assert!(events[0].recurring);
        assert_eq!(events[0].recurrence_id, None);
    }
}
