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

use std::{borrow::Cow, collections::HashSet, fmt};

use comfy_table::{presets, Cell, ContentArrangement, Row, Table};
use crossterm::style::Color;
use io_calendar::item::{CalendarItem, ICalendarComponentType, ICalendarProperty, ICalendarValue};
use serde::{ser::Serializer, Deserialize, Serialize};

use crate::table::map_color;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ListEventsTableConfig {
    pub preset: Option<String>,
    pub id_color: Option<Color>,
    pub desc_color: Option<Color>,
    pub begin_color: Option<Color>,
    pub end_color: Option<Color>,
    pub properties: Option<Vec<String>>,
}

impl ListEventsTableConfig {
    pub fn preset(&self) -> &str {
        self.preset.as_deref().unwrap_or(presets::UTF8_FULL)
    }

    pub fn id_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Red))
    }

    pub fn desc_color(&self) -> comfy_table::Color {
        map_color(self.id_color.unwrap_or(Color::Green))
    }

    pub fn begin_color(&self) -> comfy_table::Color {
        map_color(self.begin_color.unwrap_or(Color::Yellow))
    }

    pub fn end_color(&self) -> comfy_table::Color {
        map_color(self.end_color.unwrap_or(Color::Yellow))
    }
}

pub struct EventsTable {
    events: HashSet<CalendarItem>,
    width: Option<u16>,
    config: ListEventsTableConfig,
}

impl EventsTable {
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

    pub fn with_some_desc_color(mut self, color: Option<Color>) -> Self {
        self.config.desc_color = color;
        self
    }

    pub fn with_some_begin_color(mut self, color: Option<Color>) -> Self {
        self.config.begin_color = color;
        self
    }

    pub fn with_some_end_color(mut self, color: Option<Color>) -> Self {
        self.config.end_color = color;
        self
    }
}

impl fmt::Display for EventsTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();

        let headers = vec![
            String::from("ID"),
            String::from("DESC"),
            String::from("BEGIN"),
            String::from("END"),
        ];

        let mut events: Vec<_> = self.events.iter().collect();

        events.sort_by(|a, b| {
            let mut dta = None;
            let mut dtb = None;

            for component in a.components() {
                if component.component_type != ICalendarComponentType::VEvent {
                    continue;
                }

                if let Some(prop) = component.property(&ICalendarProperty::Dtstamp) {
                    for value in &prop.values {
                        if let ICalendarValue::PartialDateTime(pdt) = value {
                            dta = Some(pdt.to_date_time_with_tz(Default::default()).unwrap());
                        }
                    }
                }
            }

            for component in b.components() {
                if component.component_type != ICalendarComponentType::VEvent {
                    continue;
                }

                if let Some(prop) = component.property(&ICalendarProperty::Dtstamp) {
                    for value in &prop.values {
                        if let ICalendarValue::PartialDateTime(pdt) = value {
                            dtb = Some(pdt.to_date_time_with_tz(Default::default()).unwrap());
                        }
                    }
                }
            }

            dta.cmp(&dtb)
        });

        table
            .load_preset(self.config.preset())
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(Row::from(headers))
            .add_rows(events.iter().filter_map(|event| {
                let mut row = Row::new();

                row.add_cell(Cell::new(&event.id).fg(self.config.id_color()));

                let mut summary = None;
                let mut desc = None;
                let mut dtstart = None;
                let mut dtend = None;

                for component in event.components() {
                    if component.component_type != ICalendarComponentType::VEvent {
                        continue;
                    }

                    if let Some(prop) = component.property(&ICalendarProperty::Summary) {
                        for value in &prop.values {
                            if let ICalendarValue::Text(value) = value {
                                summary = Some(Cow::Borrowed(value));
                            }
                        }
                    }

                    if let Some(prop) = component.property(&ICalendarProperty::Description) {
                        for value in &prop.values {
                            if let ICalendarValue::Text(value) = value {
                                desc = Some(Cow::Borrowed(value));
                            }
                        }
                    }

                    if let Some(prop) = component.property(&ICalendarProperty::Dtstart) {
                        for value in &prop.values {
                            if let ICalendarValue::PartialDateTime(pdt) = value {
                                dtstart = Some(
                                    pdt.to_date_time_with_tz(Default::default())
                                        .unwrap()
                                        .to_rfc3339()
                                        .to_string(),
                                );
                            }
                        }
                    }

                    if let Some(prop) = component.property(&ICalendarProperty::Dtend) {
                        for value in &prop.values {
                            if let ICalendarValue::PartialDateTime(pdt) = value {
                                dtend = Some(
                                    pdt.to_date_time_with_tz(Default::default())
                                        .unwrap()
                                        .to_rfc3339()
                                        .to_string(),
                                );
                            }
                        }
                    }
                }

                let summary = summary.or(desc).unwrap_or_default();
                row.add_cell(Cell::new(&summary).fg(self.config.desc_color()));

                row.add_cell(Cell::new(&dtstart.unwrap_or_default()).fg(self.config.begin_color()));
                row.add_cell(Cell::new(&dtend.unwrap_or_default()).fg(self.config.end_color()));

                Some(row)
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

impl Serialize for EventsTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.events.serialize(serializer)
    }
}

impl From<HashSet<CalendarItem>> for EventsTable {
    fn from(events: HashSet<CalendarItem>) -> Self {
        Self {
            events,
            width: Default::default(),
            config: Default::default(),
        }
    }
}
