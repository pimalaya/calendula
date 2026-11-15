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
//! reaching it fails with `backend-not-built`. The tables calendula
//! no longer reads are refused by name: `todo` and `journal`, their
//! domains having moved to unfog and journal, and the rendering tables
//! (`table`, `account`, `calendar.list`, `event.list.table`,
//! `item.list.table`), the output being plain text. So are the
//! `caldav.discover` key, `caldav.server` naming the root instead, and
//! the `vdir.home-dir` key, `vdir.root` naming the home. Each refusal is
//! pimalaya-config's sentence (`removed_key!`), naming MIGRATION.md.

use std::collections::HashMap;
#[cfg(any(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
))]
use std::path::PathBuf;

#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use anyhow::{Result, bail};
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_config::secret::{Secret, SecretResolver};
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_config::toml::opt_shell_expanded_path;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
use pimalaya_config::toml::shell_expanded_path;
#[cfg(feature = "caldav")]
use pimalaya_config::toml::shell_expanded_string;
use pimalaya_config::{command::CommandConfig, removed_key, toml::TomlConfig};
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_stream::{
    proxy::{Proxy, ProxyAuth},
    tls::{Rustls, RustlsCrypto, Tls, TlsProvider, TlsServer},
};
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use secrecy::SecretString;
use serde::Deserialize;
#[cfg(not(all(
    feature = "vdir",
    feature = "caldav",
    feature = "pimdir",
    feature = "gcal",
    feature = "msgraph"
)))]
use serde::de::IgnoredAny;
#[cfg(feature = "caldav")]
use url::Url;

/// The whole configuration file: global options and every account,
/// refusing a key it does not know.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    /// Refused: the output is plain text, drawn in no table.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<TableRemovedConfig>,
    /// `calendar` command options.
    #[serde(default)]
    pub calendar: CalendarConfig,
    /// `event` command options.
    #[serde(default)]
    pub event: EventConfig,
    /// `item` command options.
    #[serde(default)]
    pub item: ItemConfig,
    /// Refused: the to-dos moved to unfog.
    #[serde(default)]
    #[allow(dead_code)]
    pub todo: Option<TodoMovedConfig>,
    /// Refused: the journal entries moved to journal.
    #[serde(default)]
    #[allow(dead_code)]
    pub journal: Option<JournalMovedConfig>,
    /// Refused: `account list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub account: Option<AccountRemovedConfig>,
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

        #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
        let entry = entry.map(|(name, mut account)| {
            account.inherit_proxy();
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
    /// Refused: the output is plain text, drawn in no table.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<TableRemovedConfig>,
    /// `calendar` command options.
    #[serde(default)]
    pub calendar: CalendarConfig,
    /// `event` command options.
    #[serde(default)]
    pub event: EventConfig,
    /// `item` command options.
    #[serde(default)]
    pub item: ItemConfig,
    /// Refused: the to-dos moved to unfog.
    #[serde(default)]
    #[allow(dead_code)]
    pub todo: Option<TodoMovedConfig>,
    /// Refused: the journal entries moved to journal.
    #[serde(default)]
    #[allow(dead_code)]
    pub journal: Option<JournalMovedConfig>,
    /// Proxy every network backend of this account goes through, unless
    /// its own block names one.
    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    /// vdir backend: one local directory per calendar.
    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
    /// pimdir backend: a local store a sync engine fills.
    #[cfg(feature = "pimdir")]
    pub pimdir: Option<PimdirConfig>,
    /// CalDAV backend: a remote calendar home-set.
    #[cfg(feature = "caldav")]
    pub caldav: Option<CaldavConfig>,
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
    /// The Google Calendar block, loaded unread by a build without gcal.
    #[cfg(not(feature = "gcal"))]
    #[serde(default)]
    pub gcal: Option<IgnoredAny>,
    /// The Microsoft Graph block, loaded unread by a build without msgraph.
    #[cfg(not(feature = "msgraph"))]
    #[serde(default)]
    pub msgraph: Option<IgnoredAny>,
    /// The account proxy, loaded unread by a build with no network backend.
    #[cfg(not(any(feature = "caldav", feature = "gcal", feature = "msgraph")))]
    #[serde(default)]
    #[allow(dead_code)]
    pub proxy: Option<IgnoredAny>,
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
    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    fn inherit_proxy(&mut self) {
        let Some(proxy) = &self.proxy else {
            return;
        };

        let slots = [
            #[cfg(feature = "caldav")]
            self.caldav.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "gcal")]
            self.gcal.as_mut().map(|c| &mut c.proxy),
            #[cfg(feature = "msgraph")]
            self.msgraph.as_mut().map(|c| &mut c.proxy),
        ];

        for slot in slots.into_iter().flatten() {
            slot.get_or_insert_with(|| proxy.clone());
        }
    }
}

/// Calendar-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CalendarConfig {
    /// Calendar id used by `event` and `item` commands when their
    /// `-k/--calendar` flag is omitted.
    pub default: Option<String>,
    /// Refused: `calendar list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub list: Option<CalendarListRemovedConfig>,
}

/// Event-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EventConfig {
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
    /// Refused: `event list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<EventListTableRemovedConfig>,
}

removed_key! {
    /// The `todo` table, refused by name, the to-dos having moved to unfog.
    pub TodoMovedConfig: Table {
        name: "todo",
        why: "the to-dos having moved to unfog, the Pimalaya CLI for tasks",
        instead: "delete it",
    }

    /// The `journal` table, refused by name, the journal entries having
    /// moved to journal.
    pub JournalMovedConfig: Table {
        name: "journal",
        why: "the journal entries having moved to journal, the Pimalaya CLI \
              for journal entries",
        instead: "delete it",
    }

    /// The `table` table, refused by name, the output being plain text.
    pub TableRemovedConfig: Table {
        name: "table",
        why: "calendula printing plain text with no border or colour",
        instead: "delete it",
    }

    /// The top-level `account` table, refused by name, `account list`
    /// printing plain text.
    pub AccountRemovedConfig: Table {
        name: "account",
        why: "`account list` printing plain text with no colour",
        instead: "delete it",
    }

    /// The `calendar.list` table, refused by name, `calendar list`
    /// printing plain text.
    pub CalendarListRemovedConfig: Table {
        name: "calendar.list",
        why: "`calendar list` printing plain text with no colour",
        instead: "delete it",
    }

    /// The `event.list.table` table, refused by name, `event list`
    /// printing plain text.
    pub EventListTableRemovedConfig: Table {
        name: "event.list.table",
        why: "`event list` printing plain text with no colour",
        instead: "delete it",
    }

    /// The `item.list.table` table, refused by name, `item list` printing
    /// plain text.
    pub ItemListTableRemovedConfig: Table {
        name: "item.list.table",
        why: "`item list` printing plain text with no colour",
        instead: "delete it",
    }
}

/// Item-level options.
///
/// The composer lives here rather than under `event`: `event` and `item`
/// are views over the same items, and what is edited is an iCalendar
/// object, which is what `item` names.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ItemConfig {
    /// Command an item is edited through, spawned on the path of a
    /// temporary iCalendar file it edits in place.
    ///
    /// A shell line or an argv list, the path appended as its last
    /// argument: `composer = "tcal edit"` runs `tcal edit <PATH>`.
    pub composer: Option<CommandConfig>,
    /// `item list` options.
    #[serde(default)]
    pub list: ItemListConfig,
}

/// `item list` options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ItemListConfig {
    /// Default `-s/--page-size` value for `item list`, 25 when unset.
    pub page_size: Option<u32>,
    /// Refused: `item list` prints plain text, in no colour.
    #[serde(default)]
    #[allow(dead_code)]
    pub table: Option<ItemListTableRemovedConfig>,
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
    /// Refused: the home is `root`, as a pimdir store's.
    #[serde(default)]
    #[allow(dead_code)]
    pub home_dir: Option<VdirHomeDirRemovedConfig>,
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
    /// Refused: calendula resolves no domain, `server` names the root.
    #[serde(default)]
    #[allow(dead_code)]
    pub discover: Option<CaldavDiscoverRemovedConfig>,
    /// DAV context root. Principal and calendar-home-set discovery
    /// start here.
    ///
    /// Accepts a full URL, a bare domain or `domain:port`, a bare
    /// authority defaulting to `https`. The host is ASCII, an
    /// internationalised one written in Punycode.
    pub server: Option<String>,
    /// Pre-resolved calendar home-set URL, skipping every discovery
    /// step. The host is ASCII, as for `server`.
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
    /// Authentication configuration.
    pub auth: CaldavAuthConfig,
}

/// ALPN identifiers CalDAV offers when `caldav.alpn` names none, pinning
/// the HTTP/1.1 io-http speaks.
#[cfg(feature = "caldav")]
pub fn default_caldav_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
}

removed_key! {
    /// The `caldav.discover` key, refused by name, calendula resolving no
    /// domain.
    #[cfg_attr(not(feature = "caldav"), allow(dead_code))]
    pub CaldavDiscoverRemovedConfig: Key {
        name: "caldav.discover",
        why: "calendula resolving no domain",
        instead: "name the server in `caldav.server` instead (Fastmail's \
                  `https://caldav.fastmail.com/dav/`, Google's \
                  `https://apidata.googleusercontent.com/caldav/v2/`)",
    }

    /// The `vdir.home-dir` key, refused by name, the vdir home being
    /// `vdir.root`.
    #[cfg_attr(not(feature = "vdir"), allow(dead_code))]
    pub VdirHomeDirRemovedConfig: Key {
        name: "vdir.home-dir",
        why: "the vdir home being named `root` as a store root is",
        instead: "rename it `vdir.root`, the path unchanged",
    }
}

/// CalDAV authentication configuration.
#[cfg(feature = "caldav")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum CaldavAuthConfig {
    /// No credentials, for a server that asks for none.
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
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
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

#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
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
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
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
}

/// Which TLS implementation carries the connection.
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TlsProviderConfig {
    /// The pure-Rust rustls stack.
    Rustls,
    /// The platform's own TLS stack.
    NativeTls,
}

/// rustls-specific options.
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RustlsConfig {
    /// Crypto provider. Unset prefers ring, else aws-lc-rs.
    pub crypto: Option<RustlsCryptoConfig>,
}

/// Which cryptographic provider backs rustls.
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RustlsCryptoConfig {
    /// The aws-lc-rs provider.
    Aws,
    /// The ring provider.
    Ring,
}

#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
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
            client: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use pimalaya_config::sample::{Variant, variants};
    use tempfile::NamedTempFile;

    use crate::config::*;

    /// The sections of config.sample.toml, one of which a user uncomments,
    /// each with whether this build compiles its backend.
    const SECTIONS: [(&str, bool); 7] = [
        ("# CalDAV", cfg!(feature = "caldav")),
        ("# Google Calendar", cfg!(feature = "gcal")),
        ("# Google CalDAV", cfg!(feature = "caldav")),
        ("# Microsoft Graph", cfg!(feature = "msgraph")),
        ("# Fastmail", cfg!(feature = "caldav")),
        ("# vdir", cfg!(feature = "vdir")),
        ("# pimdir", cfg!(feature = "pimdir")),
    ];

    /// The home directory the tilde in a test path stands for.
    #[cfg(any(
        feature = "vdir",
        feature = "caldav",
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

    /// `vdir.home-dir`, the vdir home 0.2 read, is refused by name in the
    /// family sentence, alone or beside `vdir.root`.
    #[cfg(feature = "vdir")]
    #[test]
    fn a_vdir_home_dir_is_refused_by_name() {
        let sentence = "The `vdir.home-dir` key is removed, the vdir home being \
                        named `root` as a store root is: rename it `vdir.root`, \
                        the path unchanged, see MIGRATION.md";

        for vdir in [
            "vdir.home-dir = \"~/calendars\"\n",
            "vdir.root = \"~/calendars\"\nvdir.home-dir = \"~/calendars\"\n",
        ] {
            let document = format!("[accounts.a]\n{vdir}");
            let err = toml::from_str::<Config>(&document).unwrap_err().to_string();
            assert!(err.contains(sentence), "{err}");
        }
    }

    #[cfg(feature = "pimdir")]
    #[test]
    fn a_pimdir_root_expands_its_tilde_at_deserialize() {
        let config: PimdirConfig = toml::from_str(r#"root = "~/store""#).unwrap();
        assert_eq!(config.root, home().join("store"));
    }

    /// The optional path takes the same treatment, and an absent key
    /// still reaches `None` rather than the deserializer.
    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    #[test]
    fn a_tls_certificate_expands_its_tilde_and_stays_optional() {
        let config: TlsConfig = toml::from_str(r#"cert = "~/ca.pem""#).unwrap();
        assert_eq!(config.cert, Some(home().join("ca.pem")));

        let config: TlsConfig = toml::from_str("").unwrap();
        assert_eq!(config.cert, None);
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

    #[test]
    fn a_leftover_todo_table_is_refused_naming_unfog() {
        let document = "[todo.list]\npage-size = 50\n\n[accounts.personal]\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();

        assert!(err.contains("The `todo` table is removed"), "{err}");
        assert!(err.contains("unfog"), "{err}");
        assert!(err.contains("see MIGRATION.md"), "{err}");
        assert!(err.contains("line 1"), "{err}");
    }

    #[test]
    fn a_leftover_journal_table_in_an_account_is_refused_naming_journal() {
        let document = "[accounts.personal.journal]\nlist.page-size = 50\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();

        assert!(err.contains("The `journal` table is removed"), "{err}");
        assert!(err.contains("moved to journal"), "{err}");
        assert!(err.contains("see MIGRATION.md"), "{err}");
        assert!(err.contains("line 1"), "{err}");
    }

    /// A list keeps its page size, and the colour tables of an earlier
    /// calendula are refused by name, at the top level and under an
    /// account.
    #[test]
    fn a_colour_table_is_refused_by_name() {
        let document = "[event.list]\npage-size = 10\n[accounts.a.item.list]\npage-size = 5\n";
        toml::from_str::<Config>(document).unwrap();

        let tables = [
            (
                "calendar.list",
                "calendar.list.table.name-color = \"red\"\n",
            ),
            ("event.list.table", "event.list.table.id-color = \"red\"\n"),
            ("item.list.table", "item.list.table.etag-color = \"red\"\n"),
        ];

        for (table, line) in tables {
            let root = format!("{line}[accounts.a]\n");
            let account = format!("[accounts.a]\n{line}");

            for document in [root, account] {
                let err = toml::from_str::<Config>(&document).unwrap_err();
                let err = err.to_string();
                assert!(
                    err.contains(&format!("The `{table}` table is removed")),
                    "{err}"
                );
            }
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
                        gcal.auth.token.raw = \"token\"\n\
                        msgraph.auth.token.raw = \"token\"\n";
        toml::from_str::<Config>(document).unwrap();
    }

    /// The rendering tables of an earlier calendula are refused by name,
    /// at the top level and under an account.
    #[test]
    fn a_leftover_table_or_account_table_is_refused() {
        for document in [
            "[table]\npreset = \"ASCII_FULL\"\n[accounts]\n",
            "[accounts.a.table]\narrangement = \"disabled\"\n",
        ] {
            let err = toml::from_str::<Config>(document).unwrap_err().to_string();
            assert!(err.contains("The `table` table is removed"), "{err}");
            assert!(err.contains("delete it, see MIGRATION.md"), "{err}");
        }

        let document = "[account.list.table]\nname-color = \"red\"\n[accounts]\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();
        assert!(err.contains("The `account` table is removed"), "{err}");
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
    #[cfg(all(feature = "caldav", feature = "gcal", feature = "msgraph"))]
    #[test]
    fn an_account_proxy_fills_every_backend_naming_none() {
        use pimalaya_config::toml::TomlConfig;

        let document = "[accounts.a]\n\
                        proxy.url = \"socks5h://localhost:1080\"\n\
                        caldav.server = \"example.org\"\n\
                        caldav.auth = \"none\"\n\
                        gcal.auth.token.raw = \"token\"\n\
                        msgraph.auth.token.raw = \"token\"\n\
                        msgraph.proxy.url = \"http://proxy.example.org:3128\"\n";
        let mut config: Config = toml::from_str(document).unwrap();
        let (_, account) = config.take_account(Some("a")).unwrap();

        let caldav = account.caldav.unwrap().proxy.unwrap();
        let gcal = account.gcal.unwrap().proxy.unwrap();
        let msgraph = account.msgraph.unwrap().proxy.unwrap();

        assert_eq!(caldav.url, "socks5h://localhost:1080");
        assert_eq!(gcal.url, "socks5h://localhost:1080");
        assert_eq!(msgraph.url, "http://proxy.example.org:3128");
    }

    /// A password needs a username, and both reach the runtime proxy, the
    /// password through the resolver.
    #[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
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

    /// Checks config.sample.toml: one `[accounts.example]` table and no
    /// other, every section parsed through the loader `-c` uses with every
    /// variant on top, `example` resolving as the default account and
    /// declaring a backend when this build compiles it.
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let sample = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = sample.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        let headings = SECTIONS.map(|(heading, _)| heading);

        for Variant { name, text } in variants(&sample, &headings) {
            // NOTE: a build without a backend loads its block unread.
            let heading = name.split(", ").next().unwrap_or_default();
            let compiled = !SECTIONS.contains(&(heading, false));

            let context = format!("config.sample.toml with {name} uncommented");

            let file = NamedTempFile::new().unwrap();
            fs::write(file.path(), &text).unwrap();

            let mut config = Config::load(&[file.path().to_path_buf()])
                .unwrap_or_else(|err| panic!("{context}: {err:?}"));
            #[cfg_attr(
                not(any(
                    feature = "vdir",
                    feature = "caldav",
                    feature = "pimdir",
                    feature = "gcal",
                    feature = "msgraph"
                )),
                allow(unused_variables)
            )]
            let (account, account_config) = config
                .take_account(None)
                .unwrap_or_else(|err| panic!("{context}: {err:?}"));

            let blocks: &[bool] = &[
                #[cfg(feature = "caldav")]
                account_config.caldav.is_some(),
                #[cfg(feature = "gcal")]
                account_config.gcal.is_some(),
                #[cfg(feature = "msgraph")]
                account_config.msgraph.is_some(),
                #[cfg(feature = "vdir")]
                account_config.vdir.is_some(),
                #[cfg(feature = "pimdir")]
                account_config.pimdir.is_some(),
            ];

            assert_eq!(account, "example", "{context}");
            assert!(
                !compiled || blocks.contains(&true),
                "{context}: no backend declared"
            );
        }
    }
}

/// Every network backend reaches its server through the proxy it names,
/// checked against a local HTTP proxy that reads the `CONNECT` and refuses
/// it, so nothing goes past it.
#[cfg(all(test, any(feature = "caldav", feature = "gcal", feature = "msgraph")))]
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
