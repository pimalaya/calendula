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

use std::{
    env::{self, temp_dir},
    fs,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use io_calendar::item::CalendarItem;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::{account::Account, client::Client};

/// Create a new item.
///
/// This command allows you to add a new vItem to the given
/// addressbook.
#[derive(Debug, Parser)]
pub struct CreateItemCommand {
    /// The identifier of the calendar where the iCalendar should be
    /// added to.
    #[arg(name = "CALENDAR-ID")]
    pub calendar_id: String,
}

impl CreateItemCommand {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        let mut client = Client::new(&account)?;

        let uid = CalendarItem::new_uuid();
        let path = temp_dir().join(format!("{uid}.vcf"));
        let tpl = format!(include_str!("./create.vcf"), uid);
        fs::write(&path, tpl)?;

        let args = env::var("EDITOR")?;
        let mut args = args.split_whitespace();
        let editor = args.next().unwrap();
        let edition = Command::new(editor)
            .args(args)
            .arg(&path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        if !edition.success() {
            let code = edition.code();
            bail!("error while editing vItem: error code {code:?}");
        }

        let content = fs::read_to_string(&path)?
            .replace('\r', "")
            .replace('\n', "\r\n");

        let item = CalendarItem {
            id: CalendarItem::new_uuid().to_string(),
            calendar_id: self.calendar_id,
            ical: CalendarItem::parse(content).context("cannot parse iCal")?,
        };

        client.create_item(item)?;

        printer.out("Item successfully created")
    }
}
