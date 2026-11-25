//! # Configuration
//!
//! The TOML schema: a top-level block of rendering options, plus one
//! `[accounts.<name>]` block per account carrying a sub-block per
//! backend.
//!
//! The global block is folded under the selected account at load time,
//! so a value set once at the top applies everywhere it is not
//! overridden.
//!
//! Every block sets `deny_unknown_fields`, so a typo fails the load. The
//! block of a backend left out of the build loads unread into
//! [`AccountConfig`], so one file serves every build, and a command
//! reaching it fails with `backend-not-built`. A key or a table
//! calendula does not read fails as any unknown one does.

use std::collections::HashMap;
#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "jmap",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
))]
use std::path::PathBuf;

#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
use anyhow::{Result, bail};
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
use pimalaya_config::secret::{Secret, SecretResolver};
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
use pimalaya_config::toml::opt_shell_expanded_path;
#[cfg(any(
    feature = "vdir",
    feature = "pimdir",
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
use pimalaya_config::toml::shell_expanded_path;
#[cfg(any(feature = "caldav", feature = "jmap"))]
use pimalaya_config::toml::shell_expanded_string;
use pimalaya_config::{command::CommandConfig, toml::TomlConfig};
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
use pimalaya_stream::{
    proxy::{Proxy, ProxyAuth},
    tls::{Rustls, RustlsCrypto, Tls, TlsClient, TlsProvider, TlsServer},
};
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
use secrecy::SecretString;
use serde::Deserialize;
#[cfg(not(all(
    feature = "vdir",
    feature = "caldav",
    feature = "jmap",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
)))]
use serde::de::IgnoredAny;
#[cfg(feature = "caldav")]
use serde::{Deserializer, de::Error as _};
#[cfg(any(feature = "caldav", feature = "jmap"))]
use url::{ParseError, Url};

/// The whole configuration file: global options and every account,
/// refusing a key it does not know.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    /// `calendar` command options.
    #[serde(default)]
    pub calendar: CalendarConfig,
    /// `event` command options.
    #[serde(default)]
    pub event: EventConfig,
    /// One `[accounts.<name>]` block per account.
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn sample_url() -> &'static str {
        "https://github.com/pimalaya/calendula/blob/master/config.sample.toml"
    }

    fn accounts(&self) -> impl Iterator<Item = (&str, bool)> {
        let accounts = self.accounts.iter();
        accounts.map(|(name, account)| (name.as_str(), account.default))
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        let entry = self.accounts.remove_entry(name);

        #[cfg(any(
            feature = "caldav",
            feature = "jmap",
            feature = "gcal",
            feature = "msgraph"
        ))]
        let entry = entry.map(|(name, mut account)| {
            account.inherit_proxy();
            account.inherit_tls();
            (name, account)
        });

        entry
    }
}

/// One account: the backends it reads calendars from, plus whatever it
/// overrides of the global options, refusing a key it does not know.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountConfig {
    /// Whether a command given no `-a/--account` picks this account.
    #[serde(default)]
    pub default: bool,
    /// `calendar` command options.
    #[serde(default)]
    pub calendar: CalendarConfig,
    /// `event` command options.
    #[serde(default)]
    pub event: EventConfig,
    /// Proxy every network backend of this account goes through, unless
    /// its own block names one.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// TLS every network backend of this account uses, each field its own
    /// `tls` leaves unset taking the account's.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    pub tls: Option<TlsConfig>,
    /// vdir backend: one local directory per calendar.
    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
    /// pimdir backend: a local store a sync engine fills.
    #[cfg(feature = "pimdir")]
    pub pimdir: Option<PimdirConfig>,
    /// CalDAV backend: a remote calendar home-set.
    #[cfg(feature = "caldav")]
    pub caldav: Option<CaldavConfig>,
    /// JMAP backend: a JMAP for Calendars account.
    #[cfg(feature = "jmap")]
    pub jmap: Option<JmapConfig>,
    /// Google Calendar backend, over the vendor API.
    #[cfg(feature = "gcal")]
    pub gcal: Option<GcalConfig>,
    /// Microsoft Graph backend, over the vendor API.
    #[cfg(feature = "msgraph")]
    pub msgraph: Option<MsgraphConfig>,
    /// The vdir block, loaded unread by a build without vdir.
    #[cfg(not(feature = "vdir"))]
    #[serde(default)]
    pub vdir: Option<IgnoredAny>,
    /// The pimdir block, loaded unread by a build without pimdir.
    #[cfg(not(feature = "pimdir"))]
    #[serde(default)]
    pub pimdir: Option<IgnoredAny>,
    /// The CalDAV block, loaded unread by a build without CalDAV.
    #[cfg(not(feature = "caldav"))]
    #[serde(default)]
    pub caldav: Option<IgnoredAny>,
    /// The JMAP block, loaded unread by a build without JMAP.
    #[cfg(not(feature = "jmap"))]
    #[serde(default)]
    pub jmap: Option<IgnoredAny>,
    /// The Google Calendar block, loaded unread by a build without gcal.
    #[cfg(not(feature = "gcal"))]
    #[serde(default)]
    pub gcal: Option<IgnoredAny>,
    /// The Microsoft Graph block, loaded unread by a build without msgraph.
    #[cfg(not(feature = "msgraph"))]
    #[serde(default)]
    pub msgraph: Option<IgnoredAny>,
    /// The account proxy, loaded unread by a build with no network backend.
    #[cfg(not(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    )))]
    #[serde(default)]
    #[allow(dead_code)]
    pub proxy: Option<IgnoredAny>,
    /// The account TLS, loaded unread by a build with no network backend.
    #[cfg(not(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    )))]
    #[serde(default)]
    #[allow(dead_code)]
    pub tls: Option<IgnoredAny>,
}

impl AccountConfig {
    /// The backends the account carries a block for, as `--backend`
    /// spells them, in the order `auto` tries them, each with whether this
    /// build serves it.
    pub fn blocks(&self) -> Vec<(&'static str, bool)> {
        let blocks = [
            ("vdir", self.vdir.is_some(), cfg!(feature = "vdir")),
            ("pimdir", self.pimdir.is_some(), cfg!(feature = "pimdir")),
            ("caldav", self.caldav.is_some(), cfg!(feature = "caldav")),
            ("jmap", self.jmap.is_some(), cfg!(feature = "jmap")),
            ("gcal", self.gcal.is_some(), cfg!(feature = "gcal")),
            ("msgraph", self.msgraph.is_some(), cfg!(feature = "msgraph")),
        ];

        let carried = blocks.into_iter().filter(|(_, carried, _)| *carried);
        carried.map(|(name, _, built)| (name, built)).collect()
    }

    /// The backends the account carries a block for and this build serves,
    /// in the order `auto` tries them.
    pub fn backends(&self) -> Vec<&'static str> {
        let blocks = self.blocks().into_iter();
        blocks
            .filter_map(|(name, built)| built.then_some(name))
            .collect()
    }

    /// The backends the account carries a block for and this build leaves
    /// out, in the order `auto` tries them.
    pub fn not_built(&self) -> Vec<&'static str> {
        let blocks = self.blocks().into_iter();
        blocks
            .filter_map(|(name, built)| (!built).then_some(name))
            .collect()
    }

    /// Hands the account proxy to every network backend naming none of
    /// its own.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    fn inherit_proxy(&mut self) {
        let Some(proxy) = &self.proxy else {
            return;
        };

        let slots = [
            #[cfg(feature = "caldav")]
            self.caldav.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "jmap")]
            self.jmap.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "gcal")]
            self.gcal.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "msgraph")]
            self.msgraph.as_mut().map(|c| &mut c.proxy),
        ];

        for slot in slots.into_iter().flatten() {
            slot.get_or_insert_with(|| proxy.clone());
        }
    }

    /// Hands the account TLS to every network backend, each field its
    /// own `tls` leaves unset taking the account's.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    fn inherit_tls(&mut self) {
        let Some(tls) = &self.tls else {
            return;
        };

        let slots = [
            #[cfg(feature = "caldav")]
            self.caldav.as_mut().map(|c| &mut c.tls),
            #[cfg(feature = "jmap")]
            self.jmap.as_mut().map(|c| &mut c.tls),
            #[cfg(feature = "gcal")]
            self.gcal.as_mut().map(|c| &mut c.tls),
            #[cfg(feature = "msgraph")]
            self.msgraph.as_mut().map(|c| &mut c.tls),
        ];

        for slot in slots.into_iter().flatten() {
            slot.inherit(tls);
        }
    }
}

/// Calendar-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CalendarConfig {
    /// Calendar id used by `event` commands when their `-k/--calendar`
    /// flag is omitted.
    pub default: Option<String>,
}

/// Event-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EventConfig {
    /// Command an event is edited through, spawned on the path of a
    /// temporary iCalendar file it edits in place.
    ///
    /// A shell line or an argv list, the path appended as its last
    /// argument: `composer = "tcal edit"` runs `tcal edit <PATH>`.
    pub composer: Option<CommandConfig>,
    /// `event list` options.
    #[serde(default)]
    pub list: EventListConfig,
}

/// `event list` options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EventListConfig {
    /// Default `-s/--page-size` value for `event list`, 25 when unset.
    pub page_size: Option<u32>,
}

/// vdir backend configuration.
#[cfg(feature = "vdir")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    /// Filesystem path of the vdir home: the directory holding one
    /// subdirectory per calendar. Shell-expanded at load, so `~` and
    /// environment variables both work, as in `pimdir.root`.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub root: PathBuf,
}

/// pimdir backend configuration.
///
/// A pimdir store is an offline cache a sync engine fills, not a
/// server: calendula reads what the store holds and stages its writes
/// for the next sync to push.
#[cfg(feature = "pimdir")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PimdirConfig {
    /// The store directory, holding the SQLite index and the blob
    /// tree. Shell-expanded at load.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub root: PathBuf,
    /// The account whose collections this client reads, the name the
    /// sync engine groups them under (pimdir STORAGE §9.2).
    ///
    /// Usually left unset: a store synced by one account is read as
    /// that one. A store several accounts share is refused until this
    /// names one, rather than showing their calendars merged.
    #[serde(default)]
    pub account: Option<String>,
}

/// CalDAV backend configuration.
///
/// Locating the calendar home-set takes one of two routes: `server`
/// names the context root to walk from, `home` pins the home-set.
#[cfg(feature = "caldav")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CaldavConfig {
    /// DAV context root. Principal and calendar-home-set discovery
    /// start here.
    ///
    /// Accepts a full URL, a bare domain or `domain:port`, a bare
    /// authority defaulting to `https`. The host is ASCII, an
    /// internationalised one written in Punycode.
    pub server: Option<String>,
    /// Pre-resolved calendar home-set URL, skipping every discovery
    /// step. The host is ASCII, as for `server`.
    #[serde(default, deserialize_with = "ascii_url")]
    pub home: Option<Url>,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, unset offering
    /// [`default_caldav_alpn`].
    pub alpn: Option<Vec<String>>,
    /// Proxy the CalDAV connection goes through, overriding the account's
    /// `proxy`, and then the `all_proxy` and `https_proxy` environment
    /// variables.
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration, absent sending no credential.
    #[serde(default)]
    pub auth: CaldavAuthConfig,
}

/// ALPN identifiers CalDAV offers when `caldav.alpn` names none, pinning
/// the HTTP/1.1 io-http speaks.
#[cfg(feature = "caldav")]
pub fn default_caldav_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
}

/// Parses `input` into a URL, a host that is not ASCII refused with the
/// way out: its Punycode form, url linking no Unicode IDNA back end.
#[cfg(any(feature = "caldav", feature = "jmap"))]
pub fn parse_url(input: &str) -> Result<Url> {
    match Url::parse(input) {
        Err(ParseError::IdnaError) => bail!(
            "Host `{}` is not ASCII: write it in its Punycode form (xn--...)",
            host_of(input)
        ),
        result => Ok(result?),
    }
}

/// Deserializes an optional URL through [`parse_url`].
#[cfg(feature = "caldav")]
fn ascii_url<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Url>, D::Error> {
    let input = String::deserialize(deserializer)?;
    let url = parse_url(&input).map_err(D::Error::custom)?;
    Ok(Some(url))
}

/// The host `input` names, as written: what follows the scheme, short of
/// the path, the user info and the port.
#[cfg(any(feature = "caldav", feature = "jmap"))]
fn host_of(input: &str) -> &str {
    let rest = input.split_once("://").map_or(input, |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);

    match host.rsplit_once(':') {
        Some((name, port)) if port.bytes().all(|b| b.is_ascii_digit()) => name,
        _ => host,
    }
}

/// CalDAV authentication configuration.
#[cfg(feature = "caldav")]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum CaldavAuthConfig {
    /// No credentials, for a server that asks for none: no `auth` at all
    /// reads as this.
    #[default]
    None,
    /// HTTP Basic (RFC 7617), the usual username and password, often
    /// an app password.
    Basic {
        /// The login to present.
        #[serde(deserialize_with = "shell_expanded_string")]
        username: String,
        /// The password, read from the configuration or from the
        /// standard output of a command.
        password: Secret,
    },
    /// HTTP Bearer (RFC 6750), a provider-issued API token.
    Bearer {
        /// The token, read from the configuration or from the standard
        /// output of a command. An OAuth 2.0 token broker is a command
        /// like any other.
        token: Secret,
    },
}

/// JMAP configuration (RFC 8620 and draft-ietf-jmap-calendars), shaped as
/// cardamum's and himalaya's.
#[cfg(feature = "jmap")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct JmapConfig {
    /// The JMAP server address.
    ///
    /// A bare authority (`fastmail.com`, `mail.example.com:8080`) is
    /// discovered through `GET /.well-known/jmap`, a full URL reaches the
    /// session endpoint directly. Schemes: `http`, `https`, `jmap` and
    /// `jmaps`, the last two mapping to the first two.
    pub server: String,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, `["http/1.1"]`
    /// by default since JMAP rides on HTTP/1.1.
    ///
    /// An empty list skips ALPN negotiation. Only rustls reads it,
    /// `native-tls` ignores ALPN.
    #[serde(default = "io_jmap::client::JmapClientStd::default_alpn")]
    pub alpn: Vec<String>,
    /// Proxy the connection goes through, falling back to the account one
    /// and then to the `all_proxy`/`https_proxy` environment variables.
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration.
    pub auth: JmapAuthConfig,
}

/// JMAP authentication configuration.
#[cfg(feature = "jmap")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum JmapAuthConfig {
    /// A whole Authorization header value, sent verbatim.
    Header(Secret),
    /// A bearer token, OAuth 2.0 access token or provider API token.
    Bearer { token: Secret },
    /// HTTP Basic authentication.
    Basic {
        /// The username the server knows the account by.
        #[serde(deserialize_with = "shell_expanded_string")]
        username: String,
        /// The matching password, read from the configured secret.
        password: Secret,
    },
}

/// Parses a `server` config string into a URL.
///
/// A full URL is taken verbatim, a bare `host[:port]` takes
/// `default_scheme`, and a scheme outside `allowed` is rejected.
#[cfg(feature = "jmap")]
pub fn parse_server(server: &str, default_scheme: &str, allowed: &[&str]) -> Result<Url> {
    let url = if server.contains("://") {
        parse_url(server)?
    } else {
        parse_url(&format!("{default_scheme}://{server}"))?
    };

    let scheme = url.scheme();

    if !allowed.contains(&scheme) {
        bail!("Invalid server scheme `{scheme}`: expected one of {allowed:?}");
    }

    Ok(url)
}

/// Google Calendar backend configuration.
///
/// The API endpoint is fixed, so there is nothing to discover and no
/// server to name: a token and a TLS profile are the whole block.
#[cfg(feature = "gcal")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GcalConfig {
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake.
    ///
    /// Unset offers [`default_gcal_alpn`], an empty list skips ALPN
    /// negotiation and a non-empty one replaces the default. Only
    /// rustls reads it, native-tls ignoring ALPN altogether.
    pub alpn: Option<Vec<String>>,
    /// Proxy the Google Calendar connection goes through, overriding the
    /// account's `proxy`, and then the `all_proxy` and `https_proxy`
    /// environment variables.
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration.
    pub auth: GcalAuthConfig,
}

/// ALPN identifiers gcal offers when `gcal.alpn` names none.
///
/// io-http speaks HTTP/1.1 only, so the default pins it rather than
/// letting Google negotiate HTTP/2 on a connection nothing can read.
#[cfg(feature = "gcal")]
pub fn default_gcal_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
}

/// Google Calendar authentication configuration.
///
/// A struct rather than an enumeration of kinds: the Calendar API takes
/// an OAuth 2.0 bearer token and nothing else. It still nests under
/// `auth`, so every Pimalaya backend spells its credentials alike.
#[cfg(feature = "gcal")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GcalAuthConfig {
    /// The token, read from the configuration or from the standard
    /// output of a command. Google expires an access token within the
    /// hour, so a token broker is the usual answer, and a broker is a
    /// command like any other.
    pub token: Secret,
}

/// Microsoft Graph backend configuration.
///
/// The API endpoint is fixed: the calendar owner, a token and a TLS
/// profile are the whole block.
#[cfg(feature = "msgraph")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MsgraphConfig {
    /// Graph user id, `me` by default: the authenticated user.
    #[serde(default = "default_msgraph_user_id")]
    pub user_id: String,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// ALPN identifiers offered during the TLS handshake, unset offering
    /// [`default_msgraph_alpn`].
    pub alpn: Option<Vec<String>>,
    /// Proxy the Microsoft Graph connection goes through, overriding the
    /// account's `proxy`, and then the `all_proxy` and `https_proxy`
    /// environment variables.
    pub proxy: Option<ProxyConfig>,
    /// Authentication configuration.
    pub auth: MsgraphAuthConfig,
}

/// The Graph user a block names when it names none: the authenticated one.
#[cfg(feature = "msgraph")]
fn default_msgraph_user_id() -> String {
    String::from("me")
}

/// ALPN identifiers msgraph offers when `msgraph.alpn` names none, pinning
/// the HTTP/1.1 io-http speaks.
#[cfg(feature = "msgraph")]
pub fn default_msgraph_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
}

/// Microsoft Graph authentication configuration: an OAuth 2.0 bearer token
/// and nothing else, calendula running no OAuth flow itself.
#[cfg(feature = "msgraph")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MsgraphAuthConfig {
    /// The token, read from the configuration or from the standard output
    /// of a command, a token broker such as ortie.
    pub token: Secret,
}

/// Proxy configuration.
///
/// `url` is a `socks5://`, `socks5h://` or `http://` proxy URL. Its user
/// info authenticates too, but `username` and `password` keep the secret
/// out of the URL.
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProxyConfig {
    /// The proxy URL.
    pub url: String,
    /// The proxy username, required by `password`.
    pub username: Option<String>,
    /// The proxy password, read from the configuration or from the
    /// standard output of a command.
    pub password: Option<Secret>,
}

#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
impl ProxyConfig {
    /// Resolves an optional configuration, an absent one reading the
    /// environment at connect time.
    pub fn resolve(config: Option<Self>, resolver: &mut SecretResolver) -> Result<Proxy> {
        match config {
            Some(config) => config.try_into_proxy(resolver),
            None => Ok(Proxy::System),
        }
    }

    /// Resolves the configuration into a runtime [`Proxy`], the password
    /// going through `resolver`.
    pub fn try_into_proxy(self, resolver: &mut SecretResolver) -> Result<Proxy> {
        let mut proxy = Proxy::from_url(&self.url)?;

        let auth = match (self.username, self.password) {
            (None, None) => return Ok(proxy),
            (None, Some(_)) => bail!("Proxy password requires a username"),
            (Some(user), pass) => ProxyAuth {
                user,
                pass: match pass {
                    Some(pass) => resolver.resolve(pass)?,
                    None => SecretString::default(),
                },
            },
        };

        match &mut proxy {
            Proxy::Socks5 { auth: slot, .. } | Proxy::Http { auth: slot, .. } => {
                *slot = Some(auth);
            }
            Proxy::None | Proxy::System => {}
        }

        Ok(proxy)
    }
}

/// SSL/TLS configuration.
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsConfig {
    /// TLS implementation. Unset takes the first compiled in, rustls
    /// before native-tls.
    pub provider: Option<TlsProviderConfig>,
    /// rustls options, ignored when native-tls is the provider.
    #[serde(default)]
    pub rustls: RustlsConfig,
    /// PEM certificate to trust, shell-expanded at load.
    ///
    /// Pinned to the server's leaf under rustls and taken as a root
    /// certificate under native-tls.
    #[serde(default, deserialize_with = "opt_shell_expanded_path")]
    pub cert: Option<PathBuf>,
    /// A client certificate, presented when the server asks for one.
    pub client: Option<TlsClientConfig>,
}

#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
impl TlsConfig {
    /// Takes from `account` each field this block leaves unset: the
    /// provider, the rustls crypto, the trusted certificate and the
    /// client certificate.
    pub fn inherit(&mut self, account: &TlsConfig) {
        if self.provider.is_none() {
            self.provider.clone_from(&account.provider);
        }
        if self.rustls.crypto.is_none() {
            self.rustls.crypto.clone_from(&account.rustls.crypto);
        }
        if self.cert.is_none() {
            self.cert.clone_from(&account.cert);
        }
        if self.client.is_none() {
            self.client.clone_from(&account.client);
        }
    }
}

/// Client certificate configuration, both in PEM format.
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsClientConfig {
    /// The certificate, optionally followed by its intermediates.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub cert: PathBuf,
    /// The private key, PKCS#8 under native-tls.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub key: PathBuf,
}

/// Which TLS implementation carries the connection.
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TlsProviderConfig {
    /// The pure-Rust rustls stack.
    Rustls,
    /// The platform's own TLS stack.
    NativeTls,
}

/// rustls-specific options.
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RustlsConfig {
    /// Crypto provider. Unset prefers ring, else aws-lc-rs.
    pub crypto: Option<RustlsCryptoConfig>,
}

/// Which cryptographic provider backs rustls.
#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RustlsCryptoConfig {
    /// The aws-lc-rs provider.
    Aws,
    /// The ring provider.
    Ring,
}

#[cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]
impl TlsConfig {
    /// Builds the runtime [`Tls`] handle the connect helpers expect,
    /// folding in the protocol-level `alpn` list.
    ///
    /// The TOML schema never exposes `tls.rustls.alpn` directly, the
    /// per-backend `alpn` field standing for it. An empty list skips
    /// ALPN, so every caller says what it negotiates.
    pub fn into_tls(self, alpn: Vec<String>) -> Tls {
        Tls {
            provider: self.provider.map(|provider| match provider {
                TlsProviderConfig::Rustls => TlsProvider::Rustls,
                TlsProviderConfig::NativeTls => TlsProvider::NativeTls,
            }),
            rustls: Rustls {
                crypto: self.rustls.crypto.map(|crypto| match crypto {
                    RustlsCryptoConfig::Aws => RustlsCrypto::Aws,
                    RustlsCryptoConfig::Ring => RustlsCrypto::Ring,
                }),
                alpn,
            },
            server: self.cert.map(|cert| TlsServer { cert }),
            client: self.client.map(|client| TlsClient {
                cert: client.cert,
                key: client.key,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs, io::Write, path::PathBuf};

    use tempfile::NamedTempFile;

    use crate::config::*;

    /// The home directory the tilde in a test path stands for.
    #[cfg(any(
        feature = "vdir",
        feature = "caldav",
        feature = "jmap",
        feature = "pimdir",
        feature = "gcal",
        feature = "msgraph"
    ))]
    fn home() -> PathBuf {
        let home = std::env::var("HOME").expect("HOME must be set to expand a tilde");
        PathBuf::from(home)
    }

    /// Expansion happens at deserialize, so no call site can read a
    /// literal `./~/…` directory under the current one; an environment
    /// variable expands as the tilde does.
    #[cfg(feature = "vdir")]
    #[test]
    fn a_vdir_root_expands_its_tilde_at_deserialize() {
        for root in ["~/calendars", "$HOME/calendars"] {
            let document = format!("root = {root:?}");
            let config: VdirConfig = toml::from_str(&document).unwrap();
            assert_eq!(config.root, home().join("calendars"), "{root}");
        }
    }

    /// A CalDAV block with no `auth` loads, sending no credential, as
    /// `auth = "none"` does.
    #[cfg(feature = "caldav")]
    #[test]
    fn a_caldav_block_without_auth_loads() {
        let config: CaldavConfig = toml::from_str("server = \"dav.example.org\"").unwrap();
        assert!(matches!(config.auth, CaldavAuthConfig::None));

        let document = "[accounts.a]\ncaldav.home = \"http://localhost:5232/a/\"";
        let config: Config = toml::from_str(document).unwrap();
        let caldav = config.accounts["a"].caldav.as_ref().unwrap();
        assert!(matches!(caldav.auth, CaldavAuthConfig::None));
    }

    #[cfg(feature = "pimdir")]
    #[test]
    fn a_pimdir_root_expands_its_tilde_at_deserialize() {
        let config: PimdirConfig = toml::from_str(r#"root = "~/store""#).unwrap();
        assert_eq!(config.root, home().join("store"));
    }

    /// The optional path takes the same treatment, and an absent key
    /// still reaches `None` rather than the deserializer.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    #[test]
    fn a_tls_certificate_expands_its_tilde_and_stays_optional() {
        let config: TlsConfig = toml::from_str(r#"cert = "~/ca.pem""#).unwrap();
        assert_eq!(config.cert, Some(home().join("ca.pem")));

        let config: TlsConfig = toml::from_str("").unwrap();
        assert_eq!(config.cert, None);
    }

    /// A client certificate comes with its key, both paths expanded, and
    /// a certificate without its key is refused.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    #[test]
    fn a_client_certificate_needs_its_key() {
        let config: TlsConfig =
            toml::from_str(r#"client = { cert = "~/me.pem", key = "~/me.key.pem" }"#).unwrap();
        let client = config.client.unwrap();
        assert_eq!(client.cert, home().join("me.pem"));
        assert_eq!(client.key, home().join("me.key.pem"));

        assert!(toml::from_str::<TlsConfig>(r#"client.cert = "~/me.pem""#).is_err());
    }

    /// A backend's `tls` takes from the account's each field it leaves
    /// unset, the client certificate as one.
    #[cfg(all(feature = "caldav", feature = "gcal"))]
    #[test]
    fn an_account_tls_fills_each_field_a_backend_leaves_unset() {
        use pimalaya_config::toml::TomlConfig;

        let document = "[accounts.a]\n\
                        tls.provider = \"native-tls\"\n\
                        tls.cert = \"/ca.pem\"\n\
                        tls.client = { cert = \"/me.pem\", key = \"/me.key\" }\n\
                        caldav.server = \"example.org\"\n\
                        caldav.auth = \"none\"\n\
                        caldav.tls.cert = \"/dav.pem\"\n\
                        gcal.auth.token.raw = \"token\"\n";
        let mut config: Config = toml::from_str(document).unwrap();
        let (_, account) = config.take_account(Some("a")).unwrap();

        let caldav = account.caldav.unwrap().tls;
        let gcal = account.gcal.unwrap().tls;

        assert!(matches!(
            caldav.provider,
            Some(TlsProviderConfig::NativeTls)
        ));
        assert_eq!(caldav.cert, Some(PathBuf::from("/dav.pem")));
        assert_eq!(caldav.client.unwrap().key, PathBuf::from("/me.key"));
        assert!(matches!(gcal.provider, Some(TlsProviderConfig::NativeTls)));
        assert_eq!(gcal.cert, Some(PathBuf::from("/ca.pem")));
    }

    /// Every spelling a CalDAV endpoint is written in round-trips: a
    /// full URL stays one, and a bare authority is kept verbatim for
    /// the client to resolve.
    #[cfg(feature = "caldav")]
    #[test]
    fn a_caldav_server_takes_a_url_or_a_bare_authority() {
        for spelling in [
            "https://dav.example.org/dav/",
            "http://dav.example.org:8008/",
            "example.org",
            "dav.example.org:8443",
        ] {
            let document = format!("server = \"{spelling}\"\nauth = \"none\"");
            let config: CaldavConfig = toml::from_str(&document).unwrap();
            assert_eq!(config.server.as_deref(), Some(spelling));
        }
    }

    /// A table or a key an earlier calendula read is refused as any
    /// unknown one is, with no sentence of its own.
    #[cfg(all(feature = "vdir", feature = "caldav"))]
    #[test]
    fn a_key_an_earlier_calendula_read_is_an_unknown_one() {
        for (document, field) in [
            ("[todo.list]\npage-size = 50\n[accounts.a]\n", "todo"),
            ("[accounts.a.journal]\nlist.page-size = 50\n", "journal"),
            ("[table]\npreset = \"ASCII_FULL\"\n[accounts]\n", "table"),
            (
                "[account.list.table]\nname-color = \"red\"\n[accounts]\n",
                "account",
            ),
            (
                "[accounts.a]\ncalendar.list.table.name-color = \"red\"\n",
                "list",
            ),
            (
                "[accounts.a]\nevent.list.table.id-color = \"red\"\n",
                "table",
            ),
            ("[accounts.a]\nvdir.home-dir = \"~/c\"\n", "home-dir"),
            (
                "[accounts.a]\ncaldav.discover = true\ncaldav.auth = \"none\"\n",
                "discover",
            ),
        ] {
            let err = toml::from_str::<Config>(document).unwrap_err().to_string();
            assert!(err.contains(&format!("unknown field `{field}`")), "{err}");
            assert!(!err.contains("MIGRATION"), "{err}");
        }
    }

    #[test]
    fn an_unknown_root_key_is_refused() {
        let document = "colour = \"red\"\n[accounts.a]\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();
        assert!(err.contains("unknown field `colour`"), "{err}");
    }

    #[test]
    fn an_unknown_account_key_is_refused() {
        let document = "[accounts.a]\ndefualt = true\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();
        assert!(err.contains("unknown field `defualt`"), "{err}");
    }

    /// A backend table, the tables nested in it and the option tables
    /// refuse a key they do not know.
    #[cfg(all(feature = "vdir", feature = "caldav"))]
    #[test]
    fn an_unknown_backend_or_nested_key_is_refused() {
        for (document, refusal) in [
            ("[accounts.a]\nvdir.rot = \"~/c\"\n", "unknown field `rot`"),
            (
                "[accounts.a]\ncaldav.server = \"example.org\"\n\
                 caldav.auth = \"none\"\ncaldav.tls.certificate = \"ca.pem\"\n",
                "unknown field `certificate`",
            ),
            (
                "[accounts.a]\ncaldav.server = \"example.org\"\n\
                 caldav.auth.basic.usernme = \"alice\"\n",
                "unexpected keys in table: usernme",
            ),
            (
                "[accounts.a]\nevent.list.page-sise = 10\n",
                "unknown field `page-sise`",
            ),
        ] {
            let err = toml::from_str::<Config>(document).unwrap_err().to_string();
            assert!(err.contains(refusal), "{err}");
        }
    }

    /// One file serves every build: a block of each backend loads in a
    /// build leaving any of them out, unread.
    #[test]
    fn a_backend_left_out_of_the_build_still_loads() {
        let document = "[accounts.a]\n\
                        vdir.root = \"~/calendars\"\n\
                        pimdir.root = \"~/store\"\n\
                        caldav.server = \"example.org\"\n\
                        caldav.auth = \"none\"\n\
                        jmap.server = \"jmap.example.org\"\n\
                        jmap.auth.bearer.token.raw = \"token\"\n\
                        gcal.auth.token.raw = \"token\"\n\
                        msgraph.auth.token.raw = \"token\"\n";
        toml::from_str::<Config>(document).unwrap();
    }

    #[cfg(feature = "caldav")]
    #[test]
    fn a_caldav_alpn_is_read_and_defaults_to_http_1_1() {
        let document = "home = \"https://dav.example.org/\"\nauth = \"none\"\n";
        let config: CaldavConfig = toml::from_str(document).unwrap();
        assert_eq!(config.alpn, None);
        assert_eq!(default_caldav_alpn(), ["http/1.1"]);

        let document = format!("{document}alpn = []\n");
        let config: CaldavConfig = toml::from_str(&document).unwrap();
        assert_eq!(config.alpn, Some(Vec::new()));
    }

    /// The account `proxy` reaches every network backend naming none of its
    /// own, and a backend's own wins.
    #[cfg(all(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    #[test]
    fn an_account_proxy_fills_every_backend_naming_none() {
        use pimalaya_config::toml::TomlConfig;

        let document = "[accounts.a]\n\
                        proxy.url = \"socks5h://localhost:1080\"\n\
                        caldav.server = \"example.org\"\n\
                        caldav.auth = \"none\"\n\
                        jmap.server = \"jmap.example.org\"\n\
                        jmap.auth.bearer.token.raw = \"token\"\n\
                        gcal.auth.token.raw = \"token\"\n\
                        msgraph.auth.token.raw = \"token\"\n\
                        msgraph.proxy.url = \"http://proxy.example.org:3128\"\n";
        let mut config: Config = toml::from_str(document).unwrap();
        let (_, account) = config.take_account(Some("a")).unwrap();

        let caldav = account.caldav.unwrap().proxy.unwrap();
        let jmap = account.jmap.unwrap().proxy.unwrap();
        let gcal = account.gcal.unwrap().proxy.unwrap();
        let msgraph = account.msgraph.unwrap().proxy.unwrap();

        assert_eq!(caldav.url, "socks5h://localhost:1080");
        assert_eq!(jmap.url, "socks5h://localhost:1080");
        assert_eq!(gcal.url, "socks5h://localhost:1080");
        assert_eq!(msgraph.url, "http://proxy.example.org:3128");
    }

    /// A password needs a username, and both reach the runtime proxy, the
    /// password through the resolver.
    #[cfg(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    ))]
    #[test]
    fn a_proxy_password_needs_a_username() {
        use pimalaya_config::secret::SecretResolver;
        use pimalaya_stream::proxy::Proxy;

        let proxy = |document: &str| {
            let config: ProxyConfig = toml::from_str(document).unwrap();
            config.try_into_proxy(&mut SecretResolver::new())
        };

        let url = "url = \"http://localhost:3128\"\n";
        let err = proxy(&format!("{url}password.raw = \"secret\"\n")).unwrap_err();
        assert!(err.to_string().contains("requires a username"), "{err}");

        let document = format!("{url}username = \"alice\"\npassword.raw = \"secret\"\n");
        match proxy(&document).unwrap() {
            Proxy::Http {
                auth: Some(auth), ..
            } => assert_eq!(auth.user, "alice"),
            other => panic!("{other:?}"),
        }
    }

    /// Checks config.sample.toml parses as written and once per rank fully
    /// uncommented, `example` its only account and the default each time,
    /// every variant declaring each backend's block, a build leaving one
    /// out loading it unread.
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let sample = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = sample.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        load_sample(&sample, "as written");

        let paragraphs = sample_paragraphs(&sample);

        for rank in 1..=sample_ranks(&paragraphs) {
            let text = sample_variant(&paragraphs, rank);
            let name = format!("variant {rank}");
            let account = load_sample(&text, &name);

            let blocks = account.blocks().into_iter().map(|(block, _)| block);
            let blocks: Vec<_> = blocks.collect();
            let all = ["vdir", "pimdir", "caldav", "jmap", "gcal", "msgraph"];
            assert_eq!(blocks, all, "config.sample.toml, {name}");
        }
    }

    /// One line of config.sample.toml.
    struct SampleLine<'a> {
        /// The line as written.
        line: &'a str,
        /// The code before the comment, one `#` off when commented.
        code: &'a str,
        /// Whether the code is commented.
        commented: bool,
        /// The field the code sets, a secret's `raw` and `command` being
        /// one.
        field: Option<&'a str>,
        /// The other values the comment lists in brackets.
        values: Vec<&'a str>,
    }

    impl<'a> SampleLine<'a> {
        fn parse(line: &'a str) -> Self {
            let (code, comment) = line.split_once('\t').unwrap_or((line, ""));
            let commented = code.starts_with('#');
            let code = code.strip_prefix('#').unwrap_or(code);

            let field = code
                .split_once(" = ")
                .map(|(key, _)| key)
                .filter(|key| {
                    key.chars()
                        .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
                })
                .map(|key| {
                    let secret = key.strip_suffix(".raw").or(key.strip_suffix(".command"));
                    secret.unwrap_or(key)
                });

            let values = comment
                .trim_end()
                .strip_suffix(']')
                .and_then(|comment| comment.rsplit_once('['))
                .map(|(_, values)| values.split(", ").collect())
                .unwrap_or_default();

            Self {
                line,
                code,
                commented,
                field,
                values,
            }
        }

        /// The first two segments of the field, the table a replacing
        /// paragraph shares with the one above it (`caldav.auth`).
        fn table(&self) -> Option<&'a str> {
            let field = self.field?;
            let end = field
                .match_indices('.')
                .nth(1)
                .map_or(field.len(), |(i, _)| i);
            Some(&field[..end])
        }

        /// The values the line offers: its code as written, then each
        /// bracketed value in its place, typed like the written one.
        fn options(&self) -> Vec<String> {
            let (key, value) = self.code.split_once(" = ").unwrap();
            let mut options = vec![self.code.to_owned()];

            for other in &self.values {
                if value.starts_with('"') {
                    options.push(format!("{key} = \"{other}\""));
                } else {
                    options.push(format!("{key} = {other}"));
                }
            }

            options
        }
    }

    /// Loads `text` through the loader `-c` uses, returning the default
    /// account, which must be `example`.
    fn load_sample(text: &str, name: &str) -> AccountConfig {
        let context = format!("config.sample.toml, {name}");

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(text.as_bytes()).unwrap();

        let mut config = Config::load(&[file.path().to_path_buf()])
            .unwrap_or_else(|err| panic!("{context}: {err:?}\n{text}"));
        let (account, account_config) = config
            .take_account(None)
            .unwrap_or_else(|err| panic!("{context}: {err:?}"));

        assert_eq!(account, "example", "{context}");
        account_config
    }

    fn sample_paragraphs(sample: &str) -> Vec<Vec<SampleLine<'_>>> {
        let paragraphs = sample.split("\n\n");
        let paragraphs = paragraphs.map(|p| p.lines().map(SampleLine::parse).collect());
        paragraphs.collect()
    }

    /// Whether `below`, all commented, replaces `above`, setting keys under
    /// a table one of its lines sets.
    fn sample_replaces(above: &[SampleLine], below: &[SampleLine]) -> bool {
        let mut code = below.iter().filter(|line| line.field.is_some());
        let Some(first) = code.next() else {
            return false;
        };

        if !first.commented || code.any(|line| !line.commented) {
            return false;
        }

        above
            .iter()
            .filter_map(SampleLine::table)
            .any(|t| Some(t) == first.table())
    }

    /// The values each field takes in turn, the lines setting it in their
    /// order, each followed by its bracketed values.
    fn sample_options<'a>(lines: &[&'a SampleLine<'a>]) -> HashMap<&'a str, Vec<String>> {
        let mut options: HashMap<&str, Vec<String>> = HashMap::new();

        for line in lines {
            if let Some(field) = line.field {
                options.entry(field).or_default().extend(line.options());
            }
        }

        options
    }

    /// How many variants give every value its turn: the most values of
    /// one field, two when a paragraph replaces another.
    fn sample_ranks(paragraphs: &[Vec<SampleLine>]) -> usize {
        let lines: Vec<&SampleLine> = paragraphs.iter().flatten().collect();
        let options = sample_options(&lines);
        let values = options.values().map(Vec::len).max().unwrap_or(0);

        let replaces = paragraphs.windows(2).any(|w| sample_replaces(&w[0], &w[1]));

        if replaces { values.max(2) } else { values }
    }

    /// The sample at `rank`, every line uncommented, each field set to its
    /// value of that rank (the last when it has fewer).
    ///
    /// A replacing paragraph stays out at rank 1 and replaces the one
    /// above it from rank 2.
    fn sample_variant(paragraphs: &[Vec<SampleLine>], rank: usize) -> String {
        let mut dropped = vec![false; paragraphs.len()];

        for i in 1..paragraphs.len() {
            if sample_replaces(&paragraphs[i - 1], &paragraphs[i]) {
                let i = if rank == 1 { i } else { i - 1 };
                dropped[i] = true;
            }
        }

        let kept = paragraphs
            .iter()
            .zip(&dropped)
            .filter(|(_, dropped)| !**dropped);
        let lines: Vec<&SampleLine> = kept.flat_map(|(lines, _)| lines).collect();

        let mut options = sample_options(&lines);
        let mut text = Vec::with_capacity(lines.len());

        for line in lines {
            let Some(field) = line.field else {
                text.push(line.line.to_owned());
                continue;
            };

            if let Some(values) = options.remove(field) {
                text.push(values[rank.min(values.len()) - 1].clone());
            }
        }

        text.join("\n")
    }
}

/// Every network backend reaches its server through the proxy it names,
/// checked against a local HTTP proxy that reads the `CONNECT` and refuses
/// it, so nothing goes past it.
#[cfg(all(
    test,
    any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph"
    )
))]
mod proxy_tests {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::mpsc::{self, Receiver},
        thread,
    };

    /// A proxy refusing the one tunnel it is asked for, its URL and the
    /// request line it read.
    fn refusing_proxy() -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (sender, lines) = mpsc::channel();

        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut stream = BufReader::new(stream);
            let mut line = String::new();
            stream.read_line(&mut line).unwrap();

            let mut header = String::new();
            while stream.read_line(&mut header).unwrap() > 2 {
                header.clear();
            }

            let refusal = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
            stream.get_mut().write_all(refusal).unwrap();
            sender.send(line).unwrap();
        });

        (url, lines)
    }

    /// The `[accounts.a]` block `lines` makes, its proxy taken.
    fn account(lines: &str) -> crate::config::AccountConfig {
        use pimalaya_config::toml::TomlConfig;

        let document = format!("[accounts.a]\n{lines}");
        let mut config: crate::config::Config = toml::from_str(&document).unwrap();
        config.take_account(Some("a")).unwrap().1
    }

    #[cfg(feature = "caldav")]
    #[test]
    fn caldav_connects_through_the_account_proxy() {
        use pimalaya_config::secret::SecretResolver;

        let (url, lines) = refusing_proxy();
        let account = account(&format!(
            "proxy.url = \"{url}\"\ncaldav.home = \"http://calendars.invalid/alice/\"\n\
             caldav.auth = \"none\"\n"
        ));

        let config = account.caldav.unwrap();
        let result = crate::caldav::client::connect(&config, &mut SecretResolver::new());

        assert!(result.is_err());
        let line = lines.recv().unwrap();
        assert!(line.starts_with("CONNECT calendars.invalid:80 "), "{line}");
    }

    #[cfg(feature = "jmap")]
    #[test]
    fn jmap_connects_through_the_account_proxy() {
        use pimalaya_config::secret::SecretResolver;

        let (url, lines) = refusing_proxy();
        let account = account(&format!(
            "proxy.url = \"{url}\"\njmap.server = \"jmap.invalid\"\n\
             jmap.auth.bearer.token.raw = \"made-up\"\n"
        ));

        let config = account.jmap.unwrap();
        let result = crate::jmap::client::connect(config, &mut SecretResolver::new());

        assert!(result.is_err());
        let line = lines.recv().unwrap();
        assert!(line.starts_with("CONNECT jmap.invalid:443 "), "{line}");
    }

    #[cfg(feature = "gcal")]
    #[test]
    fn gcal_connects_through_its_own_proxy() {
        use pimalaya_config::secret::SecretResolver;

        let (url, lines) = refusing_proxy();
        let account = account(&format!(
            "proxy.url = \"http://127.0.0.1:9\"\ngcal.proxy.url = \"{url}\"\n\
             gcal.auth.token.raw = \"made-up\"\n"
        ));

        let config = account.gcal.unwrap();
        let result = crate::gcal::client::connect(&config, &mut SecretResolver::new());

        assert!(result.is_err());
        let line = lines.recv().unwrap();
        assert!(
            line.starts_with("CONNECT www.googleapis.com:443 "),
            "{line}"
        );
    }

    #[cfg(feature = "msgraph")]
    #[test]
    fn msgraph_connects_through_the_account_proxy() {
        use pimalaya_config::secret::SecretResolver;

        let (url, lines) = refusing_proxy();
        let account = account(&format!(
            "proxy.url = \"{url}\"\nmsgraph.auth.token.raw = \"made-up\"\n"
        ));

        let config = account.msgraph.unwrap();
        let backend =
            crate::msgraph::backend::MsgraphBackend::new(config, &mut SecretResolver::new());

        assert!(backend.is_err());
        let line = lines.recv().unwrap();
        assert!(
            line.starts_with("CONNECT graph.microsoft.com:443 "),
            "{line}"
        );
    }
}
