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

use std::{borrow::Cow, collections::HashSet};

use anyhow::{anyhow, Result};
use http::Uri;

use io_calendar::{
    caldav::{
        coroutines::{
            calendar_home_set::CalendarHomeSet,
            create_calendar::CreateCalendar,
            create_item::CreateCalendarItem,
            current_user_principal::CurrentUserPrincipal,
            delete_calendar::DeleteCalendar,
            delete_item::DeleteCalendarItem,
            follow_redirects::FollowRedirectsResult,
            list_calendars::ListCalendars,
            list_items::ListCalendarItems,
            read_item::ReadCalendarItem,
            send::SendResult,
            update_calendar::UpdateCalendar,
            update_item::UpdateCalendarItem,
            well_known::{WellKnown, WellKnownResult},
        },
        request::set_uri_path,
    },
    calendar::Calendar,
    item::CalendarItem,
};
use io_stream::runtimes::std::handle;
use pimalaya_toolbox::stream::Stream;

use super::config::CaldavConfig;

#[derive(Debug)]
pub struct CaldavClient<'a> {
    config: io_calendar::caldav::config::CaldavConfig<'a>,
    stream: Stream,
}

impl<'a> CaldavClient<'a> {
    pub fn new(config: &'a CaldavConfig) -> Result<Self> {
        let tls = &config.tls;

        if let Some(uri) = &config.home_uri {
            let stream = Stream::connect(uri, tls)?;
            return Self::from_home_uri(config, stream, Cow::Borrowed(uri));
        };

        if let Some(uri) = &config.server_uri {
            let stream = Stream::connect(&uri, tls)?;
            return Self::from_server_uri(config, stream, uri.clone());
        }

        if let Some(discover) = &config.discover {
            let hostname = if let Some(port) = discover.port {
                Cow::from(format!("{}:{port}", discover.host))
            } else {
                Cow::from(&discover.host)
            };

            let scheme = match &discover.scheme {
                Some(scheme) => Cow::from(scheme),
                None => Cow::from("https"),
            };

            let uri: Uri = format!("{scheme}://{hostname}/.well-known/caldav")
                .parse()
                .unwrap();

            let mut stream = Stream::connect(&uri, tls)?;

            let remote_config = io_calendar::caldav::config::CaldavConfig {
                uri: Cow::Borrowed(&uri),
                auth: TryFrom::try_from(&config.auth)?,
            };

            let mut well_known = WellKnown::new(&remote_config, discover.method.clone());
            let mut arg = None;

            let ok = loop {
                match well_known.resume(arg.take()) {
                    WellKnownResult::Ok(ok) => break ok,
                    WellKnownResult::Err(err) => {
                        return Err(anyhow!(err).context("Discover Caldav server error"));
                    }
                    WellKnownResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                }
            };

            if !ok.keep_alive {
                stream = Stream::connect(&ok.uri, tls)?;
            }

            return Self::from_server_uri(config, stream, ok.uri);
        }

        let ctx = "Cannot discover Caldav home URI";
        let err = "Missing one of `discover`, `server-uri` or `home-uri` config option";
        Err(anyhow!(err).context(ctx))
    }

    fn from_home_uri(config: &'a CaldavConfig, stream: Stream, uri: Cow<'a, Uri>) -> Result<Self> {
        let auth = TryFrom::try_from(&config.auth)?;
        let config = io_calendar::caldav::config::CaldavConfig { uri, auth };
        let client = Self { config, stream };

        return Ok(client);
    }

    fn from_server_uri(config: &'a CaldavConfig, mut stream: Stream, mut uri: Uri) -> Result<Self> {
        let tls = &config.tls;

        let remote_config = io_calendar::caldav::config::CaldavConfig {
            uri: Cow::Borrowed(&uri),
            auth: TryFrom::try_from(&config.auth)?,
        };

        let mut principal = CurrentUserPrincipal::new(&remote_config);
        let mut arg = None;

        let ok = loop {
            match principal.resume(arg.take()) {
                FollowRedirectsResult::Ok(ok) => break ok,
                FollowRedirectsResult::Err(err) => {
                    return Err(anyhow!(err).context("Get current user principal error"))
                }
                FollowRedirectsResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                FollowRedirectsResult::Reset(new_uri) => {
                    uri = new_uri;
                    stream = Stream::connect(&uri, tls)?;
                }
            }
        };

        let mut same_scheme = true;
        let mut same_authority = true;

        if let Some(discovered_uri) = ok.body {
            uri = if let Some(auth) = discovered_uri.authority() {
                same_authority = uri.authority() == Some(auth);
                same_scheme = uri.scheme() == discovered_uri.scheme();
                discovered_uri
            } else {
                set_uri_path(uri, discovered_uri.path())
            };
        }

        if !ok.keep_alive || !same_scheme || !same_authority {
            stream = Stream::connect(&uri, tls)?;
        }

        let remote_config = io_calendar::caldav::config::CaldavConfig {
            uri: Cow::Borrowed(&uri),
            auth: TryFrom::try_from(&config.auth)?,
        };

        let mut home = CalendarHomeSet::new(&remote_config);
        let mut arg = None;

        let ok = loop {
            match home.resume(arg.take()) {
                FollowRedirectsResult::Ok(ok) => break ok,
                FollowRedirectsResult::Err(err) => {
                    return Err(anyhow!(err).context("Get calendar home set error"));
                }
                FollowRedirectsResult::Io(io) => arg = Some(handle(&mut stream, io)?),
                FollowRedirectsResult::Reset(new_uri) => {
                    uri = new_uri;
                    stream = Stream::connect(&uri, tls)?;
                }
            }
        };

        let mut same_scheme = true;
        let mut same_authority = true;

        if let Some(discovered_uri) = ok.body {
            uri = if let Some(auth) = discovered_uri.authority() {
                same_authority = uri.authority() == Some(auth);
                same_scheme = uri.scheme() == discovered_uri.scheme();
                discovered_uri
            } else {
                set_uri_path(uri, discovered_uri.path())
            };
        }

        if !ok.keep_alive || !same_scheme || !same_authority {
            stream = Stream::connect(&uri, tls)?;
        }

        Self::from_home_uri(config, stream, Cow::Owned(uri))
    }

    pub fn create_calendar(&mut self, calendar: Calendar) -> Result<()> {
        let mut create = CreateCalendar::new(&self.config, calendar);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Creat calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn list_calendars(&mut self) -> Result<HashSet<Calendar>> {
        let mut list = ListCalendars::new(&self.config);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => return Err(anyhow!(err).context("List calendars error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn update_calendar(&mut self, calendar: Calendar) -> Result<()> {
        let mut update = UpdateCalendar::new(&self.config, calendar);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => return Err(anyhow!(err).context("Update calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn delete_calendar(&mut self, id: impl AsRef<str>) -> Result<()> {
        let mut delete = DeleteCalendar::new(&self.config, id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn create_item(&mut self, item: CalendarItem) -> Result<()> {
        let mut create = CreateCalendarItem::new(&self.config, item);
        let mut arg = None;

        loop {
            match create.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn list_items(&mut self, calendar_id: impl AsRef<str>) -> Result<HashSet<CalendarItem>> {
        let mut list = ListCalendarItems::new(&self.config, calendar_id);
        let mut arg = None;

        loop {
            match list.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn read_item(
        &mut self,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<CalendarItem> {
        let mut read = ReadCalendarItem::new(&self.config, calendar_id, item_id);
        let mut arg = None;

        loop {
            match read.resume(arg.take()) {
                SendResult::Ok(ok) => break Ok(ok.body),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn update_item(&mut self, item: CalendarItem) -> Result<()> {
        let mut update = UpdateCalendarItem::new(&self.config, item);
        let mut arg = None;

        loop {
            match update.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }

    pub fn delete_item(
        &mut self,
        calendar_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<()> {
        let mut delete = DeleteCalendarItem::new(&self.config, calendar_id, item_id);
        let mut arg = None;

        loop {
            match delete.resume(arg.take()) {
                SendResult::Ok(_) => break Ok(()),
                SendResult::Err(err) => return Err(anyhow!(err).context("Delete calendar error")),
                SendResult::Io(io) => arg = Some(handle(&mut self.stream, io)?),
            }
        }
    }
}
