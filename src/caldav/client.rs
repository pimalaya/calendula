//! calendula wrapper around [`io_webdav`]'s std client.
//!
//! Builds a connected CalDAV client from a [`CaldavConfig`]: resolves
//! the calendar home-set through whichever of the three configured
//! routes is set, opens the TCP or TLS connection through
//! pimalaya-stream, and leaves the client's discovery caches populated
//! so a command runs no extra round-trip.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_http::{rfc6750::bearer::HttpAuthBearer, rfc7617::basic::HttpAuthBasic};
use io_pim_discovery::{
    rfc6764::{client::DiscoveryWebdavClientStd, service::DiscoveryDavService},
    shared::dns::system_resolver,
};
use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
use pimalaya_config::toml::TomlConfig;
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    account::context::Account,
    cli::load_config,
    config::{CaldavAuthConfig, CaldavConfig},
};

/// DNS-over-TCP resolver used when no system resolver is found:
/// Cloudflare's `1.1.1.1`.
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// A connected CalDAV client bundled with the merged runtime
/// [`Account`], for the protocol-specific subcommands.
pub struct CaldavClient {
    inner: WebdavClientStd,
    pub account: Account,
}

impl CaldavClient {
    /// Wraps an already-connected client.
    pub fn new(inner: WebdavClientStd, account: Account) -> Self {
        Self { inner, account }
    }
}

impl Deref for CaldavClient {
    type Target = WebdavClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for CaldavClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Opens a connected CalDAV client and walks the discovery chain,
/// following whichever of the three configured routes is set:
///
/// 1. `home` pins the calendar home-set, so no discovery runs;
/// 2. `server` names the context root the principal and home-set walk
///    starts from;
/// 3. `discover` resolves a bare domain to that context root first,
///    through RFC 6764 SRV records and the `.well-known` path.
pub fn connect_and_resolve(config: &CaldavConfig) -> Result<WebdavClientStd> {
    let auth = build_auth(&config.auth)?;
    let tls = build_tls(config);

    if let Some(home) = &config.home {
        let mut client = WebdavClientStd::connect(home, &tls, auth)?;
        client.calendar_home_set = Some(home.clone());
        return Ok(client);
    }

    let server = match &config.server {
        Some(server) => server.clone(),
        None => {
            let domain = config
                .discover
                .as_ref()
                .ok_or_else(|| anyhow!("CalDAV config needs `discover`, `server` or `home`"))?;
            resolve_server(domain, &tls)?
        }
    };

    let mut client = WebdavClientStd::connect(&server, &tls, auth)?;
    client.calendar_home_set()?;

    Ok(client)
}

/// Resolves a bare domain to a CalDAV context root through RFC 6764,
/// reusing `tls` for the `.well-known` probe.
fn resolve_server(domain: &str, tls: &Tls) -> Result<Url> {
    let mut client = DiscoveryWebdavClientStd::new(resolver()).with_tls(tls.clone());
    Ok(client.resolve(domain, DiscoveryDavService::Caldav)?)
}

/// The resolver discovery queries: the system one, so the domain is not
/// leaked to a third party, falling back to the Cloudflare default on
/// hosts exposing none.
pub fn resolver() -> Url {
    system_resolver().unwrap_or_else(|| {
        DEFAULT_RESOLVER
            .parse()
            .expect("DEFAULT_RESOLVER must be a valid URL")
    })
}

/// The TLS profile CalDAV connects with. WebDAV speaks HTTP/1.1 only,
/// so the ALPN list pins it rather than letting a server negotiate
/// HTTP/2.
fn build_tls(config: &CaldavConfig) -> Tls {
    let mut tls: Tls = config.tls.clone().into();
    tls.rustls.alpn = vec!["http/1.1".into()];
    tls
}

/// Resolves the configured credentials into the wire auth io-webdav
/// sends. A secret backed by a command is run here, at first use.
fn build_auth(config: &CaldavAuthConfig) -> Result<WebdavAuth> {
    Ok(match config {
        CaldavAuthConfig::None => WebdavAuth::None,
        CaldavAuthConfig::Basic { username, password } => WebdavAuth::Basic(HttpAuthBasic {
            username: username.clone(),
            password: password.clone().get()?,
        }),
        CaldavAuthConfig::Bearer { token } => {
            let token = token.clone().get()?;
            WebdavAuth::Bearer(HttpAuthBearer::new(token.expose_secret()))
        }
    })
}

/// Loads the configuration, picks the active account, then opens the
/// CalDAV client. Bails when the account carries no `[caldav]` block.
pub fn build_caldav_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<CaldavClient> {
    let mut config = load_config(config_paths)?;
    let (name, mut account_config) = config
        .take_account(account_name)?
        .ok_or_else(|| anyhow!("Cannot find account"))?;

    let caldav_config = account_config
        .caldav
        .take()
        .ok_or_else(|| anyhow!("CalDAV configuration is missing for account `{name}`"))?;

    let account = Account::from(config).merge(Account::from(account_config));
    let inner = connect_and_resolve(&caldav_config)?;

    Ok(CaldavClient::new(inner, account))
}
