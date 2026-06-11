//! Cross-protocol [`CalendarClient`] for the shared subcommands
//! (`calendars`, `events`, `items`).
//!
//! Wraps [`io_calendar::client::CalendarClientStd`] and bundles the
//! active [`Account`] alongside the I/O client. Implements
//! [`Deref`]/[`DerefMut`] onto the inner client so callers can call
//! its methods directly.
//!
//! [`CalendarClientStd`] is an enum over the single active backend, so
//! construction picks the first storage backend (`vdir`, then `caldav`)
//! allowed by the [`Backend`] flag that is configured on the account,
//! then wraps it via the per-backend `From` impls.

use std::ops::{Deref, DerefMut};

use anyhow::{Result, bail};

use crate::{
    account::context::Account,
    backend::Backend,
    config::{AccountConfig, Config},
};

pub struct CalendarClient {
    inner: io_calendar::client::CalendarClientStd,
    pub account: Account,
}

impl CalendarClient {
    pub fn new(
        config: Config,
        #[allow(unused_mut)] mut account_config: AccountConfig,
        #[allow(unused)] backend: Backend,
    ) -> Result<Self> {
        use io_calendar::client::CalendarClientStd;

        let mut inner: Option<CalendarClientStd> = None;

        #[cfg(feature = "vdir")]
        if inner.is_none() && backend.allows_vdir() {
            if let Some(vdir_config) = account_config.vdir.take() {
                let client = crate::vdir::client::build(&vdir_config);
                inner = Some(CalendarClientStd::from(client));
            }
        }

        #[cfg(feature = "caldav")]
        if inner.is_none() && backend.allows_caldav() {
            if let Some(caldav_config) = account_config.caldav.take() {
                let inner_client = crate::caldav::client::connect_and_resolve(&caldav_config)?;
                let client = io_calendar::webdav::client::WebdavClientStd::new(inner_client);
                inner = Some(CalendarClientStd::from(client));
            }
        }

        let Some(inner) = inner else {
            bail!("No backend matching `{backend}` is configured for this account");
        };

        let account = Account::from(config).merge(Account::from(account_config));

        Ok(Self { inner, account })
    }
}

impl Deref for CalendarClient {
    type Target = io_calendar::client::CalendarClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CalendarClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
