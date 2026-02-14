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

use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};
use io_calendar::{caldav::TimeRange, calendar::Calendar, item::CalendarItem};

use crate::account::Account;
#[cfg(feature = "caldav")]
use crate::caldav::client::CaldavClient;
#[cfg(feature = "vdir")]
use crate::vdir::client::VdirClient;

#[derive(Debug, Default)]
pub enum Client<'a> {
    #[default]
    None,
    #[cfg(feature = "caldav")]
    Caldav(CaldavClient<'a>),
    #[cfg(feature = "vdir")]
    Vdir(VdirClient),
}

impl<'a> Client<'a> {
    pub fn new(account: &'a Account) -> Result<Self> {
        #[cfg(feature = "caldav")]
        if let Some(config) = &account.caldav {
            return Ok(Self::Caldav(CaldavClient::new(config)?));
        }

        #[cfg(feature = "vdir")]
        if let Some(config) = &account.vdir {
            return Ok(Self::Vdir(VdirClient::new(config)));
        }

        Err(anyhow!("Cannot find Caldav nor Vdir config").context("Create calendar client error"))
    }

    pub fn create_calendar(&mut self, calendar: Calendar) -> Result<()> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.create_calendar(calendar),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.create_calendar(calendar),
        }
    }

    pub fn list_calendars(&mut self) -> Result<HashSet<Calendar>> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.list_calendars(),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.list_calendars(),
        }
    }

    pub fn list_items(&mut self, calendar_id: impl AsRef<str>) -> Result<HashSet<CalendarItem>> {
        match self {
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.list_items(calendar_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.list_items(calendar_id),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn list_events(&mut self, calendar_id: impl AsRef<str>) -> Result<HashSet<CalendarItem>> {
        match self {
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.list_events(calendar_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.list_items(calendar_id),
            Self::None => bail!("client not defined"),
        }
    }

    pub fn list_events_in_range(
        &mut self,
        calendar_id: impl AsRef<str>,
        time_range: &TimeRange,
    ) -> Result<HashSet<CalendarItem>> {
        match self {
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.list_events_in_range(calendar_id, time_range),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => {
                log::warn!("vdir backend does not support date filtering, showing all events");
                client.list_items(calendar_id)
            }
            Self::None => bail!("client not defined"),
        }
    }

    pub fn update_calendar(&mut self, calendar: Calendar) -> Result<()> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.update_calendar(calendar),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.update_calendar(calendar),
        }
    }

    pub fn delete_calendar(&mut self, id: impl AsRef<str>) -> Result<()> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.delete_calendar(id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.delete_calendar(id),
        }
    }

    pub fn create_item(&mut self, item: CalendarItem) -> Result<()> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.create_item(item),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.create_item(item),
        }
    }

    pub fn read_item(
        &mut self,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<CalendarItem> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.read_item(calendar_id, item_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.read_item(calendar_id, item_id),
        }
    }

    pub fn update_item(&mut self, item: CalendarItem) -> Result<()> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.update_item(item),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.update_item(item),
        }
    }

    pub fn delete_item(
        &mut self,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<()> {
        match self {
            Self::None => bail!("Missing calendar backend"),
            #[cfg(feature = "caldav")]
            Self::Caldav(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "vdir")]
            Self::Vdir(client) => client.delete_item(calendar_id, item_id),
        }
    }
}
