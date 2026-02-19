// This file is part of Calendula, a CLI to manage calendars.
//
// Copyright (C) 2025-2026 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use anyhow::Result;
use chrono::{Days, NaiveDate};
use clap::Parser;
use io_calendar::caldav::TimeRange;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, client::Client, event::table::EventsTable};

/// List all events.
///
/// This command allows you to list iCalendars from a given calendar.
/// Use --from and --to to filter events by date range (server-side).
#[derive(Debug, Parser)]
pub struct ListEventsCommand {
    /// The identifier of the CalDAV calendar to list iCalendars from.
    #[arg(name = "CALENDAR-ID")]
    pub calendar_id: String,

    /// Start date for filtering events (inclusive, format: YYYY-MM-DD).
    #[arg(long)]
    pub from: Option<NaiveDate>,

    /// End date for filtering events (inclusive, format: YYYY-MM-DD).
    #[arg(long)]
    pub to: Option<NaiveDate>,
}

/// Build a TimeRange from optional inclusive from/to dates.
///
/// The `to` date is inclusive: it gets shifted forward by one day to produce
/// an exclusive end bound for CalDAV's time-range filter.
fn build_time_range(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<Option<TimeRange>> {
    match (from, to) {
        (None, None) => Ok(None),
        (from, to) => {
            let fmt = |d: NaiveDate| format!("{}T000000Z", d.format("%Y%m%d"));
            // --to is inclusive: shift to the next day for the CalDAV end bound
            let end = match to {
                Some(d) => Some(
                    d.checked_add_days(Days::new(1))
                        .ok_or_else(|| anyhow::anyhow!("--to date is out of range"))?,
                ),
                None => None,
            };
            TimeRange::new(
                from.map(|d| fmt(d)).as_deref(),
                end.map(|d| fmt(d)).as_deref(),
            )
            .ok_or_else(|| anyhow::anyhow!("invalid date format for --from/--to"))
            .map(Some)
        }
    }
}

impl ListEventsCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;
        let time_range = build_time_range(self.from, self.to)?;

        let events = match &time_range {
            Some(tr) => client.list_events_in_range(&self.calendar_id, tr)?,
            None => client.list_events(&self.calendar_id)?,
        };

        let table = EventsTable::from(events);
        printer.out(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn both_none_returns_none() {
        assert!(build_time_range(None, None).unwrap().is_none());
    }

    #[test]
    fn from_only() {
        let tr = build_time_range(Some(date(2026, 2, 14)), None).unwrap().unwrap();
        assert_eq!(tr.start(), Some("20260214T000000Z"));
        assert_eq!(tr.end(), None);
    }

    #[test]
    fn to_only_is_inclusive() {
        let tr = build_time_range(None, Some(date(2026, 2, 21))).unwrap().unwrap();
        assert_eq!(tr.start(), None);
        // --to 2026-02-21 should produce end of 2026-02-22 (next day)
        assert_eq!(tr.end(), Some("20260222T000000Z"));
    }

    #[test]
    fn both_from_and_to() {
        let tr = build_time_range(Some(date(2026, 2, 14)), Some(date(2026, 2, 21)))
            .unwrap()
            .unwrap();
        assert_eq!(tr.start(), Some("20260214T000000Z"));
        assert_eq!(tr.end(), Some("20260222T000000Z"));
    }

    #[test]
    fn to_at_month_boundary() {
        let tr = build_time_range(None, Some(date(2026, 1, 31))).unwrap().unwrap();
        assert_eq!(tr.end(), Some("20260201T000000Z"));
    }

    #[test]
    fn to_at_year_boundary() {
        let tr = build_time_range(None, Some(date(2026, 12, 31))).unwrap().unwrap();
        assert_eq!(tr.end(), Some("20270101T000000Z"));
    }
}
