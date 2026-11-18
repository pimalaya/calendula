//! # Calendar client
//!
//! The cross-protocol dispatcher the unified commands run on.
//!
//! One variant per compiled-in backend, a value always holding exactly one,
//! picked from the account configuration by the `--backend` flag. Each method
//! dispatches to the active backend, whose glue lives in that protocol
//! module's backend submodule.
//!
//! The surface is the union of what the backends serve: a command checks
//! the capabilities it reaches through [`CalendarClient::require`] before
//! anything connects, and an operation a backend serves with another
//! result stays in its protocol-specific subcommand.
//!
//! A build without any backend keeps the client, the unified command tree
//! being the same in every build, with no arm to read its arguments.

#![cfg_attr(
    not(any(
        feature = "vdir",
        feature = "caldav",
        feature = "jmap",
        feature = "pimdir",
        feature = "gcal",
        feature = "msgraph"
    )),
    allow(unused_variables)
)]

use std::path::PathBuf;

#[cfg(not(any(
    feature = "vdir",
    feature = "caldav",
    feature = "jmap",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
)))]
use std::convert::Infallible;

use anyhow::{Error, Result, bail};
use log::debug;
use pimalaya_config::secret::SecretResolver;

use crate::{
    account::context::Account,
    backend::Backend,
    capability::{self, Capability},
    cli::resolve_account,
    config::{AccountConfig, Config},
    error,
    mcp::sending_closed,
    shared::{
        arg::SendUpdates,
        calendar::{Calendar, CalendarDiff},
        ical::invites,
        item::{CalendarItem, CalendarItemList, CalendarItemPage, CalendarItemQuery, ItemOrigin},
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
    /// The account's credentials resolved so far, so a reopened backend
    /// runs no secret command twice.
    resolver: SecretResolver,
    /// The backends the account configures, which a refusal names.
    configured: Vec<&'static str>,
    /// The name of the account the command runs against.
    pub name: String,
    /// The account the command runs against, config already merged.
    pub account: Account,
}

/// The backend the unified commands run against: the first one the account
/// configures that `backend` allows, in priority order (vdir, pimdir,
/// CalDAV, JMAP, gcal, msgraph), `None` when it configures none.
///
/// `auto` stops on the block of a backend this build leaves out, returned
/// as the error, rather than falls through to the next one: a configuration
/// reaches the same backend in every build, or fails saying why.
pub fn resolve_backend(
    account_config: &AccountConfig,
    backend: Backend,
) -> Result<Option<Backend>, &'static str> {
    for (name, built) in account_config.blocks() {
        if !built {
            if backend == Backend::Auto {
                return Err(name);
            }

            continue;
        }

        let Ok(named) = name.parse::<Backend>() else {
            continue;
        };

        if backend == Backend::Auto || backend == named {
            return Ok(Some(named));
        }
    }

    Ok(None)
}

/// The configuration of the backend a [`CalendarClient`] speaks to.
enum BackendConfig {
    #[cfg(feature = "vdir")]
    Vdir(crate::config::VdirConfig),
    #[cfg(feature = "caldav")]
    Caldav(Box<crate::config::CaldavConfig>),
    #[cfg(feature = "jmap")]
    Jmap(Box<crate::config::JmapConfig>),
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
        feature = "jmap",
        feature = "pimdir",
        feature = "gcal",
        feature = "msgraph"
    )))]
    #[allow(dead_code)]
    Unused(Infallible),
}

/// Exactly one of the compiled-in per-backend glue clients.
enum BackendClient {
    #[cfg(feature = "vdir")]
    Vdir(crate::vdir::backend::VdirBackend),
    #[cfg(feature = "caldav")]
    Caldav(Box<crate::caldav::backend::CaldavBackend>),
    #[cfg(feature = "jmap")]
    Jmap(Box<crate::jmap::backend::JmapBackend>),
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
        feature = "jmap",
        feature = "pimdir",
        feature = "gcal",
        feature = "msgraph"
    )))]
    #[allow(dead_code)]
    Unused(Infallible),
}

impl CalendarClient {
    /// Selects the backend from the account configuration.
    ///
    /// The first configured backend `backend` allows wins, in calendula's
    /// priority order: vdir, pimdir, CalDAV, JMAP, gcal, msgraph. That order
    /// prefers a local store to a network round-trip, and a
    /// protocol-standard server to a vendor API. It skips no backend for
    /// a capability it lacks, every unified verb exchanging ids the
    /// others take. Nothing is connected here: the first call that needs
    /// the network opens it.
    pub fn new(
        config: Config,
        name: String,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        backend: Backend,
    ) -> Result<Self> {
        let configured = account_config.backends();

        let resolved = resolve_backend(&account_config, backend)
            .map_err(|name| error::backend_not_built(name, &configured))?;

        let selected = match resolved {
            #[cfg(feature = "vdir")]
            Some(Backend::Vdir) => account_config.vdir.take().map(BackendConfig::Vdir),
            #[cfg(feature = "pimdir")]
            Some(Backend::Pimdir) => account_config.pimdir.take().map(BackendConfig::Pimdir),
            #[cfg(feature = "caldav")]
            Some(Backend::Caldav) => account_config
                .caldav
                .take()
                .map(|config| BackendConfig::Caldav(Box::new(config))),
            #[cfg(feature = "jmap")]
            Some(Backend::Jmap) => account_config
                .jmap
                .take()
                .map(|config| BackendConfig::Jmap(Box::new(config))),
            #[cfg(feature = "gcal")]
            Some(Backend::Gcal) => account_config
                .gcal
                .take()
                .map(|config| BackendConfig::Gcal(Box::new(config))),
            #[cfg(feature = "msgraph")]
            Some(Backend::Msgraph) => account_config
                .msgraph
                .take()
                .map(|config| BackendConfig::Msgraph(Box::new(config))),
            _ => None,
        };

        let Some(config_) = selected else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self {
            config: config_,
            inner: None,
            resolver: SecretResolver::new(),
            configured,
            name,
            account,
        })
    }

    /// Resolves the account a unified command runs against, and selects
    /// the backend serving it.
    ///
    /// A session calls this when a unified command first needs the client,
    /// so a `build` reaches neither.
    pub fn resolve(
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        backend: Backend,
    ) -> Result<Self> {
        let (config, name, account_config) = resolve_account(config_paths, account_name)?;

        Self::new(config, name, account_config, backend)
    }

    /// `err` as the backend's configuration explains it: a CalDAV 401
    /// to a block with no `auth` says the block configures none.
    pub fn explain(&self, err: Error) -> Error {
        match &self.config {
            #[cfg(feature = "caldav")]
            BackendConfig::Caldav(config) => {
                crate::caldav::client::unauthenticated(err, &config.auth)
            }
            #[allow(unreachable_patterns)]
            _ => err,
        }
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

    /// The name of the backend serving the account, as `--backend`
    /// spells it, known without opening it.
    pub fn backend_name(&self) -> &'static str {
        match &self.config {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendConfig::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendConfig::Vdir(_) => "vdir",
            #[cfg(feature = "caldav")]
            BackendConfig::Caldav(_) => "caldav",
            #[cfg(feature = "jmap")]
            BackendConfig::Jmap(_) => "jmap",
            #[cfg(feature = "pimdir")]
            BackendConfig::Pimdir(_) => "pimdir",
            #[cfg(feature = "gcal")]
            BackendConfig::Gcal(_) => "gcal",
            #[cfg(feature = "msgraph")]
            BackendConfig::Msgraph(_) => "msgraph",
        }
    }

    /// Fails with `capability-missing` when the backend does not serve one
    /// of `capabilities`, before anything connects.
    pub fn require(&self, capabilities: &[Capability]) -> Result<()> {
        capability::require(self.backend_name(), &self.configured, capabilities)
    }

    /// Whether the backend queues a write for a sync to push rather than
    /// make it on the server, so a write is confirmed as queued.
    pub fn queues_writes(&self) -> bool {
        match &self.config {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendConfig::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendConfig::Vdir(_) => false,
            #[cfg(feature = "caldav")]
            BackendConfig::Caldav(_) => false,
            #[cfg(feature = "jmap")]
            BackendConfig::Jmap(_) => false,
            #[cfg(feature = "pimdir")]
            BackendConfig::Pimdir(_) => true,
            #[cfg(feature = "gcal")]
            BackendConfig::Gcal(_) => false,
            #[cfg(feature = "msgraph")]
            BackendConfig::Msgraph(_) => false,
        }
    }

    /// The open backend, opening it when it is not.
    fn open(&mut self) -> Result<&mut BackendClient> {
        if self.inner.is_none() {
            debug!("opening the backend connection");
            self.inner = Some(self.config.open(&mut self.resolver)?);
        }

        Ok(self.inner.as_mut().expect("just opened"))
    }

    /// Refuses, before anything connects, a calendar id the backend cannot
    /// address and, when the call names one, an item id: on CalDAV an id
    /// io-webdav refuses, coded as a calendar or an item the server does not
    /// hold.
    #[cfg_attr(not(feature = "caldav"), allow(unused_variables))]
    fn check_ids(&self, calendar_id: &str, item_id: Option<&str>) -> Result<()> {
        match &self.config {
            #[cfg(feature = "caldav")]
            BackendConfig::Caldav(_) => {
                crate::caldav::client::check_calendar_id(calendar_id)?;
                item_id.map_or(Ok(()), crate::caldav::client::check_item_id)
            }
            #[allow(unreachable_patterns)]
            _ => Ok(()),
        }
    }

    /// Refuses, before anything connects, the id of a calendar to create
    /// the backend cannot address: on CalDAV, where the id is the
    /// collection's URL segment, one io-webdav refuses.
    #[cfg_attr(not(feature = "caldav"), allow(unused_variables))]
    fn check_new_id(&self, id: Option<&str>) -> Result<()> {
        match (&self.config, id) {
            #[cfg(feature = "caldav")]
            (BackendConfig::Caldav(_), Some(id)) => {
                crate::caldav::client::check_new_calendar_id(id)
            }
            #[allow(unreachable_patterns)]
            _ => Ok(()),
        }
    }

    /// Refuses, inside an MCP server without `--allow-send`, to `verb` an
    /// item whose `contents` or stored copy `item_id` names an attendee,
    /// whom the server or a sync may notify.
    ///
    /// The contents are checked first, so a refused document costs no
    /// read; the stored copy costs one.
    fn refuse_invitations(
        &mut self,
        verb: &str,
        calendar_id: &str,
        item_id: Option<&str>,
        contents: Option<&[u8]>,
    ) -> Result<()> {
        if !sending_closed() {
            return Ok(());
        }

        let mut invited = contents.is_some_and(invites);

        if !invited && let Some(item_id) = item_id {
            invited = invites(&self.get_item(calendar_id, item_id)?.contents);
        }

        if invited {
            bail!(
                "Cannot {verb} an item naming an attendee (ATTENDEE) without --allow-send: the \
                 server or a sync may notify them"
            );
        }

        Ok(())
    }

    /// Refuses, inside an MCP server without `--allow-send`, to delete the
    /// calendar `calendar_id` while one of its items names an attendee, or
    /// cannot be read to tell: deleting the calendar deletes the item.
    fn refuse_held_invitations(&mut self, calendar_id: &str) -> Result<()> {
        if !sending_closed() {
            return Ok(());
        }

        let listed = self.list_items(calendar_id, None, None, Default::default())?;
        let unread = !listed.errors.is_empty();

        if unread || listed.items.iter().any(|item| invites(&item.contents)) {
            bail!(
                "Cannot delete calendar `{calendar_id}` holding an item that names an attendee \
                 (ATTENDEE), or one that cannot be read to tell, without --allow-send: the \
                 server or a sync may notify them"
            );
        }

        Ok(())
    }

    /// Lists every calendar available to the active account.
    pub fn list_calendars(&mut self) -> Result<Vec<Calendar>> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_calendars(),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.list_calendars(),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_calendars(),
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
    /// it lets the caller name a collection, one it derives when `id` is
    /// `None`, a server-minted one elsewhere.
    pub fn create_calendar(
        &mut self,
        id: Option<&str>,
        name: &str,
        description: Option<&str>,
        color: Option<&str>,
    ) -> Result<String> {
        self.check_new_id(id)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.create_calendar(id, name, description, color),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_calendar(id, name, description, color),
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
        self.check_ids(id, None)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.update_calendar(id, patch),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.update_calendar(id, patch),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.update_calendar(id, patch),
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
        self.check_ids(id, None)?;
        self.refuse_held_invitations(id)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_calendar(id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.delete_calendar(id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_calendar(id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_calendar(id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.delete_calendar(id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_calendar(id),
        }
    }

    /// Lists one page of the items of `calendar_id` narrowed by `query`,
    /// saying whether an item follows it.
    ///
    /// The page is the 1-indexed `page`, the first by default, of
    /// `page_size` items, every item when the size is `None` or zero.
    pub fn list_items(
        &mut self,
        calendar_id: &str,
        page: Option<u32>,
        page_size: Option<u32>,
        mut query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemPage> {
        self.check_ids(calendar_id, None)?;

        let page_size = page_size.filter(|size| *size > 0).map(|size| size as usize);

        if let Some(size) = page_size {
            let page = page.unwrap_or(1).max(1) as usize;
            query.skip = (page - 1).saturating_mul(size);
            // NOTE: one item past the page tells whether another follows.
            query.take = Some(size.saturating_add(1));
        }

        let CalendarItemList {
            mut items,
            mut errors,
        } = self.list_window(calendar_id, query)?;
        let more = page_size.is_some_and(|size| items.len() > size);

        if let Some(size) = page_size
            && items.len() > size
        {
            // NOTE: the item past the page is the next page's, and so is
            // an entry naming it.
            let past: Vec<String> = items.drain(size..).map(|item| item.id).collect();
            errors.retain(|error| error.id.as_ref().is_none_or(|id| !past.contains(id)));
        }

        Ok(CalendarItemPage {
            items,
            more,
            errors,
        })
    }

    /// Lists the items of `calendar_id` narrowed by `query`.
    ///
    /// A backend SHALL narrow by kind and by range before it windows, so a
    /// page of a component family holds that family. A server-backed
    /// backend pushes both down where its protocol defines such a filter;
    /// the others parse and filter after the fact.
    fn list_window(
        &mut self,
        calendar_id: &str,
        query: CalendarItemQuery<'_>,
    ) -> Result<CalendarItemList> {
        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.list_items(calendar_id, query),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.list_items(calendar_id, query),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.list_items(calendar_id, query),
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
        self.check_ids(calendar_id, Some(item_id))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.get_item(calendar_id, item_id),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.get_item(calendar_id, item_id),
        }
    }

    /// Stores raw iCalendar bytes as a new item, returning the id assigned.
    ///
    /// `send_updates` says whom a gcal write mails; the command refuses it
    /// before reaching any other backend, which ignores it.
    /// `origin` says whether a pimdir write queues the body whole or
    /// without its `X-PIMDIR-` properties; every other backend sends it as
    /// given.
    pub fn create_item(
        &mut self,
        calendar_id: &str,
        contents: Vec<u8>,
        origin: ItemOrigin,
        send_updates: Option<SendUpdates>,
    ) -> Result<String> {
        // NOTE: read by gcal alone, the commands refusing it elsewhere.
        #[cfg(not(feature = "gcal"))]
        let _ = send_updates;
        #[cfg(not(feature = "pimdir"))]
        let _ = origin;

        self.check_ids(calendar_id, None)?;
        self.refuse_invitations("create", calendar_id, None, Some(&contents))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.create_item(calendar_id, contents),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.create_item(calendar_id, contents, origin),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => client.create_item(calendar_id, contents, send_updates),
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.create_item(calendar_id, contents),
        }
    }

    /// Replaces the contents of `item_id` inside `calendar_id`.
    ///
    /// `if_match` is the entity tag to gate the write on, `None` overwriting
    /// unconditionally. A backend with no guard concept ignores it.
    /// `send_updates` is read by gcal alone, as on
    /// [`create_item`](Self::create_item), and `origin` by pimdir
    /// alone. `read` is the body `if_match` was read with when calendula's
    /// own read gave it, `None` for the caller's `--if-match`, read by
    /// msgraph alone.
    #[allow(clippy::too_many_arguments)]
    pub fn update_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        contents: Vec<u8>,
        if_match: Option<&str>,
        read: Option<&[u8]>,
        origin: ItemOrigin,
        send_updates: Option<SendUpdates>,
    ) -> Result<()> {
        // NOTE: read by gcal alone, the commands refusing it elsewhere.
        #[cfg(not(feature = "gcal"))]
        let _ = send_updates;
        #[cfg(not(feature = "pimdir"))]
        let _ = origin;
        #[cfg(not(feature = "msgraph"))]
        let _ = read;

        self.check_ids(calendar_id, Some(item_id))?;
        self.refuse_invitations("update", calendar_id, Some(item_id), Some(&contents))?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
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
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => {
                client.update_item(calendar_id, item_id, contents, if_match)
            }
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => {
                client.update_item(calendar_id, item_id, contents, if_match, origin)
            }
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => {
                client.update_item(calendar_id, item_id, contents, if_match, send_updates)
            }
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => {
                client.update_item(calendar_id, item_id, contents, if_match, read)
            }
        }
    }

    /// Deletes `item_id` from `calendar_id`.
    ///
    /// `if_match` gates the delete as it gates [`update_item`](Self::update_item);
    /// a backend with no such guard refuses it rather than pretending the
    /// check happened. `send_updates` is read by gcal alone, as on
    /// [`create_item`](Self::create_item).
    pub fn delete_item(
        &mut self,
        calendar_id: &str,
        item_id: &str,
        if_match: Option<&str>,
        send_updates: Option<SendUpdates>,
    ) -> Result<()> {
        // NOTE: read by gcal alone, the commands refusing it elsewhere.
        #[cfg(not(feature = "gcal"))]
        let _ = send_updates;

        self.check_ids(calendar_id, Some(item_id))?;
        self.refuse_invitations("delete", calendar_id, Some(item_id), None)?;

        match self.open()? {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
                feature = "pimdir",
                feature = "gcal",
                feature = "msgraph"
            )))]
            BackendClient::Unused(never) => match *never {},
            #[cfg(feature = "vdir")]
            BackendClient::Vdir(client) => client.delete_item(calendar_id, item_id, if_match),
            #[cfg(feature = "caldav")]
            BackendClient::Caldav(client) => client.delete_item(calendar_id, item_id, if_match),
            #[cfg(feature = "jmap")]
            BackendClient::Jmap(client) => client.delete_item(calendar_id, item_id, if_match),
            #[cfg(feature = "pimdir")]
            BackendClient::Pimdir(client) => client.delete_item(calendar_id, item_id, if_match),
            #[cfg(feature = "gcal")]
            BackendClient::Gcal(client) => {
                client.delete_item(calendar_id, item_id, if_match, send_updates)
            }
            #[cfg(feature = "msgraph")]
            BackendClient::Msgraph(client) => client.delete_item(calendar_id, item_id, if_match),
        }
    }
}

impl BackendConfig {
    /// Opens the backend this configuration describes, its credentials
    /// resolved through `resolver`.
    #[cfg_attr(
        not(any(
            feature = "caldav",
            feature = "jmap",
            feature = "gcal",
            feature = "msgraph"
        )),
        allow(unused_variables)
    )]
    fn open(&self, resolver: &mut SecretResolver) -> Result<BackendClient> {
        match self {
            #[cfg(not(any(
                feature = "vdir",
                feature = "caldav",
                feature = "jmap",
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
                let client = CaldavBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Caldav(Box::new(client)))
            }
            #[cfg(feature = "jmap")]
            Self::Jmap(config) => {
                use crate::jmap::backend::JmapBackend;
                let client = JmapBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Jmap(Box::new(client)))
            }
            #[cfg(feature = "gcal")]
            Self::Gcal(config) => {
                use crate::gcal::backend::GcalBackend;
                let client = GcalBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Gcal(Box::new(client)))
            }
            #[cfg(feature = "msgraph")]
            Self::Msgraph(config) => {
                use crate::msgraph::backend::MsgraphBackend;
                let client = MsgraphBackend::new(config.as_ref().clone(), resolver)?;
                Ok(BackendClient::Msgraph(Box::new(client)))
            }
        }
    }
}

/// Cuts the window `skip` and `take` name out of a whole list.
#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "gcal",
    feature = "msgraph",
    all(test, feature = "pimdir")
))]
pub fn window<T>(items: Vec<T>, skip: usize, take: Option<usize>) -> Vec<T> {
    let items = items.into_iter().skip(skip);
    items.take(take.unwrap_or(usize::MAX)).collect()
}

#[cfg(all(test, feature = "vdir"))]
mod tests {
    use std::fs;

    use crate::{
        backend::Backend,
        config::{AccountConfig, Config, VdirConfig},
        shared::{client::CalendarClient, item::CalendarItemQuery},
    };

    /// The ids page `page` of `size` holds among five items, comma-joined,
    /// and whether an item follows it.
    fn page(page: Option<u32>, size: Option<u32>) -> (String, bool) {
        let dir = tempfile::tempdir().unwrap();
        let cal = dir.path().join("cal");
        fs::create_dir(&cal).unwrap();

        for id in ["1", "2", "3", "4", "5"] {
            fs::write(cal.join(format!("{id}.ics")), "BEGIN:VCALENDAR\r\n").unwrap();
        }

        let account = AccountConfig {
            vdir: Some(VdirConfig {
                root: dir.path().to_path_buf(),
            }),
            ..Default::default()
        };
        let mut client =
            CalendarClient::new(Config::default(), "a".into(), account, Backend::Auto).unwrap();

        let query = CalendarItemQuery::default();
        let page = client.list_items("cal", page, size, query).unwrap();
        let ids: Vec<String> = page.items.into_iter().map(|item| item.id).collect();
        (ids.join(","), page.more)
    }

    #[test]
    fn a_page_says_whether_an_item_follows_it() {
        assert_eq!(page(None, None), ("1,2,3,4,5".into(), false));
        assert_eq!(page(Some(1), Some(2)), ("1,2".into(), true));
        assert_eq!(page(Some(2), Some(2)), ("3,4".into(), true));
        assert_eq!(page(Some(3), Some(2)), ("5".into(), false));
        assert_eq!(page(Some(1), Some(5)), ("1,2,3,4,5".into(), false));
        assert_eq!(page(Some(9), Some(2)), ("".into(), false));
        assert_eq!(page(None, Some(0)), ("1,2,3,4,5".into(), false));

        // NOTE: a page of zero is clamped to the first page rather than
        // underflowing the skip.
        assert_eq!(page(Some(0), Some(2)), ("1,2".into(), true));
    }
}

#[cfg(all(test, unix, feature = "caldav"))]
mod caldav_tests {
    use std::fs;

    use crate::{backend::Backend, config::Config, shared::client::CalendarClient};

    /// A backend opened twice, the first open failing past its credential,
    /// resolves that credential once: a reopened backend runs no secret
    /// command again.
    #[test]
    fn a_reopened_backend_runs_its_secret_command_once() {
        let dir = tempfile::tempdir().unwrap();
        let runs = dir.path().join("runs");
        let command = format!("echo run >> '{}'; echo secret", runs.display());

        // NOTE: port 1 on the loopback refuses the connection at once.
        let toml = format!(
            "[accounts.a]\ncaldav.home = \"http://127.0.0.1:1/\"\n\
             caldav.auth.basic.username = \"a\"\n\
             caldav.auth.basic.password.command = {command:?}\n"
        );
        let mut config: Config = toml::from_str(&toml).unwrap();
        let account = config.accounts.remove("a").unwrap();
        let mut client = CalendarClient::new(config, "a".into(), account, Backend::Auto).unwrap();

        assert!(client.list_calendars().is_err());
        assert!(client.list_calendars().is_err());
        assert_eq!(fs::read_to_string(&runs).unwrap(), "run\n");
    }
}

#[cfg(all(test, feature = "msgraph"))]
impl CalendarClient {
    /// A client of a Graph account already open on `backend`, whose
    /// server a test scripts.
    pub(crate) fn over_msgraph(backend: crate::msgraph::backend::MsgraphBackend) -> Self {
        let toml = "[accounts.a]\nmsgraph.auth.token.raw = \"made-up\"\n";
        let mut config: Config = toml::from_str(toml).unwrap();
        let account = config.accounts.remove("a").unwrap();
        let mut client = Self::new(config, "a".into(), account, Backend::Auto).unwrap();
        client.inner = Some(BackendClient::Msgraph(Box::new(backend)));
        client
    }

    /// The Graph backend the client holds open, to read what it sent.
    pub(crate) fn into_msgraph(self) -> crate::msgraph::backend::MsgraphBackend {
        match self.inner {
            Some(BackendClient::Msgraph(backend)) => *backend,
            _ => panic!("no Graph backend open"),
        }
    }
}
