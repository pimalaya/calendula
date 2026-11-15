//! # Vdir backend
//!
//! The shared-API adapter over a vdir home directory, projecting
//! [`io_vdir`]'s collections and items onto the shared types.
//!
//! A collection directory is a calendar, its marker files carry the
//! display name, description and color, and each `.ics` file is an
//! item. vdir has no server, hence no entity tag: `if_match` is ignored
//! and a [`CalendarTimeRange`] filters after parsing, never pushed down.
//!
//! [`CalendarTimeRange`]: crate::shared::item::CalendarTimeRange

use std::io;

use anyhow::{Result, anyhow, bail};
use ical::component::IcalComponentKind;
use io_vdir::{
    client::{VdirClient, VdirClientError},
    collection::VdirCollection,
    item::{VdirItem, VdirItemKind, get::VdirItemGetError},
    path::VdirPath,
};

use crate::{
    config::VdirConfig,
    error::{self, ErrorCode, calendar_duplicate, calendar_not_found, recoded},
    shared::{
        calendar::{Calendar, CalendarDiff},
        client::window,
        ical::may_hold_kind,
        item::{CalendarItem, CalendarItemList, CalendarItemQuery},
    },
    vdir::client::{collection_dir, is_dir_name},
};

/// The shared-API glue over a vdir home directory.
pub struct VdirBackend {
    client: VdirClient,
    root: VdirPath,
}

impl VdirBackend {
    /// Opens the backend on the configured home directory.
    ///
    /// The path arrives shell-expanded, [`VdirConfig::root`] doing
    /// it at deserialize. Nothing is checked here, so a missing home
    /// surfaces on the first operation.
    pub fn new(config: VdirConfig) -> Self {
        let root = VdirPath::new(config.root.to_string_lossy().into_owned());

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

    /// Creates the collection directory and its metadata markers, the
    /// directory named `id`, else `name`.
    ///
    /// An id that is not one directory name is refused, as it would reach
    /// out of the home or below one of its calendars, and one naming a
    /// calendar of the home `calendar-duplicate`, io-vdir writing its
    /// markers over an existing directory.
    pub fn create_calendar(
        &mut self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        let id = match id {
            Some(id) if !is_dir_name(id) => bail!("Calendar id `{id}` is not one directory name"),
            Some(id) => id,
            None if !is_dir_name(name) => bail!(
                "Calendar name `{name}` is not one directory name: give the calendar an id \
                 with `--id`"
            ),
            None => name,
        };

        if collection_dir(&self.root, id).is_some() {
            return Err(calendar_duplicate(id));
        }

        let collection = VdirCollection {
            path: self.root.join(id),
            display_name: Some(name.to_owned()),
            description: description.map(str::to_owned),
            color: color.map(str::to_owned),
        };

        self.client.create_collection(collection)?;
        Ok(id.to_owned())
    }

    /// Rewrites a collection's metadata markers.
    ///
    /// Reads the current ones first, so a field the patch leaves
    /// untouched survives.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        let path = self.calendar_path(id)?;
        let mut collection = self
            .client
            .list_collections()?
            .into_iter()
            .find(|collection| collection.path == path)
            .ok_or_else(|| calendar_not_found(format!("Calendar `{id}` not found")))?;

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
        self.client.delete_collection(self.calendar_path(id)?)?;
        Ok(())
    }

    /// Lists a collection's iCalendar items, dropping any vCards a
    /// mixed directory holds beside them.
    ///
    /// A file that cannot be read is an `item-unreadable` entry, named on
    /// the first window alone since it holds no place in any; one removed
    /// since the directory was listed is left out.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        let path = self.calendar_path(calendar_id)?;
        let (items, unread) = self.client.list_readable_items(path)?;

        let items: Vec<CalendarItem> = items
            .into_iter()
            .filter(|item| item.kind == VdirItemKind::Ical)
            .map(|item| item_from(calendar_id, item))
            .filter(|item| is_kind(item, query.kind))
            .filter(|item| item.starts_within(query.range))
            .collect();

        let errors = match query.skip {
            0 => unread
                .into_iter()
                .filter(|(entry, err)| {
                    entry.kind == VdirItemKind::Ical && err.kind() != io::ErrorKind::NotFound
                })
                .map(|(entry, err)| {
                    let path = entry.path;
                    let err =
                        anyhow::Error::from(err).context(format!("Cannot read item file {path}"));
                    error::unreadable(Some(&entry.id), err)
                })
                .collect(),
            _ => Vec::new(),
        };

        Ok(CalendarItemList {
            items: window(items, query.skip, query.take),
            errors,
        })
    }

    /// Reads one item's bytes off disk.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        let path = self.calendar_path(calendar_id)?;
        let item = self
            .client
            .get_item(path, item_id)
            .map_err(unreadable)
            .map_err(|err| error::read_failed(err, item_id, calendar_id))?;

        Ok(item_from(calendar_id, item))
    }

    /// Writes new bytes under a freshly minted id.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        let path = self.calendar_path(calendar_id)?;
        let (id, _) = self
            .client
            .store_item(path, None, VdirItemKind::Ical, contents)?;
        Ok(id)
    }

    /// Overwrites the bytes of an item the calendar holds, an id naming
    /// none refused `item-not-found` as on every other backend, never
    /// written.
    ///
    /// vdir carries no entity tag, so `if_match` cannot be honoured.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        _if_match: Option<&str>,
    ) -> Result<()> {
        let path = self.calendar_path(calendar_id)?;
        let (_, kind) = self.client.locate_item(path.clone(), item_id)?;
        self.client
            .store_item(path, Some(item_id.to_owned()), kind, contents)?;
        Ok(())
    }

    /// Removes an item file.
    ///
    /// A vdir item has no entity tag, so an `if_match` is refused.
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
    ) -> Result<()> {
        if if_match.is_some() {
            return Err(anyhow!(
                "The vdir backend has no ETag, so it cannot honour If-Match"
            ));
        }
        let path = self.calendar_path(calendar_id)?;
        self.client.delete_item(path, item_id)?;
        Ok(())
    }

    /// The directory of the calendar `id`, refused `calendar-not-found` when
    /// the home holds none of that name.
    fn calendar_path(&self, id: &str) -> Result<VdirPath> {
        collection_dir(&self.root, id)
            .ok_or_else(|| calendar_not_found(format!("Calendar `{id}` not found")))
    }
}

/// Projects a vdir collection onto the shared [`Calendar`].
///
/// The directory name stands in when no display name marker is set.
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
        default: false,
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
///
/// Whether an item is of `kind`, every item passing when none is named.
fn is_kind(item: &CalendarItem, kind: Option<IcalComponentKind>) -> bool {
    kind.is_none_or(|kind| may_hold_kind(&item.contents, kind))
}

/// The failure of an item read: a file located but not read is
/// `item-unreadable`, its `io::Error` kept in the message.
fn unreadable(err: VdirClientError) -> anyhow::Error {
    match err {
        VdirClientError::FileRead { .. }
        | VdirClientError::VdirItemGet(VdirItemGetError::Unreadable(_)) => {
            recoded(ErrorCode::ItemUnreadable, err)
        }
        err => err.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pimalaya_cli::error::code_of;

    use super::*;
    use crate::error::{ErrorCode, code};

    /// A backend over a home holding the calendar `cal` and its one item
    /// `a`.
    fn backend() -> (tempfile::TempDir, VdirBackend) {
        let home = tempfile::tempdir().unwrap();
        fs::create_dir(home.path().join("cal")).unwrap();
        fs::write(home.path().join("cal").join("a.ics"), "BEGIN:VCALENDAR\r\n").unwrap();

        let config = VdirConfig {
            root: home.path().to_path_buf(),
            home_dir: None,
        };
        (home, VdirBackend::new(config))
    }

    #[test]
    fn a_missing_item_or_calendar_carries_its_code() {
        let (_home, mut backend) = backend();

        let err = backend.get_item("cal", "nope").unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        assert!(!format!("{err:#}").contains("Read item"), "{err:#}");

        let err = backend.delete_item("cal", "nope", None).unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");

        let patch = CalendarDiff {
            name: Some("Renamed".into()),
            ..Default::default()
        };
        let err = backend.update_calendar("nope", patch).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CalendarNotFound), "{err:#}");
    }

    /// An update naming no item of the calendar fails `item-not-found` and
    /// writes nothing, where it created the item; one naming an item
    /// overwrites it.
    #[test]
    fn an_update_of_a_missing_item_creates_nothing() {
        let (home, mut backend) = backend();
        let ical = b"BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n".to_vec();

        let err = backend
            .update_item("cal", "nope", ical.clone(), None)
            .unwrap_err();
        assert_eq!(code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        assert!(!home.path().join("cal").join("nope.ics").exists());

        backend.update_item("cal", "a", ical.clone(), None).unwrap();
        let stored = fs::read(home.path().join("cal").join("a.ics")).unwrap();
        assert_eq!(stored, ical);
    }

    /// An item file located but not read fails its read as unreadable.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_item_file_is_coded_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let (home, mut backend) = backend();
        let path = home.path().join("cal").join("a.ics");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

        // A superuser reads it all the same: nothing to test.
        if fs::read(&path).is_ok() {
            return;
        }

        let err = backend.get_item("cal", "a").unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::ItemUnreadable), "{err:#}");
    }
}
