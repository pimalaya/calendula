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

// mod create;
// mod delete;
mod agenda;
mod list;
// mod read;
// mod update;

use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

use self::{
    agenda::AgendaCommand,
    // read::ReadEventCommand,
    // update::UpdateEventCommand,
    // create::CreateEventCommand,
    // delete::DeleteEventCommand,
    list::ListEventsCommand,
};

/// Create, list, update and delete calendar events.
///
/// This subcommand allows you to create, list, update and delete
/// events from calendars.
#[derive(Debug, Subcommand)]
pub enum EventSubcommand {
    // #[command(alias = "new", alias = "add")]
    // Create(CreateEventCommand),
    // #[command(alias = "get")]
    // Read(ReadEventCommand),
    #[command(alias = "cal")]
    Agenda(AgendaCommand),
    #[command(alias = "lst")]
    List(ListEventsCommand),
    // #[command(alias = "set", alias = "change")]
    // Update(UpdateEventCommand),
    // #[command(alias = "remove", alias = "rm")]
    // Delete(DeleteEventCommand),
}

impl EventSubcommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            // Self::Create(cmd) => cmd.execute(printer, account),
            // Self::Read(cmd) => cmd.execute(printer, account),
            Self::Agenda(cmd) => cmd.execute(printer, account),
            Self::List(cmd) => cmd.execute(printer, account),
            // Self::Update(cmd) => cmd.execute(printer, account),
            // Self::Delete(cmd) => cmd.execute(printer, account),
        }
    }
}
