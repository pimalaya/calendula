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

use std::{borrow::Cow, collections::HashSet, fmt};

use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use io_calendar::calendar::Calendar;
use serde::{
    de::{value::CowStrDeserializer, IntoDeserializer},
    Deserialize, Serialize, Serializer,
};

use crate::table::map_color;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListCalendarsTableConfig {
    pub preset: Option<String>,

    pub id_color: Option<Color>,
    pub name_color: Option<Color>,
    pub desc_color: Option<Color>,
}

impl ListCalendarsTableConfig {
    pub fn preset(&self) -> &str {
        self.preset.as_deref().unwrap_or(presets::UTF8_FULL)
    }

    pub fn id_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Red))
    }

    pub fn name_color(&self) -> comfy_table::Color {
        map_color(self.name_color.unwrap_or(Color::Green))
    }

    pub fn desc_color(&self) -> comfy_table::Color {
        map_color(self.desc_color.unwrap_or(Color::Reset))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarsTable {
    calendars: Vec<Calendar>,
    width: Option<u16>,
    config: ListCalendarsTableConfig,
}

impl CalendarsTable {
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

impl From<HashSet<Calendar>> for CalendarsTable {
    fn from(calendars: HashSet<Calendar>) -> Self {
        let mut calendars: Vec<_> = calendars.into_iter().collect();
        calendars.sort_by(|a, b| a.display_name.cmp(&b.display_name));

        Self {
            calendars,
            width: None,
            config: Default::default(),
        }
    }
}

impl fmt::Display for CalendarsTable {
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
            .add_rows(self.calendars.iter().map(|calendar| {
                let mut row = Row::new();
                row.max_height(1);

                row.add_cell(Cell::new(&calendar.id).fg(self.config.id_color()));

                if let Some(name) = &calendar.display_name {
                    row.add_cell(Cell::new(name).fg(self.config.name_color()));
                } else {
                    row.add_cell(Cell::new(String::new()));
                }

                if let Some(desc) = &calendar.description {
                    row.add_cell(Cell::new(desc).fg(self.config.desc_color()));
                } else {
                    row.add_cell(Cell::new(String::new()));
                }

                let mut color_cell = Cell::new("");

                if let Some(color) = &calendar.color {
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

impl Serialize for CalendarsTable {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.calendars.serialize(serializer)
    }
}
