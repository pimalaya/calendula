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

use anyhow::Result;
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, client::Client};

/// Read the content of a item.
///
/// This command allows you to read the content of a vItem, from the
/// given addressbook.
#[derive(Debug, Parser)]
pub struct ReadItemCommand {
    /// The identifier of the addressbook where the vItem should be
    /// read from.
    #[arg(name = "CALENDAR-ID")]
    pub calendar_id: String,

    /// The identifier of the item that should be read.
    #[arg(name = "ITEM-ID")]
    pub id: String,
}

impl ReadItemCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;
        let item = client.read_item(self.calendar_id, self.id)?;
        printer.out(item.to_string().trim_end())
    }
}
