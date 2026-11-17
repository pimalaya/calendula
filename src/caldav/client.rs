//! # CalDAV client
//!
//! calendula's wrapper around [`io_webdav`]'s std client. Builds a
//! connected client from a [`CaldavConfig`], leaving its discovery
//! caches populated so a command runs no extra round-trip.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use io_http::{rfc6750::bearer::HttpAuthBearer, rfc7617::basic::HttpAuthBasic};
use io_webdav::{
    client::{WebdavClientStd, WebdavClientStdConnectOptions, WebdavClientStdError},
    rfc4918::{
        WebdavAuth, WebdavIdError, check_collection_id, check_resource_id,
        follow_redirects::WebdavFollowRedirectsError, send::WebdavSendError,
    },
    rfc6578::sync_collection::WebdavSyncCollectionError,
};
use pimalaya_cli::error::with_code;
use pimalaya_config::secret::SecretResolver;
use pimalaya_stream::tls::Tls;
use secrecy::ExposeSecret;
use url::{ParseError, Url};

use crate::{
    cli::resolve_account,
    config::{CaldavAuthConfig, CaldavConfig, ProxyConfig, default_caldav_alpn, parse_url},
    error::{ErrorCode, recoded},
};

/// A connected CalDAV client, for the protocol-specific subcommands.
pub struct CaldavClient {
    inner: WebdavClientStd,
}

impl CaldavClient {
    /// Wraps an already-connected client.
    pub fn new(inner: WebdavClientStd) -> Self {
        Self { inner }
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
/// resolving the credential through `resolver` so a command another
/// backend of the same account names too is spawned once.
///
/// Follows whichever route the config sets: `home` pins the home-set
/// and skips discovery, `server` names the context root the principal
/// walk starts from.
pub fn connect(config: &CaldavConfig, resolver: &mut SecretResolver) -> Result<WebdavClientStd> {
    let auth = build_auth(&config.auth, resolver)?;
    let tls = build_tls(config);
    let proxy = ProxyConfig::resolve(config.proxy.clone(), resolver)?;

    if let Some(home) = &config.home {
        let opts = WebdavClientStdConnectOptions { tls, proxy };
        let mut client = WebdavClientStd::connect(home, auth, opts)?;
        client.calendar_home_set = Some(home.clone());
        return Ok(client);
    }

    let server = config
        .server
        .as_ref()
        .ok_or_else(|| anyhow!("CalDAV config needs `server` or `home`"))?;
    let server = parse_server(server)?;

    let opts = WebdavClientStdConnectOptions { tls, proxy };
    let mut client = WebdavClientStd::connect(&server, auth, opts)?;
    client.calendar_home_set()?;

    Ok(client)
}

/// Parses a `server` config string into a [`Url`].
///
/// Accepts a full URL, a bare domain, or `domain:port`; anything
/// without an explicit `http` or `https` scheme defaults to `https://`,
/// since `url` would otherwise read the leading label of `domain:port`
/// as the scheme. A URL that fails to parse is refused rather than
/// prefixed, a host that is not ASCII naming its Punycode form
/// ([`parse_url`]).
pub fn parse_server(server: &str) -> Result<Url> {
    match Url::parse(server) {
        Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(url),
        Ok(_) | Err(ParseError::RelativeUrlWithoutBase) => parse_url(&format!("https://{server}")),
        Err(_) => parse_url(server),
    }
}

/// The TLS profile CalDAV connects with.
///
/// The account's `caldav.alpn` wins, else [`default_caldav_alpn`] pins the
/// HTTP/1.1 io-http is the only client for.
fn build_tls(config: &CaldavConfig) -> Tls {
    let alpn = config.alpn.clone().unwrap_or_else(default_caldav_alpn);
    config.tls.clone().into_tls(alpn)
}

/// Resolves the configured credentials into the auth io-webdav sends.
///
/// A secret backed by a command is run here, at first use, unless
/// `resolver` already holds what that same command answered.
fn build_auth(config: &CaldavAuthConfig, resolver: &mut SecretResolver) -> Result<WebdavAuth> {
    Ok(match config {
        CaldavAuthConfig::None => WebdavAuth::None,
        CaldavAuthConfig::Basic { username, password } => WebdavAuth::Basic(HttpAuthBasic {
            username: username.clone(),
            password: resolver.resolve(password.clone())?,
        }),
        CaldavAuthConfig::Bearer { token } => {
            let token = resolver.resolve(token.clone())?;
            WebdavAuth::Bearer(HttpAuthBearer::new(token.expose_secret()))
        }
    })
}

/// The status and body of an answer that refused an exchange, wherever
/// io-webdav surfaces it.
pub(crate) fn refused(err: &WebdavClientStdError) -> Option<(u16, &str)> {
    match err {
        WebdavClientStdError::Send(WebdavSendError::HttpStatus { status, body })
        | WebdavClientStdError::WebdavSyncCollection(WebdavSyncCollectionError::Send(
            WebdavSendError::HttpStatus { status, body },
        ))
        | WebdavClientStdError::WebdavFollowRedirects(WebdavFollowRedirectsError::HttpStatus {
            status,
            body,
        }) => Some((*status, body)),
        _ => None,
    }
}

/// `err` under a message saying the account configures no `auth`, when
/// the server answered a request 401 (RFC 9110 §15.5.2) and `auth` sent
/// no credential; any other failure is `err` unchanged, its code kept.
pub fn unauthenticated(err: anyhow::Error, auth: &CaldavAuthConfig) -> anyhow::Error {
    let asked = |cause: &(dyn std::error::Error + 'static)| {
        cause
            .downcast_ref::<WebdavClientStdError>()
            .and_then(refused)
            .is_some_and(|(status, _)| status == 401)
    };

    if !matches!(auth, CaldavAuthConfig::None) || !err.chain().any(asked) {
        return err;
    }

    err.context(
        "CalDAV server answered 401: it asks for credentials, and the account configures no `auth`",
    )
}

/// The code of a CalDAV refusal in an error chain.
///
/// A 507, or a body naming `quota-not-exceeded` (RFC 4331 §6), is
/// `quota-exceeded`; a 429 `rate-limited`; a 403 `permission-denied` when
/// its body names `need-privileges` (RFC 3744 §7.1.1) or no precondition at
/// all. A 400 or a 403 refusing the iCalendar written ([`rejects_item`]) is
/// `item-invalid`; any other 403 names a precondition the written body
/// failed (RFC 4791 §5.3.2.1), no matter of rights. A 412 is
/// `precondition-failed`; a `no-uid-conflict` `item-duplicate`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let err = cause.downcast_ref::<WebdavClientStdError>()?;

    if err.is_duplicate_uid() {
        return Some(ErrorCode::ItemDuplicate);
    }

    let (status, body) = refused(err)?;
    status_code(status, body)
}

/// The code of a listing row the server answered with `status` and no
/// body, as [`refusal_code`] reads a refusal: a 403 `permission-denied`.
pub(crate) fn row_code(status: u16) -> Option<ErrorCode> {
    status_code(status, "")
}

/// The code of a refusal of `status` answered with `body`.
fn status_code(status: u16, body: &str) -> Option<ErrorCode> {
    match status {
        _ if body.contains("quota-not-exceeded") => Some(ErrorCode::QuotaExceeded),
        507 => Some(ErrorCode::QuotaExceeded),
        429 => Some(ErrorCode::RateLimited),
        403 if body.contains("need-privileges") => Some(ErrorCode::PermissionDenied),
        _ if rejects_item(status, body) => Some(ErrorCode::ItemInvalid),
        403 if !names_precondition(body) => Some(ErrorCode::PermissionDenied),
        412 => Some(ErrorCode::PreconditionFailed),
        _ => None,
    }
}

/// Whether a refusal of `status` answered with `body` refuses the
/// iCalendar written rather than the request: a 400 or a 403 whose body
/// names a precondition on its content (RFC 4791 §5.3.2.1:
/// `supported-calendar-data`, `valid-calendar-data`,
/// `valid-calendar-object-resource`, `supported-calendar-component`), or
/// the iCalendar itself, as cardamum reads a refused vCard.
pub(crate) fn rejects_item(status: u16, body: &str) -> bool {
    let body = body.to_ascii_lowercase();
    let names_item = [
        "supported-calendar-data",
        "valid-calendar-data",
        "valid-calendar-object-resource",
        "supported-calendar-component",
        "icalendar",
        "vcalendar",
    ];

    matches!(status, 400 | 403) && names_item.iter().any(|name| body.contains(name))
}

/// Whether a refusal's body is a `DAV:error` element, which names the
/// precondition the request failed (RFC 4918 §16).
fn names_precondition(body: &str) -> bool {
    body.contains(":error") || body.contains("<error")
}

/// The failure of a request naming one item: a 404 or a 410 is
/// `item-not-found`, the item gone or never there.
pub fn item_refusal(err: WebdavClientStdError) -> anyhow::Error {
    match err {
        WebdavClientStdError::Send(WebdavSendError::HttpStatus {
            status: 404 | 410, ..
        }) => recoded(ErrorCode::ItemNotFound, err),
        err => err.into(),
    }
}

/// Refuses a calendar id no collection can have, one io-webdav's
/// `check_collection_id` refuses (empty, or holding an empty, `.` or `..`
/// segment), before anything connects: `calendar-not-found`, as vdir
/// refuses an id that is not one directory name, io-webdav's reason in
/// `sources`.
pub(crate) fn check_calendar_id(id: &str) -> Result<()> {
    check_collection_id(id).map_err(|err| {
        not_found(err, ErrorCode::CalendarNotFound).context(format!("Calendar `{id}` not found"))
    })
}

/// Refuses an item id no resource of a calendar can have, one io-webdav's
/// `check_resource_id` refuses, before anything connects: `item-not-found`,
/// io-webdav's reason in `sources`.
pub(crate) fn check_item_id(id: &str) -> Result<()> {
    check_resource_id(id).map_err(|err| {
        not_found(err, ErrorCode::ItemNotFound).context(format!("Item `{id}` not found"))
    })
}

/// Refuses the id of a calendar to create that no collection can have,
/// before anything connects: uncoded, as vdir refuses a new id that is not
/// one directory name, io-webdav's reason in `sources`.
pub(crate) fn check_new_calendar_id(id: &str) -> Result<()> {
    check_collection_id(id)
        .with_context(|| format!("Calendar id `{id}` names no CalDAV collection"))
}

/// io-webdav's refusal of an id, carrying `code`.
fn not_found(err: WebdavIdError, code: ErrorCode) -> anyhow::Error {
    with_code(err.into(), |_| Some(code))
}

/// Loads the configuration, picks the active account and takes its
/// `[caldav]` block, so a command checks its ids before anything connects.
///
/// Bails when the account carries no `[caldav]` block.
pub fn caldav_config(config_paths: &[PathBuf], account_name: Option<&str>) -> Result<CaldavConfig> {
    let (_, name, mut account_config) = resolve_account(config_paths, account_name)?;

    account_config
        .caldav
        .take()
        .ok_or_else(|| anyhow!("CalDAV configuration is missing for account `{name}`"))
}

/// Connects the client the `caldav` commands run against.
pub fn build_caldav_client(config: &CaldavConfig) -> Result<CaldavClient> {
    let inner = connect(config, &mut SecretResolver::new())?;
    Ok(CaldavClient::new(inner))
}

#[cfg(test)]
mod tests {
    use super::parse_server;
    use crate::config::CaldavConfig;

    /// A host outside ASCII is refused by `server` and `home` alike,
    /// rather than sent elsewhere, its Punycode form working: no Unicode
    /// IDNA back end is linked.
    #[test]
    fn a_non_ascii_host_is_refused() {
        for server in ["https://bücher.example/dav/", "bücher.example"] {
            let parsed = parse_server(server);
            assert!(parsed.is_err(), "{server}: {parsed:?}");
        }

        assert_eq!(
            parse_server("https://xn--bcher-kva.example/dav/")
                .unwrap()
                .as_str(),
            "https://xn--bcher-kva.example/dav/"
        );

        let home = |url: &str| {
            let document = format!("home = \"{url}\"\nauth = \"none\"");
            toml::from_str::<CaldavConfig>(&document).map(|config| config.home)
        };

        assert!(home("https://bücher.example/dav/").is_err());
        assert_eq!(
            home("https://xn--bcher-kva.example/dav/")
                .unwrap()
                .map(String::from),
            Some(String::from("https://xn--bcher-kva.example/dav/"))
        );
    }

    /// The refusal of a host outside ASCII names the host and its Punycode
    /// form as the way out, from `server` and `home` alike.
    #[test]
    fn a_non_ascii_host_names_its_punycode_form() {
        let hint = "Host `bücher.example` is not ASCII: write it in its Punycode form";

        for server in [
            "https://bücher.example/dav/",
            "bücher.example",
            "bücher.example:8443",
        ] {
            let err = parse_server(server).unwrap_err().to_string();
            assert!(err.contains(hint), "{server}: {err}");
        }

        let document = "home = \"https://alice@bücher.example:8443/dav/\"";
        let err = toml::from_str::<CaldavConfig>(document).unwrap_err();
        assert!(err.to_string().contains(hint), "{err}");
    }

    /// A URL that fails to parse is refused, rather than prefixed with
    /// `https://` as a bare authority would be and so sent to host
    /// `https`.
    #[test]
    fn a_malformed_url_is_refused_rather_than_prefixed() {
        for server in ["https://dav.example.org:99999/", "http://dav example.org/"] {
            let parsed = parse_server(server);
            assert!(parsed.is_err(), "{server}: {parsed:?}");
        }
    }

    /// Every spelling a user writes a context root in resolves, and a
    /// bare authority takes `https` rather than reading its first label
    /// as a scheme.
    #[test]
    fn a_bare_authority_defaults_to_https_and_a_full_url_is_kept() {
        let parsed = |server| parse_server(server).unwrap().to_string();

        assert_eq!(
            parsed("https://dav.example.org/dav/"),
            "https://dav.example.org/dav/"
        );
        assert_eq!(
            parsed("http://dav.example.org:8008/"),
            "http://dav.example.org:8008/"
        );
        assert_eq!(parsed("example.org"), "https://example.org/");
        assert_eq!(
            parsed("dav.example.org:8443"),
            "https://dav.example.org:8443/"
        );
    }
}
