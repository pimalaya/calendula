//! Input-driven service discovery for the wizard.
//!
//! The typed address or domain feeds io-pim-discovery's parallel
//! discovery (fixed provider rules, PACC, Mozilla autoconfig, and the
//! RFC 6764 CalDAV resolve), and every reachable service becomes one
//! selectable entry carrying the authentication capabilities it
//! advertised. The concrete method is picked once the service is
//! chosen, so a service appears exactly once in the list.
//!
//! calendula speaks one calendar protocol over the network, so the
//! discovery surface is narrow: one entry per reachable CalDAV context
//! root. It stays worth running, because it is what turns an email
//! address into a server without asking anyone to know their provider's
//! DAV URL.

use std::{collections::BTreeSet, env, fmt, time::Duration};

use anyhow::Result;
use io_pim_discovery::compose::{
    client::DiscoveryComposeClientStd,
    config::{DiscoveryAuthMethod, DiscoveryEndpoint, DiscoveryService},
};
use pimalaya_stream::tls::{Rustls, Tls};
use url::Url;

use crate::caldav::client::resolver;

/// Upper bound on the parallel discovery fan-out.
///
/// An unreachable endpoint (a firewalled port, a black-hole host) must
/// not stall the interactive wizard, so a mechanism that has not
/// reported by then is abandoned and only what completed in time is
/// offered.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(8);

/// One selectable way to reach the account's calendars, carrying the
/// authentication capabilities the service advertised.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discovered {
    /// The CalDAV context root to configure.
    pub server: Url,
    /// The login the mechanism advertised, usually the email address.
    pub username: Option<String>,
    /// What the service accepts, folded across its methods.
    pub auth: AuthCaps,
}

/// The authentication capabilities a service advertised, folded across
/// all its discovered methods.
///
/// It drives the per-service auth prompt: which HTTP schemes to offer,
/// and whether the OAuth token brokers appear. calendula reads a token
/// an external manager (Ortie, pizauth, oama) issues but never runs a
/// grant itself, so OAuth is not a method of its own: it only unlocks
/// the brokers behind the API-token flow.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AuthCaps {
    /// HTTP Basic, a username and password, often an app password.
    pub basic: bool,
    /// A static bearer token.
    pub bearer: bool,
    /// An OAuth 2.0 grant is advertised, so a broker can issue a token.
    pub oauth: bool,
}

impl AuthCaps {
    /// Whether any capability was advertised. When none was, the auth
    /// prompt offers every method, so the user is never left without a
    /// choice.
    pub fn any(self) -> bool {
        self.basic || self.bearer || self.oauth
    }

    /// Whether a token, static or broker-issued, is on offer.
    pub fn token(self) -> bool {
        self.bearer || self.oauth
    }
}

impl fmt::Display for Discovered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CalDAV {}", self.server)
    }
}

impl Discovered {
    /// The best default login for the credential prompt: the advertised
    /// username when it looks like an address, else the searched email
    /// when the user typed a full one, else nothing (a bare domain,
    /// whose synthesized `@domain` form is rejected here).
    pub fn login_default(&self, email: &str) -> Option<String> {
        self.username
            .clone()
            .filter(|username| looks_like_address(username))
            .or_else(|| looks_like_address(email).then(|| email.to_owned()))
    }
}

/// Searches every calendar service reachable from `email`, one entry
/// per distinct context root.
pub fn search(email: &str) -> Result<Vec<Discovered>> {
    let client = DiscoveryComposeClientStd::new(discovery_resolver(), discovery_tls());
    let services = BTreeSet::from([DiscoveryService::Caldav]);
    let configs = client.compose_all_within(email, services, DISCOVERY_TIMEOUT)?;

    let mut found: Vec<Discovered> = Vec::new();

    for config in configs {
        if config.service != DiscoveryService::Caldav {
            continue;
        }

        let DiscoveryEndpoint::Http(raw) = &config.endpoint else {
            continue;
        };

        let Ok(server) = Url::parse(raw) else {
            continue;
        };

        // Mechanisms overlap: SRV and PACC routinely name the same
        // root, and offering it twice is a choice with no difference.
        if let Some(existing) = found.iter_mut().find(|entry| entry.server == server) {
            let caps = caps_of(&config.auth);
            existing.auth.basic |= caps.basic;
            existing.auth.bearer |= caps.bearer;
            existing.auth.oauth |= caps.oauth;
            continue;
        }

        found.push(Discovered {
            server,
            username: config.username.clone(),
            auth: caps_of(&config.auth),
        });
    }

    Ok(found)
}

/// Folds a service's advertised methods into its [`AuthCaps`]: password
/// into `basic`, bearer into `bearer`, and every OAuth grant into
/// `oauth`, which only unlocks the token brokers.
fn caps_of(auth: &[DiscoveryAuthMethod]) -> AuthCaps {
    let mut caps = AuthCaps::default();

    for method in auth {
        match method {
            DiscoveryAuthMethod::Password => caps.basic = true,
            DiscoveryAuthMethod::Bearer => caps.bearer = true,
            _ => caps.oauth = true,
        }
    }

    caps
}

/// Whether a value is a full `local@domain` address, both parts
/// non-empty, rejecting the bare-domain `@domain` form discovery
/// synthesizes.
fn looks_like_address(value: &str) -> bool {
    value
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && !domain.is_empty())
}

/// The resolver discovery queries: the `CALENDULA_DNS_RESOLVER`
/// override first, then the system resolver, then the Cloudflare
/// default. This keeps the domain off a third-party resolver by
/// default, and works around networks blocking the fallback.
fn discovery_resolver() -> Url {
    if let Ok(resolver) = env::var("CALENDULA_DNS_RESOLVER")
        && let Ok(url) = resolver.parse()
    {
        return url;
    }

    resolver()
}

/// The TLS profile the HTTPS-bound mechanisms use; they only speak
/// HTTP/1.1 to `.well-known` endpoints.
fn discovery_tls() -> Tls {
    Tls {
        rustls: Rustls {
            alpn: vec!["http/1.1".into()],
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_fold_each_method_onto_its_own_axis() {
        let oauth = DiscoveryAuthMethod::OauthIssuer("https://issuer".into());

        assert_eq!(
            caps_of(&[DiscoveryAuthMethod::Password]),
            AuthCaps {
                basic: true,
                ..Default::default()
            }
        );

        // The Fastmail shape: a bearer token plus a grant and no Basic
        // is one API-token method whose brokers are unlocked.
        let fastmail = caps_of(&[DiscoveryAuthMethod::Bearer, oauth]);
        assert!(fastmail.token());
        assert!(!fastmail.basic);
        assert!(fastmail.any());
    }

    #[test]
    fn no_advertised_method_reads_as_no_capability_at_all() {
        assert!(!AuthCaps::default().any());
        assert!(!AuthCaps::default().token());
    }

    #[test]
    fn the_login_default_rejects_the_synthesized_bare_domain() {
        let entry = Discovered {
            server: "https://dav.example.org/".parse().unwrap(),
            username: None,
            auth: AuthCaps::default(),
        };

        assert_eq!(
            entry.login_default("alice@example.org").as_deref(),
            Some("alice@example.org")
        );
        assert_eq!(entry.login_default("@example.org"), None);
    }

    #[test]
    fn an_advertised_username_wins_over_the_searched_address() {
        let entry = Discovered {
            server: "https://dav.example.org/".parse().unwrap(),
            username: Some("alice.doe@example.org".into()),
            auth: AuthCaps::default(),
        };

        assert_eq!(
            entry.login_default("alice@example.org").as_deref(),
            Some("alice.doe@example.org")
        );
    }
}
