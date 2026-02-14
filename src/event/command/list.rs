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
use chrono::NaiveDate;
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

    /// End date for filtering events (exclusive, format: YYYY-MM-DD).
    #[arg(long)]
    pub to: Option<NaiveDate>,
}

impl ListEventsCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let time_range = match (self.from, self.to) {
            (None, None) => None,
            (from, to) => Some(TimeRange {
                start: from.map(|d| format!("{}T000000Z", d.format("%Y%m%d"))),
                end: to.map(|d| format!("{}T000000Z", d.format("%Y%m%d"))),
            }),
        };

        let events = match &time_range {
            Some(tr) => client.list_events_in_range(&self.calendar_id, tr)?,
            None => client.list_events(&self.calendar_id)?,
        };

        let table = EventsTable::from(events);
        printer.out(table)
    }
}
