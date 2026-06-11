//! Calendula wrapper around [`io_webdav::client::WebdavClientStd`].
//!
//! Builds a connected CalDAV client from a [`CaldavConfig`] block:
//! resolves the server URI from one of the three configured paths
//! (server-uri, discover.host, or home-uri), opens the TCP/TLS
//! connection via pimalaya-stream, then optionally walks the RFC 6764
//! well-known + RFC 5397 principal + RFC 4791 calendar-home-set
//! discovery chain.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use io_http::{rfc6750::bearer::HttpAuthBearer, rfc7617::basic::HttpAuthBasic};
use io_webdav::{client::WebdavClientStd, rfc4918::WebdavAuth};
use pimalaya_config::toml::TomlConfig;
use pimalaya_stream::tls::Tls;
use pimconf::rfc6764::{client::DiscoveryWebdavClientStd, types::DavService};
use secrecy::ExposeSecret;
use url::Url;

use crate::{
    account::context::Account,
    cli::load_or_wizard,
    config::{CaldavAuthConfig, CaldavConfig},
};

const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

pub struct CaldavClient {
    inner: WebdavClientStd,
    pub account: Account,
}

impl CaldavClient {
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

/// Opens a connected CalDAV client and walks the discovery chain so its
/// caches are populated, following whichever of the three config routes
/// is set:
///
/// 1. `home`: pre-resolved calendar home set; no discovery runs.
/// 2. `server`: principal + calendar-home-set discovery start from the
///    given context root.
/// 3. `discover`: a bare domain resolved to a context root through
///    pimconf (RFC 6764 SRV + `.well-known`) before that walk.
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
                .ok_or_else(|| anyhow!("CalDAV config needs `server`, `home`, or `discover`"))?;
            resolve_server(domain, &tls)?
        }
    };

    let mut client = WebdavClientStd::connect(&server, &tls, auth)?;
    client.calendar_home_set()?;

    Ok(client)
}

/// Resolves a bare domain to a CalDAV context root via pimconf
/// (RFC 6764 SRV + `.well-known`), reusing `tls` for the `.well-known`
/// probe.
fn resolve_server(domain: &str, tls: &Tls) -> Result<Url> {
    let resolver = Url::parse(DEFAULT_RESOLVER).expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryWebdavClientStd::new(resolver).with_tls(tls.clone());
    let server = client.resolve(domain, DavService::Caldav)?;
    Ok(server)
}

fn build_tls(config: &CaldavConfig) -> Tls {
    let mut tls: Tls = config.tls.clone().into();
    tls.rustls.alpn = vec!["http/1.1".into()];
    tls
}

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
/// CalDAV client. Bails when the account has no `[caldav]` block.
pub fn build_caldav_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<CaldavClient> {
    let mut config = load_or_wizard(config_paths)?;
    let (name, mut ac) = config
        .take_account(account_name)?
        .ok_or_else(|| anyhow!("Cannot find account"))?;
    let caldav_config = ac
        .caldav
        .take()
        .ok_or_else(|| anyhow!("CalDAV config is missing for account `{name}`"))?;
    let account = Account::from(config).merge(Account::from(ac));
    let inner = connect_and_resolve(&caldav_config)?;
    Ok(CaldavClient::new(inner, account))
}
