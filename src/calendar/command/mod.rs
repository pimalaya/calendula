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

mod create;
mod delete;
mod list;
mod update;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

use self::{
    create::CreateCalendarCommand, delete::DeleteCalendarCommand, list::ListCalendarsCommand,
    update::UpdateCalendarCommand,
};

/// Create, list, update and delete calendars.
///
/// This subcommand allows you to create, list, update and delete
/// calendars.
#[derive(Debug, Subcommand)]
pub enum CalendarSubcommand {
    #[command(alias = "new", alias = "add")]
    Create(CreateCalendarCommand),
    List(ListCalendarsCommand),
    #[command(alias = "set")]
    Update(UpdateCalendarCommand),
    #[command(alias = "remove", alias = "rm")]
    Delete(DeleteCalendarCommand),
}

impl CalendarSubcommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Create(cmd) => cmd.execute(printer, account),
            Self::List(cmd) => cmd.execute(printer, account),
            Self::Update(cmd) => cmd.execute(printer, account),
            Self::Delete(cmd) => cmd.execute(printer, account),
        }
    }
}
