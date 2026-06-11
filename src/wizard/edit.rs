//! Interactive editor for an existing (or new) account.

use std::path::Path;

#[cfg(feature = "caldav")]
use anyhow::anyhow;
use anyhow::{Result, bail};
#[cfg(feature = "caldav")]
use log::debug;
use log::info;
use pimalaya_cli::prompt;
#[cfg(feature = "caldav")]
use pimalaya_cli::{
    spinner::Spinner,
    wizard::caldav::{
        self as caldav_wizard, CaldavAuth, CaldavSecret, Encryption as CaldavEncryption,
        WizardCaldavConfig,
    },
};
#[cfg(feature = "caldav")]
use pimalaya_config::secret::Secret;
#[cfg(feature = "caldav")]
use pimconf::{
    rfc6186::types::SrvService,
    rfc6764::{client::DiscoveryWebdavClientStd, types::WebdavSrvReport},
};
#[cfg(feature = "caldav")]
use url::Url;

use crate::config::{AccountConfig, Config};
#[cfg(feature = "caldav")]
use crate::config::{CaldavAuthConfig, CaldavConfig};

#[cfg(feature = "caldav")]
const DEFAULT_RESOLVER: &str = "tcp://1.1.1.1:53";

/// Edits (or creates) the account named `account_name`. Writes the
/// updated config to `target` before returning.
pub fn edit_account(target: &Path, mut config: Config, account_name: &str) -> Result<Config> {
    let existing = config.accounts.remove(account_name);

    #[cfg(feature = "caldav")]
    let (local_part, domain) = {
        let email = prompt::text::<&str>("Email address:", None)?;
        let (local, dom) = email
            .rsplit_once('@')
            .ok_or_else(|| anyhow!("Invalid email address `{email}`: missing `@`"))?;
        (local.to_owned(), dom.to_owned())
    };

    let is_first_account = config.accounts.is_empty() && existing.is_none();
    let default = existing
        .as_ref()
        .map(|a| a.default)
        .unwrap_or(is_first_account);

    let kinds = available_backends();
    if kinds.is_empty() {
        bail!("No calendar backend compiled in; rebuild with `vdir` or `caldav`");
    }

    let initial_kind = existing.as_ref().and_then(initial_backend);
    let kind = prompt::item("Backend:", kinds.iter().copied(), initial_kind)?;

    let mut account = AccountConfig {
        default,
        ..AccountConfig::default()
    };

    match kind {
        #[cfg(feature = "vdir")]
        BackendKind::Vdir => {
            let prev = existing.as_ref().and_then(|a| a.vdir.as_ref());
            let home = prompt::text(
                "Vdir home directory:",
                prev.map(|c| c.home_dir.to_string_lossy().into_owned())
                    .as_deref(),
            )?;
            account.vdir = Some(crate::config::VdirConfig {
                home_dir: shellexpand::full(&home)
                    .map(|s| std::path::PathBuf::from(s.into_owned()))
                    .unwrap_or_else(|_| home.into()),
            });
        }
        #[cfg(feature = "caldav")]
        BackendKind::Caldav => {
            let prev_wizard = existing
                .as_ref()
                .and_then(|a| a.caldav.as_ref())
                .map(caldav_to_wizard_defaults);
            let srv_defaults = srv_discover_caldav(&domain);
            let defaults = prev_wizard.as_ref().or(srv_defaults.as_ref());

            let cfg = caldav_wizard::run(account_name, &local_part, &domain, defaults)?;
            account.caldav = Some(wizard_to_caldav(cfg)?);
        }
    }

    config.accounts.insert(account_name.to_owned(), account);
    config.write(target)?;
    info!("Configuration written to {}", target.display());

    Ok(config)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackendKind {
    #[cfg(feature = "vdir")]
    Vdir,
    #[cfg(feature = "caldav")]
    Caldav,
}

impl std::fmt::Display for BackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "vdir")]
            Self::Vdir => write!(f, "vdir"),
            #[cfg(feature = "caldav")]
            Self::Caldav => write!(f, "caldav"),
        }
    }
}

#[allow(clippy::vec_init_then_push, unused_mut)]
fn available_backends() -> Vec<BackendKind> {
    let mut backends = Vec::new();
    #[cfg(feature = "vdir")]
    backends.push(BackendKind::Vdir);
    #[cfg(feature = "caldav")]
    backends.push(BackendKind::Caldav);
    backends
}

fn initial_backend(existing: &AccountConfig) -> Option<BackendKind> {
    #[cfg(feature = "vdir")]
    if existing.vdir.is_some() {
        return Some(BackendKind::Vdir);
    }
    #[cfg(feature = "caldav")]
    if existing.caldav.is_some() {
        return Some(BackendKind::Caldav);
    }
    let _ = existing;
    None
}

/// Runs the RFC 6764 SRV discovery chain for `_caldav` / `_caldavs`
/// against `domain` and folds the best record into a wizard default.
/// Prefers `_caldavs` (TLS) over `_caldav` (plain) when both are
/// published.
#[cfg(feature = "caldav")]
fn srv_discover_caldav(domain: &str) -> Option<WizardCaldavConfig> {
    let spinner = Spinner::start(format!("Probing SRV records for {domain}"));
    let resolver: Url = DEFAULT_RESOLVER
        .parse()
        .expect("DEFAULT_RESOLVER must be a valid URL");
    let mut client = DiscoveryWebdavClientStd::new(resolver);

    match client.discover(domain) {
        Ok(report) if !report_is_empty(&report) => {
            spinner.success(srv_summary(domain, &report));
            caldav_from_report(&report)
        }
        Ok(_) => {
            spinner.failure(format!("SRV: no records for {domain}"));
            None
        }
        Err(err) => {
            debug!("SRV discovery for {domain} failed: {err}");
            spinner.failure(format!("SRV: no records for {domain}"));
            None
        }
    }
}

#[cfg(feature = "caldav")]
fn report_is_empty(report: &WebdavSrvReport) -> bool {
    report.caldav.is_none()
        && report.caldavs.is_none()
        && report.carddav.is_none()
        && report.carddavs.is_none()
}

#[cfg(feature = "caldav")]
fn srv_summary(domain: &str, report: &WebdavSrvReport) -> String {
    let mut protos = Vec::with_capacity(2);
    if report.caldav.is_some() || report.caldavs.is_some() {
        protos.push("CalDAV");
    }
    if report.carddav.is_some() || report.carddavs.is_some() {
        protos.push("CardDAV");
    }
    format!("SRV: discovered {} for {domain}", protos.join(" + "))
}

#[cfg(feature = "caldav")]
fn caldav_from_report(report: &WebdavSrvReport) -> Option<WizardCaldavConfig> {
    let (service, encryption) = if let Some(s) = report.caldavs.as_ref() {
        (s, CaldavEncryption::Tls)
    } else if let Some(s) = report.caldav.as_ref() {
        (s, CaldavEncryption::None)
    } else {
        return None;
    };

    Some(srv_service_to_wizard(service, encryption))
}

#[cfg(feature = "caldav")]
fn srv_service_to_wizard(service: &SrvService, encryption: CaldavEncryption) -> WizardCaldavConfig {
    WizardCaldavConfig {
        host: service.host.clone(),
        port: service.port,
        encryption,
        home_url: None,
        // NOTE: empty Basic placeholder; the wizard re-prompts for
        // strategy when the username field is empty.
        auth: CaldavAuth::Basic {
            username: String::new(),
            secret: CaldavSecret::Raw(String::new().into()),
        },
    }
}

/// Folds an existing [`CaldavConfig`] into a [`WizardCaldavConfig`]
/// so the wizard can populate prompt defaults from it.
#[cfg(feature = "caldav")]
fn caldav_to_wizard_defaults(existing: &CaldavConfig) -> WizardCaldavConfig {
    // The wizard model is host/port/encryption; the persisted config is
    // a bare `discover` domain (or an explicit `server` URL), so derive
    // the prompt defaults from whichever is set.
    let host = existing
        .server
        .as_ref()
        .and_then(|url| url.host_str().map(str::to_owned))
        .or_else(|| existing.discover.clone())
        .unwrap_or_default();
    let scheme = existing.server.as_ref().map(Url::scheme);
    let encryption = if matches!(scheme, Some("http")) {
        CaldavEncryption::None
    } else {
        CaldavEncryption::Tls
    };
    let port = existing
        .server
        .as_ref()
        .and_then(Url::port)
        .unwrap_or(default_port(encryption));

    let home_url = existing.home.as_ref().map(Url::to_string);

    let auth = match &existing.auth {
        CaldavAuthConfig::Basic { username, .. } => CaldavAuth::Basic {
            username: username.clone(),
            secret: CaldavSecret::Raw(String::new().into()),
        },
        CaldavAuthConfig::Bearer { .. } => CaldavAuth::Bearer {
            secret: CaldavSecret::Raw(String::new().into()),
        },
        CaldavAuthConfig::None => CaldavAuth::Basic {
            username: String::new(),
            secret: CaldavSecret::Raw(String::new().into()),
        },
    };

    WizardCaldavConfig {
        host,
        port,
        encryption,
        home_url,
        auth,
    }
}

#[cfg(feature = "caldav")]
fn default_port(encryption: CaldavEncryption) -> u16 {
    match encryption {
        CaldavEncryption::Tls => 443,
        CaldavEncryption::None => 80,
    }
}

#[cfg(feature = "caldav")]
fn wizard_to_caldav(cfg: WizardCaldavConfig) -> Result<CaldavConfig> {
    // Persist the host as a bare `discover` domain; pimconf re-derives
    // scheme + port (and follows `.well-known`) at connection time.
    let home = match cfg.home_url {
        Some(raw) => Some(Url::parse(&raw).map_err(|err| anyhow!("Invalid home URL: {err}"))?),
        None => None,
    };

    Ok(CaldavConfig {
        discover: Some(cfg.host),
        server: None,
        home,
        tls: Default::default(),
        auth: caldav_auth_to_config(cfg.auth),
    })
}

#[cfg(feature = "caldav")]
fn caldav_auth_to_config(auth: CaldavAuth) -> CaldavAuthConfig {
    match auth {
        CaldavAuth::Basic { username, secret } => CaldavAuthConfig::Basic {
            username,
            password: caldav_secret_to_secret(secret),
        },
        CaldavAuth::Bearer { secret } => CaldavAuthConfig::Bearer {
            token: caldav_secret_to_secret(secret),
        },
    }
}

#[cfg(feature = "caldav")]
fn caldav_secret_to_secret(secret: CaldavSecret) -> Secret {
    match secret {
        CaldavSecret::Raw(s) => Secret::Raw(s),
        CaldavSecret::Command(cmd) => Secret::Command(pimalaya_config::command::shell(&cmd)),
    }
}
