//! The TOML configuration schema.
//!
//! A configuration is a top-level block of rendering options plus one
//! `[accounts.<name>]` block per account, each carrying an optional
//! sub-block per backend. The global block is folded under the selected
//! account at load time, so a value set once at the top applies
//! everywhere it is not overridden.
//!
//! `deny_unknown_fields` is set on the leaf blocks but deliberately not
//! on [`Config`] and [`AccountConfig`], so a future TUI reading the same
//! file can add its own sections without breaking this one.

use std::{collections::HashMap, path::PathBuf};

use crossterm::style::Color;
use pimalaya_cli::table::ContentArrangement;
#[cfg(any(feature = "caldav", feature = "gcal"))]
use pimalaya_config::secret::Secret;
use pimalaya_config::toml::TomlConfig;
#[cfg(feature = "caldav")]
use pimalaya_config::toml::shell_expanded_string;
use pimalaya_stream::tls::{Rustls, RustlsCrypto, Tls, TlsProvider};
use serde::{Deserialize, Serialize};

/// Global configuration.
///
/// Represents the whole TOML user's configuration file.
/// `deny_unknown_fields` is intentionally omitted so future TUI fields
/// can coexist; today only `[accounts.*]` plus the global rendering
/// sections are consumed.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub downloads_dir: Option<PathBuf>,
    #[serde(default)]
    pub table: TableConfig,
    #[serde(default)]
    pub calendar: CalendarConfig,
    #[serde(default)]
    pub event: EventConfig,
    #[serde(default)]
    pub todo: TodoConfig,
    #[serde(default)]
    pub journal: JournalConfig,
    #[serde(default)]
    pub item: ItemConfig,
    /// `account list` rendering options (global only).
    #[serde(default)]
    pub account: AccountListingConfig,
    pub accounts: HashMap<String, AccountConfig>,
}

impl TomlConfig for Config {
    type Account = AccountConfig;

    fn project_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn take_named_account(&mut self, name: &str) -> Option<(String, Self::Account)> {
        self.accounts.remove_entry(name)
    }

    fn take_default_account(&mut self) -> Option<(String, Self::Account)> {
        let name = self
            .accounts
            .iter()
            .find_map(|(name, account)| account.default.then(|| name.clone()))?;

        self.take_named_account(&name)
    }
}

/// Account configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AccountConfig {
    #[serde(default)]
    pub default: bool,

    pub downloads_dir: Option<PathBuf>,
    #[serde(default)]
    pub table: TableConfig,
    #[serde(default)]
    pub calendar: CalendarConfig,
    #[serde(default)]
    pub event: EventConfig,
    #[serde(default)]
    pub todo: TodoConfig,
    #[serde(default)]
    pub journal: JournalConfig,
    #[serde(default)]
    pub item: ItemConfig,

    #[cfg(feature = "vdir")]
    pub vdir: Option<VdirConfig>,
    #[cfg(feature = "pimdir")]
    pub pimdir: Option<PimdirConfig>,
    #[cfg(feature = "caldav")]
    pub caldav: Option<CaldavConfig>,
    #[cfg(feature = "gcal")]
    pub gcal: Option<GcalConfig>,
}

/// Calendar-level options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CalendarConfig {
    /// Calendar id used by `event` and `item` commands when their
    /// `-k/--calendar` flag is omitted.
    pub default: Option<String>,

    #[serde(default)]
    pub list: CalendarListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CalendarListConfig {
    #[serde(default)]
    pub table: CalendarListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CalendarListTableConfig {
    pub id_color: Option<Color>,
    pub name_color: Option<Color>,
    pub description_color: Option<Color>,
    pub color_color: Option<Color>,
}

/// Event-level rendering options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EventConfig {
    #[serde(default)]
    pub list: EventListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EventListConfig {
    /// Default `-s/--page-size` value for `events list`.
    pub page_size: Option<u32>,
    #[serde(default)]
    pub table: EventListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct EventListTableConfig {
    pub id_color: Option<Color>,
    pub summary_color: Option<Color>,
    pub start_color: Option<Color>,
    pub end_color: Option<Color>,
}

/// Todo-level rendering options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TodoConfig {
    #[serde(default)]
    pub list: TodoListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TodoListConfig {
    /// Default `-s/--page-size` value for `todos list`.
    pub page_size: Option<u32>,
    #[serde(default)]
    pub table: TodoListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TodoListTableConfig {
    pub id_color: Option<Color>,
    pub summary_color: Option<Color>,
    pub due_color: Option<Color>,
    pub status_color: Option<Color>,
}

/// Journal-level rendering options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct JournalConfig {
    #[serde(default)]
    pub list: JournalListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct JournalListConfig {
    /// Default `-s/--page-size` value for `journals list`.
    pub page_size: Option<u32>,
    #[serde(default)]
    pub table: JournalListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct JournalListTableConfig {
    pub id_color: Option<Color>,
    pub summary_color: Option<Color>,
    pub start_color: Option<Color>,
}

/// Item-level rendering options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ItemConfig {
    #[serde(default)]
    pub list: ItemListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ItemListConfig {
    /// Default `-s/--page-size` value for `items list`.
    pub page_size: Option<u32>,
    #[serde(default)]
    pub table: ItemListTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ItemListTableConfig {
    pub id_color: Option<Color>,
    pub etag_color: Option<Color>,
    pub size_color: Option<Color>,
}

/// `account list` rendering options. Top-level only.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingConfig {
    #[serde(default)]
    pub list: AccountListingListConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingListConfig {
    #[serde(default)]
    pub table: AccountListingTableConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct AccountListingTableConfig {
    pub name_color: Option<Color>,
    pub backends_color: Option<Color>,
    pub default_color: Option<Color>,
}

/// Global / per-account table rendering quirks shared across every list
/// command.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TableConfig {
    /// `comfy_table` preset string. Defaults to `UTF8_FULL_CONDENSED`.
    pub preset: Option<String>,
    /// Column-arrangement strategy. Defaults to `Dynamic`.
    pub arrangement: Option<TableArrangementConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TableArrangementConfig {
    #[default]
    Dynamic,
    DynamicFullWidth,
    Disabled,
}

impl From<TableArrangementConfig> for ContentArrangement {
    fn from(arrangement: TableArrangementConfig) -> Self {
        match arrangement {
            TableArrangementConfig::Dynamic => ContentArrangement::Dynamic,
            TableArrangementConfig::DynamicFullWidth => ContentArrangement::DynamicFullWidth,
            TableArrangementConfig::Disabled => ContentArrangement::Disabled,
        }
    }
}

/// vdir backend configuration.
#[cfg(feature = "vdir")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VdirConfig {
    /// Filesystem path of the vdir home: the directory holding one
    /// subdirectory per calendar. Shell-expanded before use, so `~`
    /// and environment variables both work.
    pub home_dir: PathBuf,
}

/// pimdir backend configuration.
///
/// A pimdir store is an offline cache a sync engine fills, not a
/// server: calendula reads what the store holds and stages its writes
/// for the next sync to push.
#[cfg(feature = "pimdir")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct PimdirConfig {
    /// The store directory, holding the SQLite index and the blob
    /// tree. Shell-expanded before use.
    pub root: PathBuf,
    /// The replica source name this client opens the store as.
    ///
    /// Reads are source-independent, but a staged write is attributed
    /// to this source, so for a change to propagate it must match the
    /// source the sync engine drives for this device. Usually left
    /// unset: a store synced as a single source is opened as that
    /// source. Set it only to disambiguate a store synced from two.
    #[serde(default)]
    pub source: Option<String>,
}

/// CalDAV backend configuration.
///
/// Locating the calendar home-set takes exactly one of three routes,
/// from most to least discovery: `discover` resolves a bare domain,
/// `server` names the context root to walk from, and `home` pins the
/// home-set outright.
#[cfg(feature = "caldav")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CaldavConfig {
    /// Bare domain resolved to a context root through RFC 6764 SRV
    /// records and the `.well-known` path. Convenient, but it costs
    /// DNS and HTTP round-trips on every run; prefer `server` once the
    /// answer is known.
    pub discover: Option<String>,
    /// DAV context root. Principal and calendar-home-set discovery
    /// start here, skipping the `.well-known` step.
    pub server: Option<url::Url>,
    /// Pre-resolved calendar home-set URL, skipping every discovery
    /// step.
    pub home: Option<url::Url>,
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// Authentication configuration.
    pub auth: CaldavAuthConfig,
}

/// CalDAV authentication configuration.
#[cfg(feature = "caldav")]
#[derive(Clone, Debug, Deserialize, Serialize)]
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GcalConfig {
    /// TLS configuration.
    #[serde(default)]
    pub tls: TlsConfig,
    /// Authentication configuration.
    pub auth: GcalAuthConfig,
}

/// Google Calendar authentication configuration.
///
/// A struct rather than an enumeration of kinds: the Calendar API
/// accepts an OAuth 2.0 bearer token and nothing else, so there is
/// nothing to choose between. It still nests under `auth`, so every
/// backend across the Pimalaya CLIs spells its credentials the same
/// way.
#[cfg(feature = "gcal")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GcalAuthConfig {
    /// The token, read from the configuration or from the standard
    /// output of a command. Google expires an access token within the
    /// hour, so a token broker is the usual answer, and a broker is a
    /// command like any other.
    pub token: Secret,
}

/// SSL/TLS configuration.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct TlsConfig {
    pub provider: Option<TlsProviderConfig>,
    #[serde(default)]
    pub rustls: RustlsConfig,
    pub cert: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TlsProviderConfig {
    Rustls,
    NativeTls,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RustlsConfig {
    pub crypto: Option<RustlsCryptoConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RustlsCryptoConfig {
    Aws,
    Ring,
}

impl From<TlsConfig> for Tls {
    fn from(config: TlsConfig) -> Self {
        Tls {
            provider: config.provider.map(|config| match config {
                TlsProviderConfig::Rustls => TlsProvider::Rustls,
                TlsProviderConfig::NativeTls => TlsProvider::NativeTls,
            }),
            rustls: Rustls {
                crypto: config.rustls.crypto.map(|config| match config {
                    RustlsCryptoConfig::Aws => RustlsCrypto::Aws,
                    RustlsCryptoConfig::Ring => RustlsCrypto::Ring,
                }),
                alpn: Vec::new(),
            },
            cert: config.cert,
        }
    }
}
