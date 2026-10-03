//! # Calendar client
//!
//! The cross-protocol dispatcher the shared commands run on.
//!
//! One variant per compiled-in backend, a value always holding exactly one,
//! picked from the account configuration by the `--backend` flag. Each method
//! dispatches to the active backend, whose glue lives in that protocol
//! module's backend submodule.
//!
//! The surface is a strict least-common-denominator: an operation only
//! appears here when every backend can serve it, and anything narrower
//! belongs to a protocol-specific subcommand.

use std::path::PathBuf;

#[cfg(not(any(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
)))]
use std::convert::Infallible;

use anyhow::{Result, bail};
use log::debug;
use pimalaya_cli::printer::Printer;
#[cfg(any(feature = "gcal", feature = "msgraph"))]
use pimalaya_config::secret::SecretResolver;

use crate::{
    account::context::Account,
    backend::Backend,
    cli::resolve_account,
    config::{AccountConfig, Config},
    shared::{
        calendar::{Calendar, CalendarDiff},
        item::{CalendarItem, CalendarItemQuery},
    },
};

/// The active backend bundled with the merged runtime [`Account`].
///
/// The connection is opened on the first call that needs it and can be
/// dropped again with [`disconnect`](Self::disconnect). A command running
/// a composer holds none open while the editor is up: a server closes an
/// idle connection, and the write landing after a long edit would read
/// the end of a socket nobody is on the other end of any more.
pub struct CalendarClient {
    /// What it takes to open the backend, kept so it can be reopened.
    config: BackendConfig,
    /// The open backend, `None` until something needs it.
    inner: Option<BackendClient>,
    /// The account the command runs against, config already merged.
    pub account: Account,
}

/// The configuration of the backend a [`CalendarClient`] speaks to.
enum BackendConfig {
    #[cfg(feature = "vdir")]
    Vdir(crate::config::VdirConfig),
    #[cfg(feature = "caldav")]
    Caldav(Box<crate::config::CaldavConfig>),
    #[cfg(feature = "pimdir")]
    Pimdir(crate::config::PimdirConfig),
    #[cfg(feature = "gcal")]
    Gcal(Box<crate::config::GcalConfig>),
    #[cfg(feature = "msgraph")]
    Msgraph(Box<crate::config::MsgraphConfig>),
    /// Stands in when no backend is compiled in: never built, so every
    /// match over the backends stays exhaustive.
    #[cfg(not(any(
        feature = "vdir",
        feature = "caldav",
        feature = "pimdir",
        feature = "gcal",
        feature = "msgraph"
    )))]
    Unused(Infallible),
}

/// Exactly one of the compiled-in per-backend glue clients.
enum BackendClient {
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::backend::VdirBackend),
    #[cfg(feature = "caldav")]
    Caldav(Box<crate::caldav::backend::CaldavBackend>),
    #[cfg(feature = "pimdir")]
    Pimdir(Box<crate::pimdir::backend::PimdirBackend>),
    #[cfg(feature = "gcal")]
    Gcal(Box<crate::gcal::backend::GcalBackend>),
    #[cfg(feature = "msgraph")]
    Msgraph(Box<crate::msgraph::backend::MsgraphBackend>),
    /// Stands in when no backend is compiled in: never built, so every
    /// match over the backends stays exhaustive.
    #[cfg(not(any(
        feature = "vdir",
        feature = "caldav",
        feature = "pimdir",
        feature = "gcal",
        feature = "msgraph"
    )))]
    Unused(Infallible),
}

impl CalendarClient {
    /// Selects the backend from the account configuration.
    ///
    /// The first configured backend `backend` allows wins, in calendula's
    /// priority order: vdir, pimdir, CalDAV, gcal. That order prefers a
    /// local store to a network round-trip, and a protocol-standard server
    /// to a vendor API. Nothing is connected here: the first call that
    /// needs the network opens it.
    pub fn new(
        config: Config,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        backend: Backend,
    ) -> Result<Self> {
        #[allow(unused_mut)]
        let mut selected: Option<BackendConfig> = None;

        #[cfg(feature = "vdir")]
        if selected.is_none()
            && backend.allows_vdir()
            && let Some(vdir_config) = account_config.vdir.take()
        {
            selected = Some(BackendConfig::Vdir(vdir_config));
        }

        #[cfg(feature = "pimdir")]
        if selected.is_none()
            && backend.allows_pimdir()
            && let Some(pimdir_config) = account_config.pimdir.take()
        {
            selected = Some(BackendConfig::Pimdir(pimdir_config));
        }

        #[cfg(feature = "caldav")]
        if selected.is_none()
            && backend.allows_caldav()
            && let Some(caldav_config) = account_config.caldav.take()
        {
            selected = Some(BackendConfig::Caldav(Box::new(caldav_config)));
        }

        #[cfg(feature = "gcal")]
        if selected.is_none()
            && backend.allows_gcal()
            && let Some(gcal_config) = account_config.gcal.take()
        {
            selected = Some(BackendConfig::Gcal(Box::new(gcal_config)));
        }

        #[cfg(feature = "msgraph")]
        if selected.is_none()
            && backend.allows_msgraph()
            && let Some(msgraph_config) = account_config.msgraph.take()
        {
            selected = Some(BackendConfig::Msgraph(Box::new(msgraph_config)));
        }

        let Some(config_) = selected else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self {
            config: config_,
            inner: None,
            account,
        })
    }

    /// Resolves the account a shared command runs against, and selects
    /// the backend serving it.
    ///
    /// The four component families call this per subcommand rather than
    /// once for the whole family, so a `build` reaches neither.
    pub fn resolve(
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<Self> {
        let (config, account_config) = resolve_account(printer, config_paths, account_name)?;

        Self::new(config, account_config, backend)
    }

    /// Drops the connection, so the next call opens a fresh one.
    ///
    /// Called before a composer runs: an editor session lasts minutes,
    /// and a server that closed the idle connection meanwhile would fail
    /// the write that comes after it.
    pub fn disconnect(&mut self) {
        if self.inner.take().is_some() {
            debug!("closing the backend connection");
        }
    }

    /// The open backend, opening it when it is not.
    fn open(&mut self) -> Result<&mut BackendClient> {
        if self.inner.is_none() {
            debug!("opening the backend connection");
            self.inner = Some(self.config.open()?);
        }

        Ok(self.inner.as_mut().expect("just opened"))
    }

    /// Takes the notes the writes so far came back with: a capability their
    /// source supports in part, on pimdir alone.
    pub fn take_notes(&mut self) -> Vec<String> {
        match &mut self.inner {
            #[cfg(feature = "pimdir")]
            Some(BackendClient::Pimdir(client)) => client.take_notes(),
            _ => Vec::new(),
        }
    }

    /// Lists every calendar available to the active account.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_calendars(),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.list_calendars(),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_calendars(),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.list_calendars(),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.list_calendars(),
        }
    }

    /// Creates a calendar under `id`, with a name and optional decorations.
    ///
    /// Returns the identifier the backend actually assigned: `id` wherever
    /// it lets the caller name a collection, a server-minted one elsewhere.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_calendar(id, name, description, color),
        }
    }

    /// Applies a partial update to the calendar `id`, preserving the fields
    /// left as `None` in `patch`.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.update_calendar(id, patch),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.update_calendar(id, patch),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.update_calendar(id, patch),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.update_calendar(id, patch),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.update_calendar(id, patch),
        }
    }

    /// Deletes the calendar `id` and every item it contains.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_calendar(id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.delete_calendar(id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_calendar(id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.delete_calendar(id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_calendar(id),
        }
    }

    /// Lists the items of `calendar_id`, narrowed by `query`.
    ///
    /// A backend SHALL narrow by kind and by range before it paginates, so
    /// a page of a component family holds that family. A server-backed
    /// backend pushes both down where its protocol defines such a filter;
    /// the others parse and filter after the fact.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<Vec<CalendarItem>> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_items(calendar_id, query),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.list_items(calendar_id, query),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_items(calendar_id, query),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.list_items(calendar_id, query),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.list_items(calendar_id, query),
        }
    }

    /// Fetches the item `item_id` from `calendar_id`.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.get_item(calendar_id, item_id),
        }
    }

    /// Stores raw iCalendar bytes as a new item, returning the id assigned.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_item(calendar_id, contents),
        }
    }

    /// Replaces the contents of `item_id` inside `calendar_id`.
    ///
    /// `if_match` is the entity tag to gate the write on, `None` overwriting
    /// unconditionally. A backend with no guard concept ignores it.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => {
                client.update_item(calendar_id, item_id, contents, if_match)
            }
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => {
                client.update_item(calendar_id, item_id, contents, if_match)
            }
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => {
                client.update_item(calendar_id, item_id, contents, if_match)
            }
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => {
                client.update_item(calendar_id, item_id, contents, if_match)
            }
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => {
                client.update_item(calendar_id, item_id, contents, if_match)
            }
        }
    }

    /// Deletes `item_id` from `calendar_id`.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_item(calendar_id, item_id),
        }
    }
}

impl BackendConfig {
    /// Opens the backend this configuration describes.
    fn open(&self) -> Result<BackendClient> {
        match self {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            Self::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            Self::Vdir(config) => {
                use crate::vdir::backend::VdirBackend;
                Ok(BackendClient::Vdir(VdirBackend::new(config.clone())))
            }
            #[cfg(feature = "pimdir")]
            Self::Pimdir(config) => {
                use crate::pimdir::backend::PimdirBackend;
                let client = PimdirBackend::new(config.clone())?;
                Ok(BackendClient::Pimdir(Box::new(client)))
            }
            #[cfg(feature = "caldav")]
            Self::Caldav(config) => {
                use crate::caldav::backend::CaldavBackend;
                let client = CaldavBackend::new(config.as_ref().clone())?;
                Ok(BackendClient::Caldav(Box::new(client)))
            }
            #[cfg(feature = "gcal")]
            Self::Gcal(config) => {
                use crate::gcal::backend::GcalBackend;
                let client = GcalBackend::new(config.as_ref().clone(), &mut SecretResolver::new())?;
                Ok(BackendClient::Gcal(Box::new(client)))
            }
            #[cfg(feature = "msgraph")]
            Self::Msgraph(config) => {
                use crate::msgraph::backend::MsgraphBackend;
                let client =
                    MsgraphBackend::new(config.as_ref().clone(), &mut SecretResolver::new())?;
                Ok(BackendClient::Msgraph(Box::new(client)))
            }
        }
    }
}

/// 1-indexed pagination over an in-memory list.
///
/// `page_size = None` returns the whole slice, while a size of zero or a page
/// past the end returns nothing.
pub fn paginate<T>(items: Vec<T>, page: Option<u32>, page_size: Option<u32>) -> Vec<T> {
    let Some(size) = page_size else {
        return items;
    };

    if size == 0 {
        return Vec::new();
    }

    let page = page.unwrap_or(1).max(1);
    let skip = ((page - 1) as usize).saturating_mul(size as usize);

    if skip >= items.len() {
        return Vec::new();
    }

    items.into_iter().skip(skip).take(size as usize).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_windows_the_list_and_clamps_past_the_end() {
        let items = || vec![1, 2, 3, 4, 5];

        assert_eq!(paginate(items(), None, None), vec![1, 2, 3, 4, 5]);
        assert_eq!(paginate(items(), Some(1), Some(2)), vec![1, 2]);
        assert_eq!(paginate(items(), Some(3), Some(2)), vec![5]);
        assert!(paginate(items(), Some(9), Some(2)).is_empty());
        assert!(paginate(items(), None, Some(0)).is_empty());

        // NOTE: a page of zero is clamped to the first page rather than
        // underflowing the skip.
        assert_eq!(paginate(items(), Some(0), Some(2)), vec![1, 2]);
    }
}
