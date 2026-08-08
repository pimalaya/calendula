//! Configuration wizard.
//!
//! Run on bare `calendula` (no subcommand), and proposed by
//! `cli::resolve_config` when no configuration file is found. It opens
//! with a welcome banner on stderr, then either saves the resulting
//! account to a file (offered when writing to a terminal) or prints it
//! as a ready-to-save TOML document on stdout, so `calendula > <config>`
//! works as the write-back when stdout is redirected.
//!
//! One prompt takes an email address, a server URL, or a local folder
//! path, and its shape orients the setup:
//!
//! An email or a bare domain runs io-pim-discovery's parallel discovery
//! and every reachable CalDAV service becomes one selectable entry;
//! picking one then prompts its authentication method among those
//! advertised. A `caldav://` or HTTP-family URL names the context root
//! outright, which is how a self-hosted server publishing neither an
//! SRV record nor a `.well-known` redirect gets configured: that is
//! calendula's one deliberate deviation from himalaya's wizard, whose
//! providers are near-universally discoverable. An existing folder is a
//! local vdir home or pimdir store, told apart by its own markers.
//!
//! calendula runs no OAuth 2.0 grant itself: a grant only unlocks the
//! external token brokers behind the API-token prompt.

use std::{collections::HashMap, fmt, fs, io::IsTerminal, path::Path};

use anyhow::{Context, Result, bail};
use pimalaya_cli::{printer::Printer, prompt, spinner::Spinner};
use pimalaya_config::toml as config_toml;
use serde::{Serialize, Serializer};
use url::Url;

#[cfg(feature = "pimdir")]
use crate::config::PimdirConfig;
#[cfg(feature = "vdir")]
use crate::config::VdirConfig;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
use crate::wizard::local;
use crate::{
    account::check::{all_ok, check_account},
    backend::Backend,
    config::{AccountConfig, Config},
};
#[cfg(feature = "caldav")]
use crate::{
    config::CaldavConfig,
    wizard::{caldav, search},
};

/// The endpoint prompt label.
const ENDPOINT_PROMPT: &str = "Email, server or folder:";

/// The documented sample configuration, shown in the welcome banner and
/// pointed at when discovery finds nothing to configure.
const CONFIG_SAMPLE_URL: &str =
    "https://github.com/pimalaya/calendula/blob/master/config.sample.toml";

/// The backend config a flow produced, folded into a fresh
/// [`AccountConfig`] afterwards.
enum Chosen {
    #[cfg(feature = "caldav")]
    Caldav(Box<CaldavConfig>),
    #[cfg(feature = "vdir")]
    Vdir(VdirConfig),
    #[cfg(feature = "pimdir")]
    Pimdir(PimdirConfig),
}

/// Runs the wizard and either saves the resulting [`Config`] to a file
/// or prints it as a ready-to-save TOML document.
///
/// A welcome message renders on stderr first, skipped in JSON mode, to
/// frame what calendula is and what the wizard does. The generated
/// config is then offered for saving when writing to a terminal; when
/// stdout is redirected or in JSON mode it goes straight to stdout, so
/// the redirect and any script keep working.
pub fn run(printer: &mut impl Printer) -> Result<()> {
    if !printer.is_json() {
        print_welcome();
    }

    let input = prompt::text::<&str>(ENDPOINT_PROMPT, None)?;
    let input = input.trim();

    if input.is_empty() {
        bail!("Empty input: enter an email address, a server URL, or a folder path");
    }

    // NOTE: the account name is only the TOML table key, so it is
    // derived from the input rather than prompted; renaming it is
    // editing that key.
    let account_name = default_account_name(input);
    let account = build_account(&account_name, input)?;

    // Test before printing: a bad credential or endpoint stops the
    // wizard here, rather than yielding a configuration that cannot
    // connect.
    let spinner = Spinner::start("Testing account configuration");
    let checks = check_account(&account, Backend::Auto);

    if !all_ok(&checks) {
        spinner.failure("Account configuration test failed");
        let reasons: Vec<String> = checks
            .iter()
            .filter_map(|check| {
                check
                    .error
                    .as_ref()
                    .map(|err| format!("{}: {err}", check.backend))
            })
            .collect();
        bail!(
            "Account configuration test failed\n  {}",
            reasons.join("\n  ")
        );
    }

    spinner.success("Account configuration is valid");

    let config = Config {
        accounts: HashMap::from([(account_name, account)]),
        ..Default::default()
    };

    if printer.is_json() || !std::io::stdout().is_terminal() {
        return printer.out(GeneratedConfig(config));
    }

    save_or_print(printer, config)
}

/// Prints a welcome banner on stderr framing the project and the
/// wizard, so bare `calendula` explains itself before dropping into
/// prompts. On stderr so it never pollutes a redirected document.
fn print_welcome() {
    eprintln!();
    eprintln!("Welcome to calendula, the CLI to manage calendars.");
    eprintln!();
    eprintln!("calendula talks to your existing calendars over CalDAV, or reads a");
    eprintln!("local vdir home or pimdir store. Before you can list or edit an");
    eprintln!("event, it needs to know about one account.");
    eprintln!();
    eprintln!("This wizard discovers a provider's settings from your email address");
    eprintln!("(or a server URL, or a local folder path), tests the connection and");
    eprintln!("generates a ready-to-use configuration it can save for you.");
    eprintln!();
    eprintln!("Every field is documented in the sample configuration:");
    eprintln!("  {CONFIG_SAMPLE_URL}");
    eprintln!();
}

/// Offers to save the generated config to a file, falling back to
/// printing it on stdout when the user declines or an existing file
/// must not be overwritten. Prompts and confirmations render on stderr.
fn save_or_print(printer: &mut impl Printer, config: Config) -> Result<()> {
    if !prompt::bool("Save this configuration to a file, or print it?", true)? {
        return printer.out(GeneratedConfig(config));
    }

    let default = default_config_path();
    let path = prompt::text("Configuration file path:", default.as_deref())?;
    let path = shellexpand::full(path.trim())?.into_owned();
    let path = Path::new(&path);

    // Bare `calendula` runs the wizard even when a configuration
    // already exists, so guard the default path: never clobber without
    // confirmation, and fall back to printing so the generated config
    // is never lost.
    if path.exists()
        && !prompt::bool(
            format!("`{}` already exists. Overwrite it?", path.display()),
            false,
        )?
    {
        return printer.out(GeneratedConfig(config));
    }

    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("Create config directory `{}`", parent.display()))?;
    }

    fs::write(path, GeneratedConfig(config).to_string())
        .with_context(|| format!("Write config file `{}`", path.display()))?;

    eprintln!();
    eprintln!("Configuration saved to {}.", path.display());
    eprintln!("Run `calendula calendar list` to read your calendars.");

    Ok(())
}

/// The default config path, used to seed the save prompt; `None` when
/// no config directory resolves.
fn default_config_path() -> Option<String> {
    let path = dirs::config_dir()?
        .join(env!("CARGO_PKG_NAME"))
        .join("config.toml");

    Some(path.to_string_lossy().into_owned())
}

/// The account the wizard produced, rendered as a ready-to-save TOML
/// document, or serialized as an object in JSON mode.
struct GeneratedConfig(Config);

impl fmt::Display for GeneratedConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = config_toml::to_string(&self.0).map_err(|_| fmt::Error)?;
        write!(f, "{toml}")
    }
}

impl Serialize for GeneratedConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// Orients the setup from the input shape, then folds the chosen
/// backend into a fresh [`AccountConfig`].
///
/// The account is left non-default, so the wizard's output does not
/// hijack the default when merged into a configuration that already has
/// one. Being false, `default` is dropped from the printed TOML; the
/// user marks their choice with `default = true`.
fn build_account(account_name: &str, input: &str) -> Result<AccountConfig> {
    let chosen = if is_path(input) {
        configure_local(input)?
    } else if let Some(server) = server_url(input)? {
        configure_manual(account_name, server)?
    } else {
        configure_discovery(account_name, input)?
    };

    let mut account = AccountConfig {
        default: false,
        ..Default::default()
    };

    match chosen {
        #[cfg(feature = "caldav")]
        Chosen::Caldav(config) => account.caldav = Some(*config),
        #[cfg(feature = "vdir")]
        Chosen::Vdir(config) => account.vdir = Some(config),
        #[cfg(feature = "pimdir")]
        Chosen::Pimdir(config) => account.pimdir = Some(config),
    }

    Ok(account)
}

/// Runs the discovery flow for an email address or a bare domain:
/// search the CalDAV services reachable from it, let the user pick one,
/// then configure it. When nothing is discovered the wizard stops and
/// points at the sample configuration.
#[cfg(feature = "caldav")]
fn configure_discovery(account_name: &str, input: &str) -> Result<Chosen> {
    let email = if input.contains('@') {
        input.to_owned()
    } else {
        format!("@{input}")
    };

    let spinner = Spinner::start("Searching for server settings");
    let found = search::search(&email)?;

    if found.is_empty() {
        spinner.failure("No configuration found");
        return stop_undiscovered(input);
    }

    spinner.success(format!("Found {} configuration(s)", found.len()));

    let default = found.first().cloned();
    let choice = prompt::item("Choose a configuration:", found, default)?;
    let config = caldav::configure_discovered(account_name, &email, &choice)?;

    Ok(Chosen::Caldav(Box::new(config)))
}

#[cfg(not(feature = "caldav"))]
fn configure_discovery(_account_name: &str, input: &str) -> Result<Chosen> {
    bail!("`{input}` looks like an address, but no network backend is compiled in")
}

/// Configures a typed server URL: the context root is taken as given
/// and only the credentials are prompted.
#[cfg(feature = "caldav")]
fn configure_manual(account_name: &str, server: Url) -> Result<Chosen> {
    let config = caldav::configure_manual(account_name, server)?;
    Ok(Chosen::Caldav(Box::new(config)))
}

#[cfg(not(feature = "caldav"))]
fn configure_manual(_account_name: &str, server: Url) -> Result<Chosen> {
    bail!("`{server}` is a server URL, but no network backend is compiled in")
}

/// Stops the wizard when discovery found nothing for `input`: it says
/// where to go next and errors out, rather than dropping into a
/// hand-entry flow for fields nobody knows.
#[cfg(feature = "caldav")]
fn stop_undiscovered(input: &str) -> Result<Chosen> {
    bail!(
        "Could not automatically discover a configuration for `{input}`.\n\n\
         Pass your CalDAV server URL directly (for example \
         `https://dav.example.org/`) if you know it, or write the account by \
         hand starting from the documented sample:\n  {CONFIG_SAMPLE_URL}"
    )
}

/// Configures a local backend from a typed folder path.
#[cfg(any(feature = "vdir", feature = "pimdir"))]
fn configure_local(input: &str) -> Result<Chosen> {
    let raw = input.strip_prefix("file://").unwrap_or(input);
    let root = shellexpand::tilde(raw).into_owned();

    if !Path::new(&root).is_dir() {
        bail!("No such folder `{raw}`");
    }

    Ok(match local::configure(root.into())? {
        #[cfg(feature = "vdir")]
        local::Local::Vdir(config) => Chosen::Vdir(config),
        #[cfg(feature = "pimdir")]
        local::Local::Pimdir(config) => Chosen::Pimdir(config),
    })
}

#[cfg(not(any(feature = "vdir", feature = "pimdir")))]
fn configure_local(input: &str) -> Result<Chosen> {
    bail!("`{input}` looks like a folder path, but no local backend is compiled in")
}

/// The server URL a `scheme://` input names, or `None` when the input
/// is an address or a bare domain to discover from.
///
/// `caldav` and `caldavs` are accepted as aliases for `http` and
/// `https`, since that is how a DAV endpoint is often written down.
fn server_url(input: &str) -> Result<Option<Url>> {
    if !input.contains("://") {
        return Ok(None);
    }

    let normalized = match input.split_once("://") {
        Some(("caldav", rest)) => format!("http://{rest}"),
        Some(("caldavs", rest)) => format!("https://{rest}"),
        _ => input.to_owned(),
    };

    let url = Url::parse(&normalized).with_context(|| format!("Invalid server URL `{input}`"))?;

    match url.scheme() {
        "http" | "https" => Ok(Some(url)),
        other => bail!("Unsupported server scheme `{other}`: CalDAV runs over HTTP"),
    }
}

/// Proposes an account name from the input shape: the first label of
/// the domain of an address, host or bare domain, or the folder name of
/// a local path.
fn default_account_name(input: &str) -> String {
    if is_path(input) {
        let raw = input.strip_prefix("file://").unwrap_or(input);
        return Path::new(raw)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("personal")
            .to_owned();
    }

    if let Ok(url) = Url::parse(input)
        && let Some(host) = url.host_str()
    {
        return first_label(host);
    }

    match input.rsplit_once('@') {
        Some((_, domain)) => first_label(domain),
        None => first_label(input),
    }
}

/// The first dot-separated label of a host or domain.
fn first_label(host: &str) -> String {
    host.split('.').next().unwrap_or(host).to_owned()
}

/// Whether the input names a filesystem path (absolute, home-relative,
/// explicitly relative, or a `file://` URL) rather than an endpoint.
fn is_path(input: &str) -> bool {
    input.starts_with("file://")
        || input.starts_with('/')
        || input.starts_with('~')
        || input.starts_with("./")
        || input.starts_with("../")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_account_name_defaults_to_the_first_domain_label() {
        assert_eq!(default_account_name("clement.douin@posteo.net"), "posteo");
        assert_eq!(default_account_name("alice@mail.example.co.uk"), "mail");
        assert_eq!(default_account_name("@posteo.net"), "posteo");
        assert_eq!(default_account_name("posteo.net"), "posteo");
        assert_eq!(default_account_name("https://dav.example.org/"), "dav");
    }

    #[test]
    fn an_account_name_defaults_to_the_last_path_component() {
        assert_eq!(default_account_name("/home/alice/calendars/work"), "work");
        assert_eq!(default_account_name("~/calendars/personal"), "personal");
        assert_eq!(default_account_name("file:///var/cal/shared"), "shared");
    }

    #[test]
    fn a_path_is_told_apart_from_an_endpoint_by_its_prefix() {
        assert!(is_path("/srv/calendars"));
        assert!(is_path("~/calendars"));
        assert!(is_path("./calendars"));
        assert!(is_path("file:///srv/calendars"));

        assert!(!is_path("alice@example.org"));
        assert!(!is_path("example.org"));
        assert!(!is_path("https://dav.example.org/"));
    }

    #[test]
    fn only_a_scheme_input_names_a_server_and_dav_schemes_are_aliases() {
        assert_eq!(server_url("alice@example.org").unwrap(), None);
        assert_eq!(server_url("example.org").unwrap(), None);

        assert_eq!(
            server_url("https://dav.example.org/dav/").unwrap(),
            Some("https://dav.example.org/dav/".parse().unwrap())
        );
        assert_eq!(
            server_url("caldavs://dav.example.org/dav/").unwrap(),
            Some("https://dav.example.org/dav/".parse().unwrap())
        );
        assert_eq!(
            server_url("caldav://dav.example.org/dav/").unwrap(),
            Some("http://dav.example.org/dav/".parse().unwrap())
        );
    }

    #[test]
    fn a_non_http_scheme_is_rejected_by_name() {
        let err = server_url("imaps://mail.example.org")
            .unwrap_err()
            .to_string();
        assert!(err.contains("Unsupported server scheme `imaps`"), "{err}");
    }

    #[test]
    fn the_generated_document_keeps_the_account_table_as_a_header() {
        #[allow(unused_mut)]
        let mut account = AccountConfig::default();

        #[cfg(feature = "vdir")]
        {
            account.vdir = Some(VdirConfig {
                home_dir: "/srv/calendars".into(),
            });
        }

        let config = Config {
            accounts: HashMap::from([("posteo".to_string(), account)]),
            ..Default::default()
        };
        let rendered = GeneratedConfig(config).to_string();

        assert!(rendered.contains("[accounts.posteo]"), "{rendered}");
    }
}
