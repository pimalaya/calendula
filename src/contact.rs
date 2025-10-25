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
    borrow::Cow,
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
};

use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use serde::{
    de::{value::CowStrDeserializer, IntoDeserializer},
    Deserialize, Serialize, Serializer,
};
use vparser::Parser;

use crate::caldav::sans_io::{ListAddressbooksFlow, ListCardsFlow};

/// The main configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    /// The configuration of all the accounts.
    pub accounts: HashMap<String, AccountConfig>,
}

/// The account configuration.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// The defaultness of the current account.
    #[serde(default)]
    pub default: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CaldavConfig {
    /// The Caldav server hostname.
    #[serde(alias = "host")]
    pub hostname: String,

    /// The Caldav server host port.
    pub port: u16,

    /// The addressbooks root url.
    ///
    /// Also known as the addressbook home set, it represents the
    /// common base URL to all addressbooks registered on the Caldav
    /// server by the user being authenticated in this account.
    ///
    /// See [`CaldavConfig::auth`].
    pub url: String,

    /// The HTTP version to use when communicating with the Caldav
    /// server.
    ///
    /// Supported versions: 1.0, 1.1
    #[serde(default)]
    pub http_version: HttpVersion,

    /// The Caldav server authentication configuration.
    ///
    /// Authentication can be done using password or OAuth 2.0.
    #[serde(default, alias = "auth")]
    pub authentication: Authentication,

    /// The Caldav server authentication configuration.
    ///
    /// Authentication can be done using password or OAuth 2.0.
    #[serde(default)]
    pub encryption: Encryption,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum HttpVersion {
    #[serde(rename = "1.0")]
    Http1_0,
    #[default]
    #[serde(rename = "1.1")]
    Http1_1,
}

impl AsRef<str> for HttpVersion {
    fn as_ref(&self) -> &str {
        match self {
            Self::Http1_0 => "1.0",
            Self::Http1_1 => "1.1",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Authentication {
    #[default]
    None,
    Basic(BasicAuthenticationConfig),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Encryption {
    None,
    Rustls(Rustls),
}

impl Default for Encryption {
    fn default() -> Self {
        Self::Rustls(Rustls::default())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Rustls {
    #[serde(default)]
    provider: RustlsProvider,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustlsProvider {
    #[default]
    AwsLc,
    Ring,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BasicAuthenticationConfig {
    #[serde(alias = "user", alias = "login")]
    pub username: String,
    #[serde(alias = "pass")]
    pub password: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Addressbook {
    pub id: String,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub color: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Addressbooks(Vec<Addressbook>);

impl Deref for Addressbooks {
    type Target = Vec<Addressbook>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Addressbooks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListAddressbooksTableConfig {
    pub preset: Option<String>,

    pub id_color: Option<Color>,
    pub name_color: Option<Color>,
    pub desc_color: Option<Color>,
}

impl ListAddressbooksTableConfig {
    pub fn preset(&self) -> &str {
        self.preset.as_deref().unwrap_or(presets::ASCII_MARKDOWN)
    }

    pub fn id_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Red))
    }

    pub fn name_color(&self) -> comfy_table::Color {
        map_color(self.name_color.unwrap_or(Color::Reset))
    }

    pub fn desc_color(&self) -> comfy_table::Color {
        map_color(self.name_color.unwrap_or(Color::Green))
    }
}

pub struct AddressbooksTable {
    addressbooks: Addressbooks,
    width: Option<u16>,
    config: ListAddressbooksTableConfig,
}

impl AddressbooksTable {
    pub fn with_some_width(mut self, width: Option<u16>) -> Self {
        self.width = width;
        self
    }

    pub fn with_some_preset(mut self, preset: Option<String>) -> Self {
        self.config.preset = preset;
        self
    }

    pub fn with_some_id_color(mut self, color: Option<Color>) -> Self {
        self.config.id_color = color;
        self
    }

    pub fn with_some_name_color(mut self, color: Option<Color>) -> Self {
        self.config.name_color = color;
        self
    }

    pub fn with_some_desc_color(mut self, color: Option<Color>) -> Self {
        self.config.desc_color = color;
        self
    }
}

impl From<Addressbooks> for AddressbooksTable {
    fn from(addressbooks: Addressbooks) -> Self {
        Self {
            addressbooks,
            width: None,
            config: Default::default(),
        }
    }
}

impl fmt::Display for AddressbooksTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(self.config.preset())
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(Row::from([
                Cell::new("ID"),
                Cell::new("NAME"),
                Cell::new("DESC"),
                Cell::new("COLOR"),
            ]))
            .add_rows(self.addressbooks.iter().map(|addressbook| {
                let mut row = Row::new();
                row.max_height(1);

                row.add_cell(Cell::new(&addressbook.id).fg(self.config.id_color()));

                if let Some(name) = &addressbook.name {
                    row.add_cell(Cell::new(name).fg(self.config.name_color()));
                } else {
                    row.add_cell(Cell::new(String::new()));
                }

                if let Some(desc) = &addressbook.desc {
                    row.add_cell(Cell::new(desc).fg(self.config.desc_color()));
                } else {
                    row.add_cell(Cell::new(String::new()));
                }

                let mut color_cell = Cell::new("");

                if let Some(color) = &addressbook.color {
                    color_cell = Cell::new(color);

                    // hash tag (1) + rgb hex code (2 + 2 + 2)
                    if color.len() >= 7 {
                        let deserializer: CowStrDeserializer<serde::de::value::Error> =
                            Cow::from(unsafe { color.get_unchecked(..7) }).into_deserializer();

                        if let Ok(rgb) = Color::deserialize(deserializer) {
                            color_cell = color_cell.bg(map_color(rgb));
                        };
                    }
                }

                row.add_cell(color_cell);

                row
            }));

        if let Some(width) = self.width {
            table.set_width(width);
        }

        writeln!(f)?;
        write!(f, "{table}")?;
        writeln!(f)?;
        Ok(())
    }
}

impl Serialize for AddressbooksTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.addressbooks.serialize(serializer)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Card {
    pub id: String,
    pub props: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cards(Vec<Card>);

impl TryFrom<ListCardsFlow> for Cards {
    type Error = quick_xml::DeError;

    fn try_from(flow: ListCardsFlow) -> Result<Self, Self::Error> {
        let mut cards = Vec::new();
        let output = flow.output()?;

        for response in output.responses {
            let id = &response.href.value;

            for propstat in response.propstats {
                if let Some(vcf) = propstat.prop.address_data {
                    let mut card = Card {
                        id: id.clone(),
                        props: Default::default(),
                    };

                    for line in Parser::new(&vcf.value) {
                        let name = line.name().to_string();
                        let value = line.value().to_string();
                        card.props.insert(name, value);
                    }

                    cards.push(card)
                }
            }
        }

        Ok(Self(cards))
    }
}

impl Deref for Cards {
    type Target = Vec<Card>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cards {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn map_color(color: Color) -> comfy_table::Color {
    match color {
        Color::Reset => comfy_table::Color::Reset,
        Color::Black => comfy_table::Color::Black,
        Color::DarkGrey => comfy_table::Color::DarkGrey,
        Color::Red => comfy_table::Color::Red,
        Color::DarkRed => comfy_table::Color::DarkRed,
        Color::Green => comfy_table::Color::Green,
        Color::DarkGreen => comfy_table::Color::DarkGreen,
        Color::Yellow => comfy_table::Color::Yellow,
        Color::DarkYellow => comfy_table::Color::DarkYellow,
        Color::Blue => comfy_table::Color::Blue,
        Color::DarkBlue => comfy_table::Color::DarkBlue,
        Color::Magenta => comfy_table::Color::Magenta,
        Color::DarkMagenta => comfy_table::Color::DarkMagenta,
        Color::Cyan => comfy_table::Color::Cyan,
        Color::DarkCyan => comfy_table::Color::DarkCyan,
        Color::White => comfy_table::Color::White,
        Color::Grey => comfy_table::Color::Grey,
        Color::Rgb { r, g, b } => comfy_table::Color::Rgb { r, g, b },
        Color::AnsiValue(n) => comfy_table::Color::AnsiValue(n),
    }
}
