//! Cross-protocol [`CalendarClient`] for the shared subcommands
//! (`calendar`, `event`, `item`).
//!
//! One variant per compiled-in backend (vdir, pimdir, CalDAV, gcal); a
//! value always holds exactly one, picked from the account
//! configuration by the `--backend` flag. Each shared-API method
//! dispatches to the active backend's matching method, and the
//! per-backend glue lives in that protocol module's backend submodule.
//!
//! The shared surface is a strict least-common-denominator: an
//! operation only appears here when every backend can serve it.
//! Anything narrower belongs to a protocol-specific subcommand.

use anyhow::{Result, bail};

use crate::{
    account::context::Account,
    backend::Backend,
    config::{AccountConfig, Config},
    shared::{
        calendars::{Calendar, CalendarDiff},
        items::{CalendarItem, CalendarTimeRange},
    },
};

/// Cross-protocol calendar client bundling the active backend and the
/// merged runtime [`Account`].
pub struct CalendarClient {
    inner: BackendClient,
    pub account: Account,
}

/// The active backend of a [`CalendarClient`]: exactly one of the
/// compiled-in per-backend glue clients.
enum BackendClient {
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::backend::VdirBackend),
    #[cfg(feature = "caldav")]
    Caldav(Box<crate::caldav::backend::CaldavBackend>),
    #[cfg(feature = "pimdir")]
    Pimdir(Box<crate::pimdir::backend::PimdirBackend>),
    #[cfg(feature = "gcal")]
    Gcal(Box<crate::gcal::backend::GcalBackend>),
}

impl CalendarClient {
    /// Builds the client from the account configuration: the first
    /// configured backend allowed by `backend` wins, in calendula's
    /// priority order (vdir, pimdir, CalDAV, gcal), so a local store is
    /// preferred over a network round-trip and a protocol-standard
    /// server over a vendor API.
    pub fn new(
        config: Config,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        backend: Backend,
    ) -> Result<Self> {
        #[allow(unused_mut)]
        let mut inner: Option<BackendClient> = None;

        #[cfg(feature = "vdir")]
        if inner.is_none()
            && backend.allows_vdir()
            && let Some(vdir_config) = account_config.vdir.take()
        {
            use crate::vdir::backend::VdirBackend;
            inner = Some(BackendClient::Vdir(VdirBackend::new(vdir_config)));
        }

        #[cfg(feature = "pimdir")]
        if inner.is_none()
            && backend.allows_pimdir()
            && let Some(pimdir_config) = account_config.pimdir.take()
        {
            use crate::pimdir::backend::PimdirBackend;
            let client = PimdirBackend::new(pimdir_config)?;
            inner = Some(BackendClient::Pimdir(Box::new(client)));
        }

        #[cfg(feature = "caldav")]
        if inner.is_none()
            && backend.allows_caldav()
            && let Some(caldav_config) = account_config.caldav.take()
        {
            use crate::caldav::backend::CaldavBackend;
            let client = CaldavBackend::new(caldav_config)?;
            inner = Some(BackendClient::Caldav(Box::new(client)));
        }

        #[cfg(feature = "gcal")]
        if inner.is_none()
            && backend.allows_gcal()
            && let Some(gcal_config) = account_config.gcal.take()
        {
            use crate::gcal::backend::GcalBackend;
            let client = GcalBackend::new(gcal_config)?;
            inner = Some(BackendClient::Gcal(Box::new(client)));
        }

        let Some(inner) = inner else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self { inner, account })
    }

    /// Lists every calendar available to the active account.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_calendars(),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.list_calendars(),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_calendars(),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.list_calendars(),
        }
    }

    /// Creates a calendar under `id`, carrying `name` and optionally a
    /// description and a color.
    ///
    /// Returns the identifier the backend actually assigned, which is
    /// `id` everywhere the backend lets the caller name a collection,
    /// and a server-minted one where it does not.
    pub fn create_calendar(
        &mut self,
        id: &str,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.create_calendar(id, name, description, color),
        }
    }

    /// Applies a partial update to the calendar `id`. Fields left as
    /// `None` in `patch` are preserved.
    pub fn update_calendar(&mut self, id: &str, patch: CalendarDiff) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.update_calendar(id, patch),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.update_calendar(id, patch),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.update_calendar(id, patch),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.update_calendar(id, patch),
        }
    }

    /// Deletes the calendar `id` and every item it contains.
    pub fn delete_calendar(&mut self, id: &str) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_calendar(id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.delete_calendar(id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_calendar(id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.delete_calendar(id),
        }
    }

    /// Lists the items of `calendar_id`, optionally narrowed to
    /// `range`. `page` is 1-indexed and defaults to the first page;
    /// `page_size = None` returns the whole window.
    ///
    /// The server-backed backends push `range` down (CalDAV as a
    /// `time-range` filter, gcal as `timeMin` / `timeMax`); the local
    /// backends parse each item and filter after the fact.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        range: Option<&CalendarTimeRange>,
    ) -> Result<Vec<CalendarItem>> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_items(calendar_id, page, page_size, range),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.list_items(calendar_id, page, page_size, range),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.list_items(calendar_id, page, page_size, range),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.list_items(calendar_id, page, page_size, range),
        }
    }

    /// Fetches the item `item_id` from `calendar_id`.
    pub fn get_item(&mut self, calendar_id: &str, item_id: &str) -> Result<CalendarItem> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.get_item(calendar_id, item_id),
        }
    }

    /// Stores raw iCalendar bytes as a new item of `calendar_id`.
    /// Returns the identifier the backend assigned.
    pub fn create_item(&mut self, calendar_id: &str, contents: Vec<u8>) -> Result<String> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.create_item(calendar_id, contents),
        }
    }

    /// Replaces the contents of `item_id` inside `calendar_id`.
    ///
    /// `if_match` is the entity tag to gate the write on; pass `None`
    /// to overwrite unconditionally. Backends with no guard concept
    /// ignore it.
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
    ) -> Result<()> {
        match &mut self.inner {
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
        }
    }

    /// Deletes `item_id` from `calendar_id`.
    pub fn delete_item(&mut self, calendar_id: &str, item_id: &str) -> Result<()> {
        match &mut self.inner {
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_item(calendar_id, item_id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.delete_item(calendar_id, item_id),
        }
    }
}

/// 1-indexed pagination over an in-memory list.
///
/// `page_size = None` returns the whole slice; a size of zero, or a
/// page past the end, returns nothing.
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

        // A page of zero is clamped to the first page rather than
        // underflowing the skip.
        assert_eq!(paginate(items(), Some(0), Some(2)), vec![1, 2]);
    }
}
