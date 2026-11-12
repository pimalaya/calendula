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
//! `deny_unknown_fields` is set on the leaf blocks but not on [`Config`]
//! and [`AccountConfig`], whose backend blocks exist only in the builds
//! carrying their feature: a build without one still loads a file naming
//! it. The tables calendula no longer reads are refused by name instead:
//! `todo` and `journal`, their domains having moved to unfog and journal,
//! and the `table` and `account` rendering tables, the output being plain
//! text.

use std::{collections::HashMap, path::PathBuf};

#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_config::secret::Secret;
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_config::toml::opt_shell_expanded_path;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
use pimalaya_config::toml::shell_expanded_path;
#[cfg(feature = "caldav")]
use pimalaya_config::toml::shell_expanded_string;
use pimalaya_config::{command::CommandConfig, toml::TomlConfig};
#[cfg(any(feature = "caldav", feature = "gcal", feature = "msgraph"))]
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsProvider};
use serde::{Deserialize, Deserializer, de::Error};
#[cfg(feature = "caldav")]
use url::Url;

/// The whole configuration file: global options and every account.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
        self.accounts.remove_entry(name)
    }
}

/// One account: the backends it reads calendars from, plus whatever it
/// overrides of the global options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
}

/// Calendar-level options.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CalendarConfig {
    /// Calendar id used by `event` and `item` commands when their
    /// `-k/--calendar` flag is omitted.
    pub default: Option<String>,
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
}

/// The `todo` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum TodoMovedConfig {}

impl<'de> Deserialize<'de> for TodoMovedConfig {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(D::Error::custom(
            "The `todo` table moved to `unfog`, the Pimalaya CLI for tasks; \
             remove it from the calendula configuration",
        ))
    }
}

/// The `journal` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum JournalMovedConfig {}

impl<'de> Deserialize<'de> for JournalMovedConfig {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(D::Error::custom(
            "The `journal` table moved to `journal`, the Pimalaya CLI for \
             journal entries; remove it from the calendula configuration",
        ))
    }
}

/// The `table` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum TableRemovedConfig {}

impl<'de> Deserialize<'de> for TableRemovedConfig {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(D::Error::custom(
            "The `table` table is gone, calendula printing plain text with \
             no border or colour; remove it from the calendula configuration",
        ))
    }
}

/// The top-level `account` table, which no configuration holds any more.
#[derive(Clone, Debug)]
pub enum AccountRemovedConfig {}

impl<'de> Deserialize<'de> for AccountRemovedConfig {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(D::Error::custom(
            "The `account` table is gone, `account list` printing plain text \
             with no colour; remove it from the calendula configuration",
        ))
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
}

/// vdir backend configuration.
#[cfg(feature = "vdir")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    /// Filesystem path of the vdir home: the directory holding one
    /// subdirectory per calendar. Shell-expanded at load, so `~` and
    /// environment variables both work.
    #[serde(deserialize_with = "shell_expanded_path")]
    pub home_dir: PathBuf,
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
/// Locating the calendar home-set takes one of three routes, from most
/// to least discovery: `discover` resolves a bare domain, `server`
/// names the context root to walk from, `home` pins the home-set.
#[cfg(feature = "caldav")]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CaldavConfig {
    /// Bare domain resolved to a context root through RFC 6764 SRV
    /// records and the `.well-known` path.
    ///
    /// It costs DNS and HTTP round-trips on every run: prefer `server`
    /// once the answer is known.
    pub discover: Option<String>,
    /// DAV context root. Principal and calendar-home-set discovery
    /// start here, skipping the `.well-known` step.
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
    /// Authentication configuration.
    pub auth: CaldavAuthConfig,
}

/// ALPN identifiers CalDAV offers when `caldav.alpn` names none, pinning
/// the HTTP/1.1 io-http speaks.
#[cfg(feature = "caldav")]
pub fn default_caldav_alpn() -> Vec<String> {
    vec![String::from("http/1.1")]
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
            cert: self.cert,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env::var, fs};

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
    fn home() -> PathBuf {
        PathBuf::from(var("HOME").expect("HOME must be set to expand a tilde"))
    }

    /// Expansion happens at deserialize, so no call site can read a
    /// literal `./~/…` directory under the current one.
    #[cfg(feature = "vdir")]
    #[test]
    fn a_vdir_home_expands_its_tilde_at_deserialize() {
        let config: VdirConfig = toml::from_str(r#"home-dir = "~/calendars""#).unwrap();
        assert_eq!(config.home_dir, home().join("calendars"));
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

        assert!(err.contains("`todo` table moved to `unfog`"), "{err}");
        assert!(err.contains("line 1"), "{err}");
    }

    #[test]
    fn a_leftover_journal_table_in_an_account_is_refused_naming_journal() {
        let document = "[accounts.personal.journal]\nlist.page-size = 50\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();

        assert!(err.contains("`journal` table moved to `journal`"), "{err}");
        assert!(err.contains("line 1"), "{err}");
    }

    /// A list keeps its page size, and a colour key of the former tables
    /// fails the load naming the block that held it.
    #[test]
    fn a_colour_key_is_refused() {
        let document = "[event.list]\npage-size = 10\n[accounts.a.item.list]\npage-size = 5\n";
        toml::from_str::<Config>(document).unwrap();

        for (document, unknown) in [
            (
                "[event.list.table]\nid-color = \"red\"\n[accounts]\n",
                "table",
            ),
            (
                "[accounts.a.item.list.table]\nid-color = \"red\"\n",
                "table",
            ),
            (
                "[calendar.list.table]\nname-color = \"red\"\n[accounts]\n",
                "list",
            ),
        ] {
            let err = toml::from_str::<Config>(document).unwrap_err().to_string();
            assert!(err.contains(&format!("unknown field `{unknown}`")), "{err}");
        }
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
            assert!(err.contains("The `table` table is gone"), "{err}");
        }

        let document = "[account.list.table]\nname-color = \"red\"\n[accounts]\n";
        let err = toml::from_str::<Config>(document).unwrap_err().to_string();
        assert!(err.contains("The `account` table is gone"), "{err}");
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

    /// Checks config.sample.toml: one `[accounts.example]` table and no
    /// other, the sections of the compiled backends each parsed through the
    /// loader `-c` uses with every variant on top, `example` resolving as
    /// the default account and declaring a backend.
    #[test]
    fn the_sample_configuration_parses_with_each_variant() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.sample.toml");
        let sample = fs::read_to_string(&path).unwrap();
        let headers: Vec<_> = sample.lines().filter(|l| l.starts_with('[')).collect();
        assert_eq!(headers, ["[accounts.example]"]);

        let headings = SECTIONS.map(|(heading, _)| heading);

        for Variant { name, text } in variants(&sample, &headings) {
            // NOTE: a build without a backend ignores its block.
            let heading = name.split(", ").next().unwrap_or_default();
            if SECTIONS.contains(&(heading, false)) {
                continue;
            }

            let context = format!("config.sample.toml with {name} uncommented");

            let file = NamedTempFile::new().unwrap();
            fs::write(file.path(), &text).unwrap();

            let mut config = Config::load(&[file.path().to_path_buf()])
                .unwrap_or_else(|err| panic!("{context}: {err:?}"));
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
            assert!(blocks.contains(&true), "{context}: no backend declared");
        }
    }
}
