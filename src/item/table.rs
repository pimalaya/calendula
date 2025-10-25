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

use std::{collections::HashSet, fmt};

use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use io_calendar::item::CalendarItem;
use serde::{ser::Serializer, Deserialize, Serialize};

use crate::table::map_color;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListItemsTableConfig {
    pub preset: Option<String>,
    pub id_color: Option<Color>,
    pub version_color: Option<Color>,
    pub properties: Option<Vec<String>>,
}

impl ListItemsTableConfig {
    pub fn preset(&self) -> &str {
        self.preset.as_deref().unwrap_or(presets::UTF8_FULL)
    }

    pub fn id_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Red))
    }

    pub fn components_color(&self) -> comfy_table::Color {
        map_color(self.version_color.unwrap_or(Color::Blue))
    }
}

pub struct ItemsTable {
    items: HashSet<CalendarItem>,
    width: Option<u16>,
    config: ListItemsTableConfig,
}

impl ItemsTable {
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

    pub fn with_some_version_color(mut self, color: Option<Color>) -> Self {
        self.config.version_color = color;
        self
    }
}

impl fmt::Display for ItemsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        let headers = vec![String::from("ID"), String::from("COMPONENTS")];

        table
            .load_preset(self.config.preset())
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(Row::from(headers))
            .add_rows(self.items.iter().map(|item| {
                let mut row = Row::new();
                // row.max_height(1);

                row.add_cell(Cell::new(&item.id).fg(self.config.id_color()));

                let mut glue = "";
                let mut types = String::new();

                for component in item.components() {
                    types.push_str(glue);
                    types.push_str(component.component_type.as_str());
                    glue = ", ";
                }

                row.add_cell(Cell::new(&types).fg(self.config.components_color()));

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

impl Serialize for ItemsTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.items.serialize(serializer)
    }
}

impl From<HashSet<CalendarItem>> for ItemsTable {
    fn from(items: HashSet<CalendarItem>) -> Self {
        Self {
            items,
            width: Default::default(),
            config: Default::default(),
        }
    }
}
