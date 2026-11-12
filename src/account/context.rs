//! # Account context
//!
//! The merged runtime account every command consumes: the global
//! [`Config`] with the selected `[accounts.<name>]` block folded on top.
//!
//! Defaults belong to the accessor methods rather than to the merge, so
//! an unset field stays distinguishable from a set one and the global
//! block keeps overriding nothing it did not name.

use anyhow::{Result, bail};
use pimalaya_config::command::CommandConfig;

use crate::config::{AccountConfig, Config};

/// Page size every `list` command falls back to.
const DEFAULT_LIST_PAGE_SIZE: u32 = 25;

/// A configuration resolved for one run: every option a command reads,
/// each still unset until an accessor supplies its default.
#[derive(Debug, Default)]
pub struct Account {
    /// Page size `event list` falls back to.
    pub events_list_page_size: Option<u32>,
    /// Page size `item list` falls back to.
    pub items_list_page_size: Option<u32>,
    /// Fallback calendar id for `event` and `item` commands when their
    /// `-k/--calendar` flag is omitted.
    pub calendar_default: Option<String>,
    /// Command an item is edited through, spawned on the path of a
    /// temporary iCalendar file.
    pub item_composer: Option<CommandConfig>,
}

impl Account {
    /// Folds `other`'s set fields on top of `self`.
    pub fn merge(self, other: Self) -> Self {
        Self {
            events_list_page_size: other.events_list_page_size.or(self.events_list_page_size),
            items_list_page_size: other.items_list_page_size.or(self.items_list_page_size),

            calendar_default: other.calendar_default.or(self.calendar_default),
            item_composer: other.item_composer.or(self.item_composer),
        }
    }

    /// Page size `event list` uses when `-s/--page-size` is omitted.
    pub fn events_list_page_size(&self) -> u32 {
        self.events_list_page_size.unwrap_or(DEFAULT_LIST_PAGE_SIZE)
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
}

impl From<Config> for Account {
    fn from(config: Config) -> Self {
        Self {
            events_list_page_size: config.event.list.page_size,
            items_list_page_size: config.item.list.page_size,
            calendar_default: config.calendar.default,
            item_composer: config.item.composer,
        }
    }
}

impl From<AccountConfig> for Account {
    fn from(config: AccountConfig) -> Self {
        Self {
            events_list_page_size: config.event.list.page_size,
            items_list_page_size: config.item.list.page_size,
            calendar_default: config.calendar.default,
            item_composer: config.item.composer,
        }
    }
}
