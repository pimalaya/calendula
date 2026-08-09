//! iCalendar item types shared by every backend, plus the `item`
//! command family built on them.
//!
//! [`CalendarItem`] is the raw view of a calendar object resource: an
//! id, an optional entity tag, and the iCalendar bytes verbatim.
//! calendula never rewrites those bytes, so what a backend stored is
//! what a read returns. [`CalendarTimeRange`] narrows a listing to a
//! window; CalDAV pushes it server-side while the local backends apply
//! it after parsing.

pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod update;

use anyhow::{Result, bail};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// A calendar object resource: one iCalendar file.
///
/// The contents mix component kinds (VEVENT, VTODO, VJOURNAL); the
/// `event` command family filters them, the `item` family does not.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarItem {
    /// Backend-specific identifier: the file stem (vdir), the last
    /// path segment of the resource href verbatim (CalDAV), or the
    /// store-assigned public id (pimdir).
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

/// An inclusive day window narrowing a listing.
///
/// Both bounds are optional, so a range may be open on either side.
/// The stored values are iCalendar UTC date-times (RFC 5545 3.3.5), the
/// form the CalDAV `time-range` filter takes (RFC 4791 9.9).
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarTimeRange {
    /// Inclusive lower bound, as `YYYYMMDDTHHMMSSZ`.
    #[serde(default)]
    pub start: Option<String>,
    /// Exclusive upper bound, as `YYYYMMDDTHHMMSSZ`.
    #[serde(default)]
    pub end: Option<String>,
}

impl CalendarTimeRange {
    /// Builds a range from inclusive `--from` / `--to` days, mapping
    /// `to` onto the exclusive upper bound the wire format wants (the
    /// day after, at midnight). Returns `None` when both bounds are
    /// absent, and bails when they cross.
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

    /// Whether `stamp` (an iCalendar date or date-time, in any time
    /// zone form) falls inside the range. Comparison is lexicographic
    /// on the leading `YYYYMMDD`, which is enough for a day window and
    /// keeps the CLI out of time-zone resolution.
    ///
    /// The local backends need it to filter a listing, and so do the
    /// `todo` and `journal` families whatever the backend: a
    /// server-side range filter is defined against a component's start
    /// and end (RFC 4791 9.9), which a todo and a journal entry do not
    /// both carry.
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

    /// The CalDAV `time-range` element (RFC 4791 9.9) for this range,
    /// nested inside a component filter by the caller.
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
        // The upper bound is exclusive on the wire, so the whole of the
        // 31st stays inside the window.
        assert_eq!(range.end.as_deref(), Some("20260901T000000Z"));
    }

    #[test]
    fn an_absent_range_stays_absent_and_a_crossed_one_is_rejected() {
        assert!(CalendarTimeRange::from_days(None, None).unwrap().is_none());

        let from = NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        assert!(CalendarTimeRange::from_days(Some(from), Some(to)).is_err());
    }

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
