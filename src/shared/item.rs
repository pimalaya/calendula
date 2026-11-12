//! # Item
//!
//! The iCalendar object resource shared by every backend, and the `item`
//! command family built on it.
//!
//! [`CalendarItem`] is the raw view: an id, an optional entity tag, and the
//! bytes verbatim. calendula never rewrites those bytes, so what a backend
//! stored is what a read returns.
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

use anyhow::{Result, bail};
use chrono::NaiveDate;
use ical::component::IcalComponentKind;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(any(feature = "vdir", feature = "msgraph"))]
use crate::shared::event::Event;

/// A calendar object resource: one iCalendar file.
///
/// The contents mix component kinds (VEVENT, VTODO, VJOURNAL): the `event`
/// family filters them, the `item` family does not.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CalendarItem {
    /// Backend-specific identifier the other commands address it by.
    pub id: String,
    /// The calendar the item lives in.
    pub calendar_id: String,
    /// Entity tag, for the backends offering optimistic concurrency.
    #[serde(default)]
    pub etag: Option<String>,
    /// Raw iCalendar bytes, exactly as the backend stored them.
    #[serde(default)]
    pub contents: Vec<u8>,
}

#[cfg(any(feature = "vdir", feature = "msgraph"))]
impl CalendarItem {
    /// Whether one of the item's events starts within `range`, or one of
    /// its occurrences overlaps it, every item passing when none is named.
    ///
    /// The filter of a backend that cannot narrow a listing server-side.
    /// An item carrying no event is kept only when no range was asked for,
    /// so a filtered listing never shows an undated resource.
    pub fn starts_within(&self, range: Option<&CalendarTimeRange>) -> bool {
        let Some(range) = range else {
            return true;
        };

        Event::project(self)
            .iter()
            .any(|event| range.contains(&event.start))
            || !Event::occurrences(self, Some(range)).is_empty()
    }
}

/// One page of an item listing, and whether an item follows it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CalendarItemPage {
    /// The items of the page, in the order the backend lists them.
    pub items: Vec<CalendarItem>,
    /// Whether the listing holds an item past the page.
    pub more: bool,
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
    #[cfg_attr(not(feature = "pimdir"), allow(dead_code))]
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
    /// Builds a range from the inclusive `--from` and `--to` days.
    ///
    /// `to` maps onto the exclusive upper bound the wire format wants, the
    /// day after at midnight. Two absent bounds give `None`, and crossed
    /// ones bail.
    pub fn from_days(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Option<Self>> {
        if let (Some(from), Some(to)) = (from, to)
            && to < from
        {
            bail!("Invalid time range: `--to` {to} is before `--from` {from}");
        }

        if from.is_none() && to.is_none() {
            return Ok(None);
        }

        let end = to
            .and_then(|day| day.succ_opt())
            .map(|day| stamp(day, "000000"));

        Ok(Some(Self {
            start: from.map(|day| stamp(day, "000000")),
            end,
        }))
    }

    /// Whether an iCalendar date or date-time falls inside the range.
    ///
    /// Comparison is lexicographic on the leading `YYYYMMDD`, enough for a
    /// day window and keeping the CLI out of time-zone resolution.
    #[cfg(any(feature = "vdir", feature = "msgraph", feature = "pimdir"))]
    pub fn contains(&self, stamp: &str) -> bool {
        let day = &stamp[..stamp.len().min(8)];

        if let Some(start) = &self.start
            && day < &start[..8]
        {
            return false;
        }

        if let Some(end) = &self.end
            && day >= &end[..8]
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

/// Formats `day` as an iCalendar UTC date-time at `time` (`HHMMSS`).
fn stamp(day: NaiveDate, time: &str) -> String {
    format!("{}T{time}Z", day.format("%Y%m%d"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_range_maps_to_an_exclusive_upper_bound() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();

        let range = CalendarTimeRange::from_days(Some(from), Some(to))
            .unwrap()
            .unwrap();

        assert_eq!(range.start.as_deref(), Some("20260801T000000Z"));
        // NOTE: the upper bound is exclusive on the wire, so the whole of
        // the 31st stays inside the window.
        assert_eq!(range.end.as_deref(), Some("20260901T000000Z"));
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

        assert!(range.contains("20260801T090000Z"));
        assert!(range.contains("20260831"));
        assert!(!range.contains("20260731T235900Z"));
        assert!(!range.contains("20260901T000000Z"));

        let open = CalendarTimeRange {
            start: None,
            end: Some("20260901T000000Z".into()),
        };
        assert!(open.contains("19990101"));
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
