// This file is part of Calendula, a CLI to manage calendars.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
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

use std::process;

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::{
    printer::{Message, Printer},
    prompt,
};

use crate::{account::Account, client::Client};

/// Delete a calendar item.
///
/// This command allows you to delete a iCalendar from a calendar.
#[derive(Debug, Parser)]
pub struct DeleteItemCommand {
    /// The identifier of the calendar where the iCalendar should be
    /// deleted from.
    #[arg(name = "CALENDAR-ID")]
    pub calendar_id: String,

    /// The identifier of the iCalendar to delete.
    #[arg(name = "ITEM-ID")]
    pub id: String,

    #[arg(long, short)]
    pub yes: bool,
}

impl DeleteItemCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        if !self.yes {
            let confirm = "Do you really want to delete this calendar item?";

            if !prompt::bool(confirm, false)? {
                process::exit(0);
            };
        };

        let mut client = Client::new(&account)?;
        client.delete_item(self.calendar_id, self.id)?;
        printer.out(Message::new("Item successfully deleted"))
    }
}
