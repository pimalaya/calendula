//! vdir adapter for the shared cross-protocol client.
//!
//! Projects [`io_vdir`]'s collections and items onto calendula's own
//! shared types. A collection directory is a calendar, its metadata
//! marker files carry the display name, description and color, and each
//! `.ics` file inside it is an item.
//!
//! vdir has no server, so it has no entity tags and no server-side
//! query: `if_match` is ignored, and a [`CalendarTimeRange`] is applied
//! after parsing each item rather than pushed down.

use anyhow::{Context, Result, anyhow};
use io_vdir::{
    client::VdirClient,
    collection::VdirCollection,
    item::{VdirItem, VdirItemKind},
    path::VdirPath,
};

use crate::{
    config::VdirConfig,
    shared::{
        calendars::{Calendar, CalendarDiff},
        client::paginate,
        events::Event,
        items::{CalendarItem, CalendarTimeRange},
    },
};

/// The shared-API glue over a vdir home directory.
pub struct VdirBackend {
    client: VdirClient,
    root: VdirPath,
}

impl VdirBackend {
    /// Opens the backend on the configured home directory, expanding
    /// `~` and environment variables first. No filesystem check runs
    /// here; a missing home surfaces on the first operation.
    pub fn new(config: VdirConfig) -> Self {
        let root = shellexpand::full(&config.home_dir.to_string_lossy())
            .map(|home| VdirPath::new(home.into_owned()))
            .unwrap_or_else(|_| VdirPath::new(config.home_dir.to_string_lossy().into_owned()));

        Self {
            client: VdirClient::new(root.clone()),
            root,
        }
    }

    /// Lists every collection directly under the home directory.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        let collections = self.client.list_collections()?;
        Ok(collections.into_iter().map(calendar_from).collect())
    }

    /// Creates the collection directory and writes the metadata
    /// markers it carries.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<()> {
        let collection = VdirCollection {
            path: self.path(id),
            display_name: Some(name.to_owned()),
            description: description.map(str::to_owned),
            color: color.map(str::to_owned),
        };

        self.client.create_collection(collection)?;
        Ok(())
    }

    /// Rewrites the metadata markers of a collection, reading the
    /// current ones first so a field the patch leaves untouched
    /// survives.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        let path = self.path(id);
        let mut collection = self
            .client
            .list_collections()?
            .into_iter()
            .find(|collection| collection.path == path)
            .ok_or_else(|| anyhow!("Calendar `{id}` not found"))?;

        if let Some(name) = patch.name {
            collection.display_name = Some(name);
        }

        if let Some(description) = patch.description {
            collection.description = description;
        }

        if let Some(color) = patch.color {
            collection.color = color;
        }

        self.client.update_collection(collection)?;
        Ok(())
    }

    /// Recursively removes the collection directory.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        self.client.delete_collection(self.path(id))?;
        Ok(())
    }

    /// Lists the iCalendar items of a collection, dropping the vCards a
    /// mixed directory may also hold.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        range: Option<&CalendarTimeRange>,
    ) -> Result<Vec<CalendarItem>> {
        let items = self.client.list_items(self.path(calendar_id))?;

        let items: Vec<CalendarItem> = items
            .into_iter()
            .filter(|item| item.kind == VdirItemKind::Ical)
            .map(|item| item_from(calendar_id, item))
            .filter(|item| in_range(item, range))
            .collect();

        Ok(paginate(items, page, page_size))
    }

    /// Reads one item's bytes off disk.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let item = self
            .client
            .get_item(self.path(calendar_id), item_id)
            .with_context(|| format!("Read item `{item_id}` from calendar `{calendar_id}`"))?;

        Ok(item_from(calendar_id, item))
    }

    /// Writes new bytes under a freshly minted id.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let (id, _) =
            self.client
                .store_item(self.path(calendar_id), None, VdirItemKind::Ical, contents)?;
        Ok(id)
    }

    /// Overwrites an item's bytes. vdir carries no entity tag, so
    /// `if_match` cannot be honoured and is ignored.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<()> {
        self.client.store_item(
            self.path(calendar_id),
            Some(item_id.to_owned()),
            VdirItemKind::Ical,
            contents,
        )?;
        Ok(())
    }

    /// Removes an item file.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        self.client.delete_item(self.path(calendar_id), item_id)?;
        Ok(())
    }

    /// The on-disk path of the collection named `id`.
    fn path(&self, id: &str) -> VdirPath {
        self.root.join(id)
    }
}

/// Projects a vdir collection onto the shared [`Calendar`], falling
/// back to the directory name when no display name marker is set.
fn calendar_from(collection: VdirCollection) -> Calendar {
    let id = collection.id().to_owned();

    Calendar {
        name: collection
            .display_name
            .clone()
            .unwrap_or_else(|| id.clone()),
        id,
        description: collection.description,
        color: collection.color,
    }
}

/// Projects a vdir item onto the shared [`CalendarItem`].
fn item_from(calendar_id: &str, item: VdirItem) -> CalendarItem {
    CalendarItem {
        id: item.id().unwrap_or_default().to_owned(),
        calendar_id: calendar_id.to_owned(),
        etag: None,
        contents: item.contents,
    }
}

/// Whether an item has at least one VEVENT starting inside `range`.
/// An item carrying no event at all is kept only when no range was
/// asked for, so a filtered listing never shows an undated resource.
fn in_range(item: &CalendarItem, range: Option<&CalendarTimeRange>) -> bool {
    let Some(range) = range else {
        return true;
    };

    Event::project(item)
        .iter()
        .any(|event| range.contains(&event.start))
}
