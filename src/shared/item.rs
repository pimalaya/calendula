//! # Item
//!
//! The iCalendar object resource shared by every backend, and the `item`
//! command family built on it.
//!
//! [`CalendarItem`] is the raw view: an id, an optional entity tag, and the
//! bytes verbatim. calendula never rewrites those bytes, so what a backend
//! stored is what a read returns, as its text when the bytes are valid
//! UTF-8 and as base64 otherwise ([`ItemContents`]).
//!
//! [`CalendarTimeRange`] narrows a listing to a day window, pushed down to
//! the server where the protocol defines such a filter and applied after
//! parsing otherwise.

pub mod build;
pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod update;

use std::{borrow::Cow, str};

use anyhow::{Context, Result, bail};
use base64_codec::codec::Base64Codec;
use chrono::{Datelike, NaiveDate};
use ical::component::IcalComponentKind;
use jiff::tz::TimeZone;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    error::ItemError,
    shared::event::{Event, list::local_zone},
};

/// A calendar object resource: one iCalendar file.
///
/// The contents mix component kinds (VEVENT, VTODO, VJOURNAL): the `event`
/// family filters them, the `item` family does not. It prints as an
/// [`ItemRow`], its bytes never decoded lossily.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CalendarItem {
    /// Backend-specific identifier the other commands address it by.
    pub id: String,
    /// The calendar the item lives in.
    pub calendar_id: String,
    /// Entity tag, for the backends offering optimistic concurrency.
    pub etag: Option<String>,
    /// Raw iCalendar bytes, exactly as the backend stored them.
    pub contents: Vec<u8>,
    /// The event a listing shows for an item whose body is not local, read
    /// from what the backend keeps beside it (a pimdir summary), never
    /// written back.
    pub stand_in: Option<Box<Event>>,
    /// The occurrences a windowed listing already read off `contents`,
    /// which [`Event::try_occurrences`] takes for that same window rather
    /// than parsing the bytes again, never written back.
    pub windowed: Option<Box<CalendarItemWindowed>>,
}

/// The occurrences of an item a windowed listing read, with the window
/// they were read for.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CalendarItemWindowed {
    /// The window the occurrences overlap.
    pub range: CalendarTimeRange,
    /// The occurrences, in start order, untagged.
    pub events: Vec<Event>,
}

/// Who wrote the body a write sends, which decides whether its
/// `X-PIMDIR-` properties are an ask (pimdir STORAGE §18).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ItemOrigin {
    /// Written by the user: through the `event` commands, or composed by
    /// `item update -i` from the stored item. An `X-PIMDIR-ONLINE-MEETING`
    /// it carries is their ask, queued whole.
    User,
    /// An iCalendar a raw `item` command was given (a file, raw contents
    /// or stdin), which the user did not write as it stands: a pimdir
    /// write removes its `X-PIMDIR-` properties first.
    Foreign,
}

impl ItemOrigin {
    /// The origin of a raw `item` write: [`Foreign`](Self::Foreign) for an
    /// iCalendar `source` gave, [`User`](Self::User) for one composed from
    /// the stored item.
    pub fn of_source(source: Option<&str>) -> Self {
        match source {
            Some(_) => Self::Foreign,
            None => Self::User,
        }
    }
}

impl Serialize for CalendarItem {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ItemRow::from(self).serialize(serializer)
    }
}

impl JsonSchema for CalendarItem {
    fn schema_name() -> Cow<'static, str> {
        ItemRow::schema_name()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        ItemRow::json_schema(generator)
    }
}

/// An item as `item list` lists it.
///
/// A listed item may carry no bytes, which only a listing shows: a read
/// fails instead ([`ItemReadRow`]).
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "CalendarItem")]
pub struct ItemRow {
    /// Backend-specific identifier the other commands address it by.
    pub id: String,
    /// The calendar the item lives in.
    pub calendar_id: String,
    /// The entity tag of the version listed, which a write can be gated on
    /// (`--if-match`), `null` when the backend versions no item.
    pub etag: Option<String>,
    /// The iCalendar text, or its base64 when `encoding` is `base64`;
    /// `null` when the item carries no bytes, a pimdir item whose body is
    /// not local yet or cannot be read.
    pub contents: Option<String>,
    /// How `contents` spells the bytes: `utf-8` for the text itself,
    /// `base64` for bytes that are not valid UTF-8; `null` when the item
    /// carries no bytes.
    pub encoding: Option<ItemEncoding>,
}

impl From<&CalendarItem> for ItemRow {
    fn from(item: &CalendarItem) -> Self {
        let (contents, encoding) = match item.contents.is_empty() {
            true => (None, None),
            false => {
                let contents = ItemContents::of(&item.contents);
                (Some(contents.contents), Some(contents.encoding))
            }
        };

        Self {
            id: item.id.clone(),
            calendar_id: item.calendar_id.clone(),
            etag: item.etag.clone(),
            contents,
            encoding,
        }
    }
}

/// An item as `item read` and `event read` read it: the row `item list`
/// prints, its bytes always there, the read of an item carrying none
/// failing (`body-pending`).
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemReadRow {
    /// Backend-specific identifier the other commands address it by.
    pub id: String,
    /// The calendar the item lives in.
    pub calendar_id: String,
    /// The entity tag of the version read, which a write can be gated on
    /// (`--if-match`), `null` when the backend versions no item.
    pub etag: Option<String>,
    /// The iCalendar, as text or as base64.
    #[serde(flatten)]
    pub contents: ItemContents,
}

impl From<&CalendarItem> for ItemReadRow {
    fn from(item: &CalendarItem) -> Self {
        Self {
            id: item.id.clone(),
            calendar_id: item.calendar_id.clone(),
            etag: item.etag.clone(),
            contents: ItemContents::of(&item.contents),
        }
    }
}

/// iCalendar bytes as JSON carries them, never decoded lossily: the text
/// when they are valid UTF-8, their base64 otherwise, as cardamum carries
/// a vCard.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemContents {
    /// The iCalendar text, or its base64 when `encoding` is `base64`.
    pub contents: String,
    /// How `contents` spells the bytes: `utf-8` for the text itself,
    /// `base64` for bytes that are not valid UTF-8.
    pub encoding: ItemEncoding,
}

impl ItemContents {
    /// The contents of `bytes`, no bytes the empty text.
    pub fn of(bytes: &[u8]) -> Self {
        let (contents, encoding) = match str::from_utf8(bytes) {
            Ok(text) => (text.to_owned(), ItemEncoding::Utf8),
            Err(_) => (base64(bytes), ItemEncoding::Base64),
        };

        Self { contents, encoding }
    }
}

/// How an item's `contents` spells its bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
pub enum ItemEncoding {
    /// The text itself, the bytes being valid UTF-8.
    #[default]
    #[serde(rename = "utf-8")]
    Utf8,
    /// The standard base64 alphabet, padded (RFC 4648 §4), the bytes not
    /// being valid UTF-8.
    #[serde(rename = "base64")]
    Base64,
}

/// The standard base64 alphabet of `bytes`, padded.
fn base64(bytes: &[u8]) -> String {
    let mut encoded = Vec::new();
    Base64Codec::default().encode_vec(bytes, &mut encoded);

    // NOTE: the base64 alphabet is ASCII, so this never falls back.
    String::from_utf8(encoded).unwrap_or_default()
}

#[cfg(any(feature = "vdir", feature = "msgraph"))]
impl CalendarItem {
    /// Whether one of the item's events starts within `range`, or one of
    /// its occurrences overlaps it, every item passing when none is named.
    ///
    /// The filter of a backend that cannot narrow a listing server-side.
    /// An item carrying no event is kept only when no range was asked for,
    /// so a filtered listing never shows an undated resource. One whose
    /// bytes do not parse is kept, so the listing names it rather than
    /// drops it.
    pub fn starts_within(&self, range: Option<&CalendarTimeRange>) -> bool {
        let Some(range) = range else {
            return true;
        };

        let Ok(events) = Event::try_project(self) else {
            return true;
        };

        events.iter().any(|event| range.contains(&event.wire_start))
            || !Event::occurrences(self, Some(range)).is_empty()
    }
}

/// The items a backend listed, beside the error of each item it could not
/// read, in the order met.
///
/// A backend reading the whole calendar before cutting its window names
/// its errors on the first window alone, one listing an item it could not
/// read names it beside that item.
#[derive(Clone, Debug, Default)]
pub struct CalendarItemList {
    /// The items of the window, in the order the backend lists them.
    pub items: Vec<CalendarItem>,
    /// The error of each item the backend could not read.
    pub errors: Vec<ItemError>,
    /// The ids of the listed places the window covers, in order, from a
    /// backend cutting its window before it reads the bodies: an item it
    /// could not read keeps its place. `None` when the window counts the
    /// items read alone.
    pub places: Option<Vec<String>>,
}

impl CalendarItemList {
    /// Cuts a window read one place past a page of `size` into that page,
    /// every item when `size` is `None`.
    ///
    /// The place past the page is the next page's, and so are its item and
    /// an entry naming it; that it exists says another page follows.
    pub fn into_page(self, size: Option<usize>) -> CalendarItemPage {
        let Self {
            mut items,
            mut errors,
            places,
        } = self;

        let mut places =
            places.unwrap_or_else(|| items.iter().map(|item| item.id.clone()).collect());
        let more = size.is_some_and(|size| places.len() > size);

        if let Some(size) = size
            && more
        {
            let past: Vec<String> = places.drain(size..).collect();
            items.retain(|item| !past.contains(&item.id));
            errors.retain(|error| error.id.as_ref().is_none_or(|id| !past.contains(id)));
        }

        CalendarItemPage {
            items,
            more,
            errors,
        }
    }
}

/// One page of an item listing, and whether an item follows it.
#[derive(Clone, Debug, Default)]
pub struct CalendarItemPage {
    /// The items of the page, in the order the backend lists them.
    pub items: Vec<CalendarItem>,
    /// Whether the listing holds an item past the page.
    pub more: bool,
    /// The error of each item of the page the backend could not read.
    pub errors: Vec<ItemError>,
}

/// What narrows an item listing, and in which order.
///
/// The kind is applied before the window, so a page of `event list` is a
/// page of events: filtering after paginating would make it show the
/// VEVENTs among the first 25 items of any kind, which is nothing at all
/// on a calendar of to-dos.
#[derive(Clone, Copy, Debug, Default)]
pub struct CalendarItemQuery<'a> {
    /// Matching items to pass over before the window starts.
    pub skip: usize,
    /// Matching items the window holds at most, `None` for every one.
    pub take: Option<usize>,
    /// Day window the item's own dates are read against.
    #[cfg_attr(
        not(any(
            feature = "vdir",
            feature = "caldav",
            feature = "jmap",
            feature = "pimdir",
            feature = "gcal",
            feature = "msgraph"
        )),
        allow(dead_code)
    )]
    pub range: Option<&'a CalendarTimeRange>,
    /// Component kind to keep, `None` keeping every kind, which is what
    /// the raw `item` family asks for.
    #[cfg_attr(
        not(any(
            feature = "vdir",
            feature = "caldav",
            feature = "jmap",
            feature = "pimdir",
            feature = "gcal",
            feature = "msgraph"
        )),
        allow(dead_code)
    )]
    pub kind: Option<IcalComponentKind>,
    /// A `UID` the caller filters the projected events by, a hint only:
    /// a backend MAY leave out an item it can tell carries another one
    /// without reading it, and the caller still compares every event it
    /// gets. Leaving items out would shift a window, so a backend honours
    /// it only on a listing taking every item.
    #[cfg_attr(not(any(feature = "pimdir", feature = "gcal")), allow(dead_code))]
    pub uid: Option<&'a str>,
}

/// An inclusive day window narrowing a listing.
///
/// Both bounds are optional, so a range may be open on either side. The
/// stored values are iCalendar UTC date-times (RFC 5545 3.3.5), the form the
/// CalDAV `time-range` filter takes (RFC 4791 9.9).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarTimeRange {
    /// Inclusive lower bound, as `YYYYMMDDTHHMMSSZ`.
    #[serde(default)]
    pub start: Option<String>,
    /// Exclusive upper bound, as `YYYYMMDDTHHMMSSZ`.
    #[serde(default)]
    pub end: Option<String>,
}

impl CalendarTimeRange {
    /// Builds a range from the inclusive `--from` and `--to` days, local
    /// ones (`TZ`, then the system's zone).
    ///
    /// Two absent bounds give `None`, and crossed ones bail.
    pub fn from_days(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Option<Self>> {
        Self::from_days_in(from, to, &local_zone())
    }

    /// Builds a range from the inclusive `from` and `to` days of `zone`.
    ///
    /// Each bound is the UTC instant of a midnight in `zone`: `from`'s, and
    /// for `to` the next day's, the exclusive upper bound the wire format
    /// wants.
    pub fn from_days_in(
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        zone: &TimeZone,
    ) -> Result<Option<Self>> {
        if let (Some(from), Some(to)) = (from, to)
            && to < from
        {
            bail!("Invalid time range: `--to` {to} is before `--from` {from}");
        }

        if from.is_none() && to.is_none() {
            return Ok(None);
        }

        let end = match to.and_then(|day| day.succ_opt()) {
            Some(day) => Some(midnight(day, zone)?),
            None => None,
        };
        let start = match from {
            Some(day) => Some(midnight(day, zone)?),
            None => None,
        };

        Ok(Some(Self { start, end }))
    }

    /// Whether an iCalendar date or date-time falls inside the range, the
    /// local zone telling the days of the bounds.
    #[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
    pub fn contains(&self, stamp: &str) -> bool {
        self.contains_in(stamp, &local_zone())
    }

    /// Whether an iCalendar date or date-time falls inside the range.
    ///
    /// A UTC date-time compares as an instant. A date or a local date-time
    /// names no instant without resolving its zone, so its day compares to
    /// the days the bounds fall on in `zone`.
    #[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
    pub fn contains_in(&self, stamp: &str, zone: &TimeZone) -> bool {
        if stamp.len() == 16 && stamp.ends_with('Z') {
            let after_start = self.start.as_deref().is_none_or(|start| stamp >= start);
            let before_end = self.end.as_deref().is_none_or(|end| stamp < end);
            return after_start && before_end;
        }

        let day = stamp.get(..8).unwrap_or(stamp);
        let local_day = |bound: &str| bound_day(bound, zone);

        if let Some(start) = self.start.as_deref()
            && day < local_day(start).as_str()
        {
            return false;
        }

        if let Some(end) = self.end.as_deref()
            && day >= local_day(end).as_str()
        {
            return false;
        }

        true
    }

    /// The CalDAV `time-range` element (RFC 4791 9.9) for this range, which
    /// the caller nests inside a component filter.
    #[cfg(feature = "caldav")]
    pub fn to_caldav_filter(&self) -> String {
        let mut filter = String::from("<C:time-range");

        if let Some(start) = &self.start {
            filter.push_str(&format!(" start=\"{start}\""));
        }

        if let Some(end) = &self.end {
            filter.push_str(&format!(" end=\"{end}\""));
        }

        filter.push_str(" />");
        filter
    }
}

/// The UTC instant of midnight starting `day` in `zone`, as an iCalendar
/// UTC date-time.
fn midnight(day: NaiveDate, zone: &TimeZone) -> Result<String> {
    let date = jiff::civil::Date::new(day.year() as i16, day.month() as i8, day.day() as i8)
        .with_context(|| format!("Cannot read the day {day}"))?;
    let instant = date
        .to_zoned_datetime(zone.clone())
        .with_context(|| format!("Cannot tell the midnight of {day}"))?
        .timestamp();
    Ok(instant.strftime("%Y%m%dT%H%M%SZ").to_string())
}

/// The day a range bound falls on in `zone`, as `YYYYMMDD`: the bound's
/// own leading digits when it does not read as a UTC date-time.
#[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
fn bound_day(bound: &str, zone: &TimeZone) -> String {
    match jiff::fmt::strtime::parse("%Y%m%dT%H%M%SZ", bound).and_then(|tm| tm.to_timestamp()) {
        Ok(instant) => instant
            .to_zoned_datetime(zone.clone())
            .strftime("%Y%m%d")
            .to_string(),
        Err(_) => bound.get(..8).unwrap_or(bound).to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_range_maps_to_an_exclusive_upper_bound() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();

        let range = CalendarTimeRange::from_days_in(Some(from), Some(to), &TimeZone::UTC)
            .unwrap()
            .unwrap();

        assert_eq!(range.start.as_deref(), Some("20260801T000000Z"));
        assert_eq!(range.end.as_deref(), Some("20260901T000000Z"));
    }

    /// CAL-1 and CAL-7: a day is a local one, so an evening west of UTC
    /// stays on its own day.
    #[test]
    fn a_day_range_spans_the_local_days() {
        let zone = TimeZone::get("America/New_York").unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();

        let range = CalendarTimeRange::from_days_in(Some(day), Some(day), &zone)
            .unwrap()
            .unwrap();

        assert_eq!(range.start.as_deref(), Some("20261130T050000Z"));
        assert_eq!(range.end.as_deref(), Some("20261201T050000Z"));
    }

    #[test]
    fn an_absent_range_stays_absent_and_a_crossed_one_is_rejected() {
        assert!(CalendarTimeRange::from_days(None, None).unwrap().is_none());

        let from = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        assert!(CalendarTimeRange::from_days(Some(from), Some(to)).is_err());
    }

    #[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
    #[test]
    fn containment_covers_both_bounds_and_both_stamp_shapes() {
        let range = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: Some("20260901T000000Z".into()),
        };

        let utc = TimeZone::UTC;
        assert!(range.contains_in("20260801T090000Z", &utc));
        assert!(range.contains_in("20260831", &utc));
        assert!(!range.contains_in("20260731T235900Z", &utc));
        assert!(!range.contains_in("20260901T000000Z", &utc));

        let open = CalendarTimeRange {
            start: None,
            end: Some("20260901T000000Z".into()),
        };
        assert!(open.contains_in("19990101", &utc));
    }

    #[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
    #[test]
    fn containment_tells_a_utc_time_as_an_instant_and_a_local_one_by_its_day() {
        let zone = TimeZone::get("America/New_York").unwrap();
        let range = CalendarTimeRange {
            start: Some("20261130T050000Z".into()),
            end: Some("20261201T050000Z".into()),
        };

        assert!(range.contains_in("20261201T010000Z", &zone));
        assert!(!range.contains_in("20261201T050000Z", &zone));
        assert!(range.contains_in("20261130T200000", &zone));
        assert!(range.contains_in("20261130", &zone));
        assert!(!range.contains_in("20261201", &zone));
    }

    /// CAL-16: a stamp a server sent with a multi-byte character at its
    /// eighth byte is compared, never sliced through.
    #[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
    #[test]
    fn a_stamp_with_a_wide_character_is_compared_whole() {
        let range = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: None,
        };

        assert!(range.contains_in("2026080\u{e9}T1", &TimeZone::UTC));
    }

    #[cfg(feature = "caldav")]
    #[test]
    fn the_caldav_filter_omits_the_bounds_it_does_not_carry() {
        let both = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: Some("20260901T000000Z".into()),
        };
        assert_eq!(
            both.to_caldav_filter(),
            "<C:time-range start=\"20260801T000000Z\" end=\"20260901T000000Z\" />"
        );

        let open = CalendarTimeRange {
            start: Some("20260801T000000Z".into()),
            end: None,
        };
        assert_eq!(
            open.to_caldav_filter(),
            "<C:time-range start=\"20260801T000000Z\" />"
        );
    }
}
