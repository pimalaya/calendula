//! Merged runtime account: the DTO every command consumes.
//!
//! Built by the dispatch layer (`crate::cli`) by folding the global
//! [`Config`] then the selected `[accounts.<name>]` block via
//! [`Account::merge`]. Defaults are applied at consumption time by
//! the `*` accessor methods.

use std::{env::temp_dir, path::PathBuf};

use anyhow::{Result, bail};
use comfy_table::{Color as TableColor, ContentArrangement, presets};
use crossterm::style::Color;
use dirs::download_dir;

use crate::config::{
    AccountConfig, CalendarListTableConfig, Config, EventListTableConfig, ItemListTableConfig,
    TableArrangementConfig,
};

const DEFAULT_LIST_PAGE_SIZE: u32 = 25;

#[derive(Debug, Default)]
pub struct Account {
    pub downloads_dir: Option<PathBuf>,
    pub table_preset: Option<String>,
    pub table_arrangement: Option<TableArrangementConfig>,

    pub events_list_page_size: Option<u32>,
    pub items_list_page_size: Option<u32>,

    /// Fallback calendar id for `event` and `item` commands when their
    /// `-k/--calendar` flag is omitted.
    pub calendar_default: Option<String>,

    pub calendars_list_table: CalendarListTableConfig,
    pub events_list_table: EventListTableConfig,
    pub items_list_table: ItemListTableConfig,
}

impl Account {
    /// Folds `other`'s set fields on top of `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            downloads_dir: other.downloads_dir.or(self.downloads_dir),
            table_preset: other.table_preset.or(self.table_preset),
            table_arrangement: other.table_arrangement.or(self.table_arrangement),

            events_list_page_size: other.events_list_page_size.or(self.events_list_page_size),
            items_list_page_size: other.items_list_page_size.or(self.items_list_page_size),

            calendar_default: other.calendar_default.or(self.calendar_default),

            calendars_list_table: merge_calendar_table(
                self.calendars_list_table,
                other.calendars_list_table,
            ),
            events_list_table: merge_event_table(self.events_list_table, other.events_list_table),
            items_list_table: merge_item_table(self.items_list_table, other.items_list_table),
        }
    }

    /// Effective downloads directory.
    #[allow(dead_code)]
    pub fn downloads_dir(&self) -> PathBuf {
        self.downloads_dir
            .as_ref()
            .and_then(|dir| dir.to_str())
            .and_then(|dir| shellexpand::full(dir).ok())
            .map(|dir| PathBuf::from(dir.to_string()))
            .or_else(download_dir)
            .unwrap_or_else(temp_dir)
    }

    /// Effective `comfy_table` preset string.
    pub fn table_preset(&self) -> &str {
        self.table_preset
            .as_deref()
            .unwrap_or(presets::UTF8_FULL_CONDENSED)
    }

    /// Effective `comfy_table` content arrangement.
    pub fn table_arrangement(&self) -> ContentArrangement {
        self.table_arrangement
            .clone()
            .unwrap_or(TableArrangementConfig::Dynamic)
            .into()
    }

    /// Effective default page size for `events list`.
    pub fn events_list_page_size(&self) -> u32 {
        self.events_list_page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE)
    }

    /// Effective default page size for `items list`.
    pub fn items_list_page_size(&self) -> u32 {
        self.items_list_page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE)
    }

    /// Resolves the calendar id an `event` or `item` command operates
    /// on: the `-k/--calendar` flag wins; otherwise the
    /// `calendar.default` config is used; otherwise the command bails.
    pub fn calendar_id(&self, flag: Option<String>) -> Result<String> {
        if let Some(id) = flag.or_else(|| self.calendar_default.clone()) {
            return Ok(id);
        }

        bail!("Missing calendar id; pass -k/--calendar or set calendar.default")
    }

    // calendars list column colors

    pub fn calendars_list_table_id_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.id_color, Color::Red)
    }
    pub fn calendars_list_table_name_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.name_color, Color::Green)
    }
    pub fn calendars_list_table_description_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.description_color, Color::Reset)
    }
    pub fn calendars_list_table_color_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.color_color, Color::Reset)
    }

    // events list column colors

    pub fn events_list_table_id_color(&self) -> TableColor {
        map_color_or(self.events_list_table.id_color, Color::Red)
    }
    pub fn events_list_table_summary_color(&self) -> TableColor {
        map_color_or(self.events_list_table.summary_color, Color::Green)
    }
    pub fn events_list_table_start_color(&self) -> TableColor {
        map_color_or(self.events_list_table.start_color, Color::DarkYellow)
    }
    pub fn events_list_table_end_color(&self) -> TableColor {
        map_color_or(self.events_list_table.end_color, Color::DarkYellow)
    }

    // items list column colors

    pub fn items_list_table_id_color(&self) -> TableColor {
        map_color_or(self.items_list_table.id_color, Color::Red)
    }
    pub fn items_list_table_etag_color(&self) -> TableColor {
        map_color_or(self.items_list_table.etag_color, Color::Reset)
    }
    pub fn items_list_table_size_color(&self) -> TableColor {
        map_color_or(self.items_list_table.size_color, Color::Reset)
    }
}

/// Maps a [`crossterm::style::Color`] (deserialized from TOML) into a
/// [`comfy_table::Color`], substituting `fallback` when unset.
pub(crate) fn map_color_or(color: Option<Color>, fallback: Color) -> TableColor {
    match color.unwrap_or(fallback) {
        Color::Reset => TableColor::Reset,
        Color::Black => TableColor::Black,
        Color::DarkGrey => TableColor::DarkGrey,
        Color::Red => TableColor::Red,
        Color::DarkRed => TableColor::DarkRed,
        Color::Green => TableColor::Green,
        Color::DarkGreen => TableColor::DarkGreen,
        Color::Yellow => TableColor::Yellow,
        Color::DarkYellow => TableColor::DarkYellow,
        Color::Blue => TableColor::Blue,
        Color::DarkBlue => TableColor::DarkBlue,
        Color::Magenta => TableColor::Magenta,
        Color::DarkMagenta => TableColor::DarkMagenta,
        Color::Cyan => TableColor::Cyan,
        Color::DarkCyan => TableColor::DarkCyan,
        Color::White => TableColor::White,
        Color::Grey => TableColor::Grey,
        Color::Rgb { r, g, b } => TableColor::Rgb { r, g, b },
        Color::AnsiValue(n) => TableColor::AnsiValue(n),
    }
}

fn merge_calendar_table(
    base: CalendarListTableConfig,
    over: CalendarListTableConfig,
) -> CalendarListTableConfig {
    CalendarListTableConfig {
        id_color: over.id_color.or(base.id_color),
        name_color: over.name_color.or(base.name_color),
        description_color: over.description_color.or(base.description_color),
        color_color: over.color_color.or(base.color_color),
    }
}

fn merge_event_table(
    base: EventListTableConfig,
    over: EventListTableConfig,
) -> EventListTableConfig {
    EventListTableConfig {
        id_color: over.id_color.or(base.id_color),
        summary_color: over.summary_color.or(base.summary_color),
        start_color: over.start_color.or(base.start_color),
        end_color: over.end_color.or(base.end_color),
    }
}

fn merge_item_table(base: ItemListTableConfig, over: ItemListTableConfig) -> ItemListTableConfig {
    ItemListTableConfig {
        id_color: over.id_color.or(base.id_color),
        etag_color: over.etag_color.or(base.etag_color),
        size_color: over.size_color.or(base.size_color),
    }
}

impl From<Config> for Account {
    fn from(config: Config) -> Self {
        Self {
            downloads_dir: config.downloads_dir,
            table_preset: config.table.preset,
            table_arrangement: config.table.arrangement,
            events_list_page_size: config.event.list.page_size,
            items_list_page_size: config.item.list.page_size,
            calendar_default: config.calendar.default,
            calendars_list_table: config.calendar.list.table,
            events_list_table: config.event.list.table,
            items_list_table: config.item.list.table,
        }
    }
}

impl From<AccountConfig> for Account {
    fn from(config: AccountConfig) -> Self {
        Self {
            downloads_dir: config.downloads_dir,
            table_preset: config.table.preset,
            table_arrangement: config.table.arrangement,
            events_list_page_size: config.event.list.page_size,
            items_list_page_size: config.item.list.page_size,
            calendar_default: config.calendar.default,
            calendars_list_table: config.calendar.list.table,
            events_list_table: config.event.list.table,
            items_list_table: config.item.list.table,
        }
    }
}
