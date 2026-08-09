//! calendula wrapper around [`io_gcal`]'s std client.
//!
//! Builds a connected Google Calendar client from a [`GcalConfig`]:
//! resolves the bearer token (running its command when it is one) and
//! opens the TLS connection to the fixed API endpoint through
//! pimalaya-stream. There is no discovery step: the Calendar API lives
//! at one well-known base URL.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_gcal::v3::client::{GcalClientStd, GcalClientStdConnectOptions};
use pimalaya_config::toml::TomlConfig;
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;

use crate::{account::context::Account, cli::load_config, config::GcalConfig};

/// A connected Google Calendar client bundled with the merged runtime
/// [`Account`], for the protocol-specific subcommands.
pub struct GcalClient {
    inner: GcalClientStd,
    pub account: Account,
}

impl GcalClient {
    /// Wraps an already-connected client.
    pub fn new(inner: GcalClientStd, account: Account) -> Self {
        Self { inner, account }
    }
}

impl Deref for GcalClient {
    type Target = GcalClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GcalClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loads the configuration, picks the active account, then opens the
/// Calendar client. Bails when the account carries no `[gcal]` block.
pub fn build_gcal_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<GcalClient> {
    let mut config = load_config(config_paths)?;
    let (name, mut account_config) = config
        .take_account(account_name)?
        .ok_or_else(|| anyhow!("Cannot find account"))?;

    let gcal_config = account_config
        .gcal
        .take()
        .ok_or_else(|| anyhow!("Google Calendar configuration is missing for account `{name}`"))?;

    let account = Account::from(config).merge(Account::from(account_config));
    let inner = connect(&gcal_config)?;

    Ok(GcalClient::new(inner, account))
}

/// Opens a connected Google Calendar client.
pub fn connect(config: &GcalConfig) -> Result<GcalClientStd> {
    let token = config.auth.token.clone().get()?;
    let options = GcalClientStdConnectOptions {
        tls: build_tls(config),
    };

    Ok(GcalClientStd::connect(token.expose_secret(), options)?)
}

/// The TLS profile the backend connects with. io-http speaks HTTP/1.1
/// only, so the ALPN list pins it rather than letting Google negotiate
/// HTTP/2.
fn build_tls(config: &GcalConfig) -> Tls {
    let mut tls: Tls = config.tls.clone().into();
    tls.rustls.alpn = vec!["http/1.1".into()];
    tls
}
