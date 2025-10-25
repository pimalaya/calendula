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

use std::{collections::HashSet, path::PathBuf};

use anyhow::{anyhow, Result};
use io_calendar::{
    calendar::Calendar,
    item::CalendarItem,
    vdir::coroutines::{
        create_calendar::{CreateCalendar, CreateCalendarResult},
        create_item::{CreateCalendarItem, CreateCalendarItemResult},
        delete_calendar::{DeleteCalendar, DeleteCalendarResult},
        delete_item::{DeleteCalendarItem, DeleteCalendarItemResult},
        list_calendars::{ListCalendars, ListCalendarsResult},
        list_items::{ListCalendarItems, ListCalendarItemsResult},
        read_item::{ReadCalendarItem, ReadCalendarItemResult},
        update_calendar::{UpdateCalendar, UpdateCalendarResult},
        update_item::{UpdateCalendarItem, UpdateCalendarItemResult},
    },
};
use io_fs::runtimes::std::handle;

use super::config::VdirConfig;

#[derive(Debug)]
pub struct VdirClient {
    home_dir: PathBuf,
}

impl VdirClient {
    pub fn new(config: &VdirConfig) -> Self {
        Self {
            home_dir: config.home_dir.to_owned(),
        }
    }

    pub fn create_calendar(&mut self, calendar: Calendar) -> Result<()> {
        let mut create = CreateCalendar::new(&self.home_dir, calendar);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                CreateCalendarResult::Ok => break Ok(()),
                CreateCalendarResult::Err(err) => {
                    return Err(anyhow!(err).context("Creat calendar error"))
                }
                CreateCalendarResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn list_calendars(&mut self) -> Result<HashSet<Calendar>> {
        let mut list = ListCalendars::new(&self.home_dir);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                ListCalendarsResult::Ok(calendars) => break Ok(calendars),
                ListCalendarsResult::Err(err) => {
                    return Err(anyhow!(err).context("List calendars error"))
                }
                ListCalendarsResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn update_calendar(&mut self, calendar: Calendar) -> Result<()> {
        let mut update = UpdateCalendar::new(&self.home_dir, calendar);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                UpdateCalendarResult::Ok => break Ok(()),
                UpdateCalendarResult::Err(err) => {
                    return Err(anyhow!(err).context("Update calendar error"))
                }
                UpdateCalendarResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn delete_calendar(&mut self, id: impl AsRef<str>) -> Result<()> {
        let mut delete = DeleteCalendar::new(&self.home_dir, id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                DeleteCalendarResult::Ok => break Ok(()),
                DeleteCalendarResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete calendar error"))
                }
                DeleteCalendarResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn create_item(&mut self, item: CalendarItem) -> Result<()> {
        let mut create = CreateCalendarItem::new(&self.home_dir, item);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                CreateCalendarItemResult::Ok => break Ok(()),
                CreateCalendarItemResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete calendar error"))
                }
                CreateCalendarItemResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn list_items(&mut self, calendar_id: impl AsRef<str>) -> Result<HashSet<CalendarItem>> {
        let mut list = ListCalendarItems::new(&self.home_dir, calendar_id);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                ListCalendarItemsResult::Ok(ok) => break Ok(ok),
                ListCalendarItemsResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete calendar error"))
                }
                ListCalendarItemsResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn read_item(
        &mut self,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<CalendarItem> {
        let mut read = ReadCalendarItem::new(&self.home_dir, calendar_id, item_id);
        let mut arg = None;

        loop {
            match read.resume(arg.take()) {
                ReadCalendarItemResult::Ok(item) => break Ok(item),
                ReadCalendarItemResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete calendar error"))
                }
                ReadCalendarItemResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn update_item(&mut self, item: CalendarItem) -> Result<()> {
        let mut update = UpdateCalendarItem::new(&self.home_dir, item);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                UpdateCalendarItemResult::Ok => break Ok(()),
                UpdateCalendarItemResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete calendar error"))
                }
                UpdateCalendarItemResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }

    pub fn delete_item(
        &mut self,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<()> {
        let mut delete = DeleteCalendarItem::new(&self.home_dir, calendar_id, item_id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                DeleteCalendarItemResult::Ok => break Ok(()),
                DeleteCalendarItemResult::Err(err) => {
                    return Err(anyhow!(err).context("Delete calendar error"))
                }
                DeleteCalendarItemResult::Io(io) => arg = Some(handle(io)?),
            }
        }
    }
}
