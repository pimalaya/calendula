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

use std::{borrow::Cow, fmt};

use anyhow::Error;
use http::{Method, Uri};
use pimalaya_toolbox::secret::Secret;
#[cfg(feature = "caldav")]
use pimalaya_toolbox::stream::Tls;
use serde::{de::Visitor, Deserialize, Deserializer};

/// The account configuration.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CaldavConfig {
    pub discover: Option<CaldavDiscoverConfig>,
    #[serde(default, deserialize_with = "uri")]
    pub server_uri: Option<Uri>,
    #[serde(default, deserialize_with = "uri")]
    pub home_uri: Option<Uri>,
    #[serde(default)]
    pub auth: Auth,
    #[serde(default)]
    pub tls: Tls,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CaldavDiscoverConfig {
    pub host: String,
    pub port: Option<u16>,
    pub scheme: Option<String>,
    #[serde(default, deserialize_with = "method")]
    pub method: Option<Method>,
}

struct UriVisitor;

impl<'de> Visitor<'de> for UriVisitor {
    type Value = Uri;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an URI string")
    }

    fn visit_str<E: serde::de::Error>(self, uri: &str) -> Result<Self::Value, E> {
        match uri.parse() {
            Ok(uri) => Ok(uri),
            Err(err) => Err(serde::de::Error::custom(err)),
        }
    }
}

struct MethodVisitor;

impl<'de> Visitor<'de> for MethodVisitor {
    type Value = Method;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a HTTP method string")
    }

    fn visit_str<E: serde::de::Error>(self, method: &str) -> Result<Self::Value, E> {
        match method.parse() {
            Ok(method) => Ok(method),
            Err(err) => Err(serde::de::Error::custom(err)),
        }
    }
}

pub fn uri<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Uri>, D::Error> {
    deserializer.deserialize_str(UriVisitor).map(Some)
}

pub fn method<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Method>, D::Error> {
    deserializer.deserialize_str(MethodVisitor).map(Some)
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Auth {
    #[default]
    Plain,
    Basic {
        username: String,
        password: Secret,
    },
    Bearer(Secret),
}

impl<'a> TryFrom<&'a Auth> for io_calendar::caldav::config::CaldavAuth<'a> {
    type Error = Error;

    fn try_from(auth: &'a Auth) -> Result<Self, Self::Error> {
        Ok(match auth {
            Auth::Plain => io_calendar::caldav::config::CaldavAuth::Plain,
            Auth::Basic { username, password } => io_calendar::caldav::config::CaldavAuth::Basic {
                username: username.into(),
                password: Cow::Owned(password.get()?),
            },
            Auth::Bearer(token) => io_calendar::caldav::config::CaldavAuth::Bearer {
                token: Cow::Owned(token.get()?),
            },
        })
    }
}
