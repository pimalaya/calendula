//! # Google Calendar client
//!
//! calendula's wrapper around [`io_gcal`]'s std client, building a connected
//! client from a [`GcalConfig`].
//!
//! Resolves the bearer token, running its command when it is one, then opens
//! the TLS connection through pimalaya-stream. There is no discovery step:
//! the Calendar API lives at one well-known base URL.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_gcal::v3::client::{GcalClientStd, GcalClientStdConnectOptions};
use pimalaya_cli::printer::Printer;
use pimalaya_config::{secret::SecretResolver, toml::TomlConfig};
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;

use crate::{
    account::context::Account,
    cli::load_config,
    config::{GcalConfig, default_gcal_alpn},
};

/// A connected client bundled with the merged runtime [`Account`].
pub struct GcalClient {
    inner: GcalClientStd,
    /// The account the protocol-specific subcommands take defaults from.
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

/// Loads the configuration, picks the active account, then connects.
///
/// Bails when the account carries no `[gcal]` block.
pub fn build_gcal_client(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<GcalClient> {
    let mut config = load_config(printer, config_paths)?;
    let (name, mut account_config) = config
        .take_account(account_name)?
        .ok_or_else(|| anyhow!("Cannot find account"))?;

    let gcal_config = account_config
        .gcal
        .take()
        .ok_or_else(|| anyhow!("Google Calendar configuration is missing for account `{name}`"))?;

    let account = Account::from(config).merge(Account::from(account_config));
    let inner = connect(&gcal_config, &mut SecretResolver::new())?;

    Ok(GcalClient::new(inner, account))
}

/// Opens a connected Google Calendar client, resolving the bearer token
/// through `resolver` so a command another backend of the same account
/// names too is spawned once.
pub fn connect(config: &GcalConfig, resolver: &mut SecretResolver) -> Result<GcalClientStd> {
    let token = resolver.resolve(config.auth.token.clone())?;
    let options = GcalClientStdConnectOptions {
        tls: build_tls(config),
    };

    Ok(GcalClientStd::connect(token.expose_secret(), options)?)
}

/// The TLS profile the backend connects with.
///
/// The account's `gcal.alpn` wins, else [`default_gcal_alpn`] pins the
/// HTTP/1.1 io-http is the only client for.
fn build_tls(config: &GcalConfig) -> Tls {
    let alpn = config.alpn.clone().unwrap_or_else(default_gcal_alpn);
    config.tls.clone().into_tls(alpn)
}
