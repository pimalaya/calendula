//! # Account context
//!
//! The merged runtime account every command consumes: the global
//! [`Config`] with the selected `[accounts.<name>]` block folded on top.
//!
//! Defaults belong to the accessor methods rather than to the merge, so
//! an unset field stays distinguishable from a set one and the global
//! block keeps overriding nothing it did not name.

use anyhow::{Result, bail};
use crossterm::style::Color;
use pimalaya_cli::table::{Color as TableColor, ContentArrangement, TableStyle};
use pimalaya_config::command::CommandConfig;

use crate::{
    config::{
        AccountConfig, CalendarListTableConfig, Config, EventListTableConfig, ItemListTableConfig,
        JournalListTableConfig, TableArrangementConfig, TodoListTableConfig,
    },
    shared::table::{DEFAULT_PRESET, style_from_preset},
};

/// Page size every `list` command falls back to.
const DEFAULT_LIST_PAGE_SIZE: u32 = 25;

/// A configuration resolved for one run: every option a command reads,
/// each still unset until an accessor supplies its default.
#[derive(Debug, Default)]
pub struct Account {
    /// `comfy_table` preset name every table renders with.
    pub table_preset: Option<String>,
    /// Column-arrangement strategy every table renders with.
    pub table_arrangement: Option<TableArrangementConfig>,
    /// Page size `event list` falls back to.
    pub events_list_page_size: Option<u32>,
    /// Page size `todo list` falls back to.
    pub todos_list_page_size: Option<u32>,
    /// Page size `journal list` falls back to.
    pub journals_list_page_size: Option<u32>,
    /// Page size `item list` falls back to.
    pub items_list_page_size: Option<u32>,
    /// Fallback calendar id for `event` and `item` commands when their
    /// `-k/--calendar` flag is omitted.
    pub calendar_default: Option<String>,
    /// Command an item is edited through, spawned on the path of a
    /// temporary iCalendar file.
    pub item_composer: Option<CommandConfig>,
    /// Per-column colors of the `calendar list` table.
    pub calendars_list_table: CalendarListTableConfig,
    /// Per-column colors of the `event list` table.
    pub events_list_table: EventListTableConfig,
    /// Per-column colors of the `todo list` table.
    pub todos_list_table: TodoListTableConfig,
    /// Per-column colors of the `journal list` table.
    pub journals_list_table: JournalListTableConfig,
    /// Per-column colors of the `item list` table.
    pub items_list_table: ItemListTableConfig,
}

impl Account {
    /// Folds `other`'s set fields on top of `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            table_preset: other.table_preset.or(self.table_preset),
            table_arrangement: other.table_arrangement.or(self.table_arrangement),

            events_list_page_size: other.events_list_page_size.or(self.events_list_page_size),
            todos_list_page_size: other.todos_list_page_size.or(self.todos_list_page_size),
            journals_list_page_size: other
                .journals_list_page_size
                .or(self.journals_list_page_size),
            items_list_page_size: other.items_list_page_size.or(self.items_list_page_size),

            calendar_default: other.calendar_default.or(self.calendar_default),
            item_composer: other.item_composer.or(self.item_composer),

            calendars_list_table: merge_calendar_table(
                self.calendars_list_table,
                other.calendars_list_table,
            ),
            events_list_table: merge_event_table(self.events_list_table, other.events_list_table),
            todos_list_table: merge_todo_table(self.todos_list_table, other.todos_list_table),
            journals_list_table: merge_journal_table(
                self.journals_list_table,
                other.journals_list_table,
            ),
            items_list_table: merge_item_table(self.items_list_table, other.items_list_table),
        }
    }

    /// The table style the configured preset name maps to.
    pub fn table_style(&self) -> TableStyle {
        style_from_preset(self.table_preset.as_deref().unwrap_or(DEFAULT_PRESET))
    }

    /// The configured column arrangement, dynamic by default.
    pub fn table_arrangement(&self) -> ContentArrangement {
        self.table_arrangement
            .clone()
            .unwrap_or(TableArrangementConfig::Dynamic)
            .into()
    }

    /// Page size `event list` uses when `-s/--page-size` is omitted.
    pub fn events_list_page_size(&self) -> u32 {
        self.events_list_page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE)
    }

    /// Page size `todo list` uses when `-s/--page-size` is omitted.
    pub fn todos_list_page_size(&self) -> u32 {
        self.todos_list_page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE)
    }

    /// Page size `journal list` uses when `-s/--page-size` is omitted.
    pub fn journals_list_page_size(&self) -> u32 {
        self.journals_list_page_size
            .unwrap_or(DEFAULT_LIST_PAGE_SIZE)
    }

    /// Page size `item list` uses when `-s/--page-size` is omitted.
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

    /// Resolves the composer an item is edited through.
    ///
    /// The flag wins, taken as a shell line, then the `item.composer`
    /// config, otherwise this bails naming both ways of setting one.
    pub fn item_composer(&self, flag: Option<String>) -> Result<CommandConfig> {
        if let Some(line) = flag {
            return Ok(CommandConfig::Shell(line));
        }

        let Some(composer) = self.item_composer.clone() else {
            bail!(
                "No composer configured; set item.composer or pass --composer <COMMAND>, \
                 which is spawned on the path of the iCalendar to edit"
            )
        };

        Ok(composer)
    }

    /// Color of the ID column of `calendar list`, red by default.
    pub fn calendars_list_table_id_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.id_color, Color::Red)
    }
    /// Color of the NAME column of `calendar list`, green by default.
    pub fn calendars_list_table_name_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.name_color, Color::Green)
    }
    /// Color of the DESCRIPTION column of `calendar list`, unset by
    /// default.
    pub fn calendars_list_table_description_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.description_color, Color::Reset)
    }
    /// Color of the COLOR column of `calendar list`, unset by default.
    pub fn calendars_list_table_color_color(&self) -> TableColor {
        map_color_or(self.calendars_list_table.color_color, Color::Reset)
    }

    /// Color of the ID column of `event list`, red by default.
    pub fn events_list_table_id_color(&self) -> TableColor {
        map_color_or(self.events_list_table.id_color, Color::Red)
    }
    /// Color of the SUMMARY column of `event list`, green by default.
    pub fn events_list_table_summary_color(&self) -> TableColor {
        map_color_or(self.events_list_table.summary_color, Color::Green)
    }
    /// Color of the START column of `event list`, yellow by default.
    pub fn events_list_table_start_color(&self) -> TableColor {
        map_color_or(self.events_list_table.start_color, Color::DarkYellow)
    }
    /// Color of the END column of `event list`, yellow by default.
    pub fn events_list_table_end_color(&self) -> TableColor {
        map_color_or(self.events_list_table.end_color, Color::DarkYellow)
    }

    /// Color of the ID column of `todo list`, red by default.
    pub fn todos_list_table_id_color(&self) -> TableColor {
        map_color_or(self.todos_list_table.id_color, Color::Red)
    }
    /// Color of the SUMMARY column of `todo list`, green by default.
    pub fn todos_list_table_summary_color(&self) -> TableColor {
        map_color_or(self.todos_list_table.summary_color, Color::Green)
    }
    /// Color of the DUE column of `todo list`, yellow by default.
    pub fn todos_list_table_due_color(&self) -> TableColor {
        map_color_or(self.todos_list_table.due_color, Color::DarkYellow)
    }
    /// Color of the STATUS column of `todo list`, unset by default.
    pub fn todos_list_table_status_color(&self) -> TableColor {
        map_color_or(self.todos_list_table.status_color, Color::Reset)
    }

    /// Color of the ID column of `journal list`, red by default.
    pub fn journals_list_table_id_color(&self) -> TableColor {
        map_color_or(self.journals_list_table.id_color, Color::Red)
    }
    /// Color of the SUMMARY column of `journal list`, green by default.
    pub fn journals_list_table_summary_color(&self) -> TableColor {
        map_color_or(self.journals_list_table.summary_color, Color::Green)
    }
    /// Color of the START column of `journal list`, yellow by default.
    pub fn journals_list_table_start_color(&self) -> TableColor {
        map_color_or(self.journals_list_table.start_color, Color::DarkYellow)
    }

    /// Color of the ID column of `item list`, red by default.
    pub fn items_list_table_id_color(&self) -> TableColor {
        map_color_or(self.items_list_table.id_color, Color::Red)
    }
    /// Color of the ETAG column of `item list`, unset by default.
    pub fn items_list_table_etag_color(&self) -> TableColor {
        map_color_or(self.items_list_table.etag_color, Color::Reset)
    }
    /// Color of the SIZE column of `item list`, unset by default.
    pub fn items_list_table_size_color(&self) -> TableColor {
        map_color_or(self.items_list_table.size_color, Color::Reset)
    }
}

/// Maps a [`crossterm::style::Color`] (deserialized from TOML) onto a
/// [`TableColor`], substituting `fallback` when unset.
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

fn merge_todo_table(base: TodoListTableConfig, over: TodoListTableConfig) -> TodoListTableConfig {
    TodoListTableConfig {
        id_color: over.id_color.or(base.id_color),
        summary_color: over.summary_color.or(base.summary_color),
        due_color: over.due_color.or(base.due_color),
        status_color: over.status_color.or(base.status_color),
    }
}

fn merge_journal_table(
    base: JournalListTableConfig,
    over: JournalListTableConfig,
) -> JournalListTableConfig {
    JournalListTableConfig {
        id_color: over.id_color.or(base.id_color),
        summary_color: over.summary_color.or(base.summary_color),
        start_color: over.start_color.or(base.start_color),
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
            table_preset: config.table.preset,
            table_arrangement: config.table.arrangement,
            events_list_page_size: config.event.list.page_size,
            todos_list_page_size: config.todo.list.page_size,
            journals_list_page_size: config.journal.list.page_size,
            items_list_page_size: config.item.list.page_size,
            calendar_default: config.calendar.default,
            item_composer: config.item.composer,
            calendars_list_table: config.calendar.list.table,
            events_list_table: config.event.list.table,
            todos_list_table: config.todo.list.table,
            journals_list_table: config.journal.list.table,
            items_list_table: config.item.list.table,
        }
    }
}

impl From<AccountConfig> for Account {
    fn from(config: AccountConfig) -> Self {
        Self {
            table_preset: config.table.preset,
            table_arrangement: config.table.arrangement,
            events_list_page_size: config.event.list.page_size,
            todos_list_page_size: config.todo.list.page_size,
            journals_list_page_size: config.journal.list.page_size,
            items_list_page_size: config.item.list.page_size,
            calendar_default: config.calendar.default,
            item_composer: config.item.composer,
            calendars_list_table: config.calendar.list.table,
            events_list_table: config.event.list.table,
            todos_list_table: config.todo.list.table,
            journals_list_table: config.journal.list.table,
            items_list_table: config.item.list.table,
        }
    }
}
