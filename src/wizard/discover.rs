//! # Configuration wizard
//!
//! Discovers one account from a single prompt and tests it, where it lands
//! being [`super::configure`]'s business.
//!
//! The prompt takes an email address, a server URL or a local folder path,
//! and its shape orients the setup: an address or bare domain runs
//! io-pim-discovery's parallel search, whose best route is picked without a
//! question and shown in a review ([`super::plan`]); a folder is a vdir home
//! or pimdir store, told apart by its own markers.
//!
//! A `caldav://` or `https://` URL names the context root outright, which is
//! calendula's one deliberate deviation from himalaya's wizard: a self-hosted
//! Radicale or Baikal publishes neither an SRV record nor a `.well-known`
//! redirect, so nothing else reaches it. Discovering nothing asks for that
//! URL instead of stopping.
//!
//! The sign-in follows the review, tested as soon as it is typed
//! ([`super::sign_in`]).

use std::path::Path;

use anyhow::{Result, bail};
#[cfg(feature = "caldav")]
use pimalaya_cli::wizard::keyring::TokenBroker;
use pimalaya_cli::{prompt, spinner::Spinner};
use url::Url;

use crate::config::AccountConfig;
#[cfg(any(feature = "vdir", feature = "pimdir"))]
use crate::{
    account::check::{all_ok, check_account},
    backend::Backend,
    wizard::local,
};
#[cfg(feature = "caldav")]
use crate::{
    config::CONFIG_SAMPLE_URL,
    wizard::{
        plan::{self, Plan},
        search,
        sign_in::{Identity, is_address},
    },
};

/// The endpoint prompt label.
const ENDPOINT_PROMPT: &str = "Email, server or folder:";

/// Discovers one account from a single prompt, signs it in and tests it, and
/// hands back its name and configuration.
///
/// The name is derived from the input and suffixed until `taken` does not
/// hold it, before any sign-in: the Ortie account and the keyring entry
/// derive from it. Where the account lands is
/// [`configure`](super::configure)'s business, and no welcome renders here,
/// whoever reached this point having been told what the wizard is.
pub fn run(taken: &[String]) -> Result<(String, AccountConfig)> {
    let input = prompt::text::<&str>(ENDPOINT_PROMPT, None)?;
    let input = input.trim();

    if input.is_empty() {
        bail!("Empty input: enter an email address, a server URL, or a folder path");
    }

    let name = account_name(&default_account_name(input), taken);

    let account = if is_path(input) {
        configure_local(input)?
    } else if input.contains("://") {
        configure_manual(&name, input)?
    } else {
        configure_discovery(&name, input)?
    };

    Ok((name, account))
}

/// Runs the discovery flow for an email address or a bare domain.
///
/// Searches the routes to the calendars reachable from it and picks the best
/// one; discovering nothing asks for a server URL, an empty answer stopping
/// the wizard at the documented sample.
#[cfg(feature = "caldav")]
fn configure_discovery(name: &str, input: &str) -> Result<AccountConfig> {
    let email = if input.contains('@') {
        input.to_owned()
    } else {
        format!("@{input}")
    };

    let domain = email.rsplit_once('@').map_or(input, |(_, domain)| domain);
    let ortie = TokenBroker::Ortie.installed();

    let spinner = Spinner::start(format!("Looking up {domain}"));

    let routes = match search::search(&email, ortie) {
        Ok(routes) => routes,
        Err(err) => {
            spinner.failure(format!("Cannot look up {domain}"));
            return Err(err);
        }
    };

    let pick = match routes.first() {
        Some(auto) => {
            spinner.success(format!("Looked up {domain}"));
            auto.clone()
        }
        None => {
            spinner.failure(format!("Nothing could be discovered for {domain}"));

            match plan::prompt_custom()? {
                Some(route) => route,
                None => bail!(
                    "Cannot configure `{input}` without a server URL, write the \
                     account by hand starting from the documented sample:\n  \
                     {CONFIG_SAMPLE_URL}"
                ),
            }
        }
    };

    let identity = Identity {
        name: name.to_owned(),
        address: is_address(input).then(|| input.to_owned()),
        domain: Some(domain.to_owned()),
    };

    let plan = Plan {
        routes,
        pick,
        ortie,
    };

    settle(plan, &identity)
}

#[cfg(not(feature = "caldav"))]
fn configure_discovery(_name: &str, input: &str) -> Result<AccountConfig> {
    bail!("`{input}` looks like an address, but discovery needs the CalDAV backend")
}

/// Configures a typed server URL: the context root is taken as given, and
/// only how it signs in is asked.
#[cfg(feature = "caldav")]
fn configure_manual(name: &str, input: &str) -> Result<AccountConfig> {
    let pick = plan::custom_route(plan::server_url(input)?)?;

    let identity = Identity {
        name: name.to_owned(),
        address: None,
        domain: None,
    };

    let plan = Plan {
        routes: Vec::new(),
        pick,
        ortie: TokenBroker::Ortie.installed(),
    };

    settle(plan, &identity)
}

#[cfg(not(feature = "caldav"))]
fn configure_manual(_name: &str, input: &str) -> Result<AccountConfig> {
    bail!("`{input}` is a server URL, but no CalDAV backend is compiled in")
}

/// Reviews the plan and signs its pick in, back to the review until a
/// sign-in passes its test.
#[cfg(feature = "caldav")]
fn settle(mut plan: Plan, identity: &Identity) -> Result<AccountConfig> {
    loop {
        plan.review()?;

        if let Some(account) = identity.sign_in(&plan.pick)? {
            return Ok(account);
        }
    }
}

/// Configures a local backend from a typed folder path, then tests it.
#[cfg(any(feature = "vdir", feature = "pimdir"))]
fn configure_local(input: &str) -> Result<AccountConfig> {
    let raw = input.strip_prefix("file://").unwrap_or(input);
    let root = shellexpand::tilde(raw).into_owned();

    if !Path::new(&root).is_dir() {
        bail!("No such folder `{raw}`");
    }

    let mut account = AccountConfig::default();

    match local::configure(root.into())? {
        #[cfg(feature = "vdir")]
        local::Local::Vdir(config) => account.vdir = Some(config),
        #[cfg(feature = "pimdir")]
        local::Local::Pimdir(config) => account.pimdir = Some(config),
    }

    let spinner = Spinner::start("Testing account configuration");
    let checks = check_account(&account, Backend::Auto);

    if !all_ok(&checks) {
        spinner.failure("Account configuration test failed");

        let reasons: Vec<String> = checks
            .iter()
            .filter_map(|check| {
                let err = check.error.as_ref()?;
                Some(format!("{}: {err}", check.backend))
            })
            .collect();

        bail!(
            "Account configuration test failed\n  {}",
            reasons.join("\n  ")
        );
    }

    spinner.success("Account configuration is valid");

    Ok(account)
}

#[cfg(not(any(feature = "vdir", feature = "pimdir")))]
fn configure_local(input: &str) -> Result<AccountConfig> {
    bail!("`{input}` looks like a folder path, but no local backend is compiled in")
}

/// The name discovery proposes, suffixed until `taken` does not hold it.
///
/// Not prompted, the name being only the TOML table key. It still has to be
/// free: a second `[accounts.<name>]` table makes the whole document fail to
/// parse, taking the working accounts down with it.
fn account_name(base: &str, taken: &[String]) -> String {
    if !taken.iter().any(|name| name == base) {
        return base.to_string();
    }

    let mut suffix = 2;

    loop {
        let name = format!("{base}-{suffix}");

        if !taken.contains(&name) {
            return name;
        }

        suffix += 1;
    }
}

/// Proposes an account name from the input shape: the first domain label of
/// an address, host or bare domain, or the folder name of a local path.
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
    use crate::wizard::discover::*;

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
    fn a_taken_name_gets_a_suffix() {
        let taken = [String::from("perso"), String::from("perso-2")];

        assert_eq!(account_name("perso", &[]), "perso");
        assert_eq!(account_name("perso", &taken), "perso-3");
        assert_eq!(account_name("work", &taken), "work");
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
}
