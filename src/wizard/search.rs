//! # Service discovery
//!
//! Input-driven discovery for the wizard. The typed address or domain feeds
//! io-pim-discovery's parallel search (the provider rules, matched by domain
//! or MX, PACC, autoconfig, and the RFC 6764 CalDAV resolve).
//!
//! Every way to reach the calendars becomes one [`Route`]: a protocol, a
//! server and a sign-in. Two routes reading the same are one, and the list
//! is ranked so that its first is the Auto pick of [`super::plan`].

use std::{collections::BTreeSet, env, fmt, time::Duration};

use anyhow::Result;
use io_pim_discovery::compose::{
    client::DiscoveryComposeClientStd,
    config::{DiscoveryAuthMethod, DiscoveryEndpoint, DiscoveryService, DiscoveryServiceConfig},
};
use pimalaya_stream::tls::{Rustls, Tls};
use url::Url;

use crate::caldav::client::resolver;

/// Upper bound on the parallel discovery fan-out.
///
/// An unreachable endpoint (a firewalled port, a black-hole host) must not
/// stall the interactive wizard, so mechanisms that have not reported by then
/// are abandoned and only what completed in time is offered.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(8);

/// A calendar protocol the wizard configures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    /// CalDAV (RFC 4791), the `caldav` backend.
    Caldav,
    /// The Google Calendar API, the `gcal` backend.
    #[cfg(feature = "gcal")]
    Gcal,
    /// Microsoft Graph calendars, the `msgraph` backend.
    #[cfg(feature = "msgraph")]
    Msgraph,
}

impl Protocol {
    /// The protocol serving a discovered service, `None` when this build
    /// configures none.
    pub fn of(service: DiscoveryService) -> Option<Self> {
        match service {
            DiscoveryService::Caldav => Some(Self::Caldav),
            #[cfg(feature = "gcal")]
            DiscoveryService::Gcal => Some(Self::Gcal),
            #[cfg(feature = "msgraph")]
            DiscoveryService::MsgraphCalendar => Some(Self::Msgraph),
            _ => None,
        }
    }

    /// The name the review and the change list show.
    pub fn name(self) -> &'static str {
        match self {
            Self::Caldav => "CalDAV",
            #[cfg(feature = "gcal")]
            Self::Gcal => "Google Calendar API",
            #[cfg(feature = "msgraph")]
            Self::Msgraph => "Microsoft Graph",
        }
    }

    /// The io-pim-discovery wire name, the service Ortie signs in for.
    pub fn service(self) -> &'static str {
        match self {
            Self::Caldav => "caldav",
            #[cfg(feature = "gcal")]
            Self::Gcal => "gcal",
            #[cfg(feature = "msgraph")]
            Self::Msgraph => "msgraphCalendar",
        }
    }

    /// The configuration block, which also suffixes the keyring entry.
    pub fn backend(self) -> &'static str {
        match self {
            Self::Caldav => "caldav",
            #[cfg(feature = "gcal")]
            Self::Gcal => "gcal",
            #[cfg(feature = "msgraph")]
            Self::Msgraph => "msgraph",
        }
    }

    /// Whether this is a provider's own API, which only a provider rule
    /// names and whose endpoint is fixed.
    pub fn is_api(self) -> bool {
        !matches!(self, Self::Caldav)
    }
}

/// How a route signs in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignIn {
    /// A login and a password, often an app password.
    Password,
    /// A static API token, sent as a bearer token.
    Token,
    /// An OAuth 2.0 grant, whatever grants the server offers.
    Oauth,
}

impl SignIn {
    /// The position of this sign-in among those of one server: OAuth first
    /// when Ortie is on the `PATH`, then a token, then a password, and the
    /// other way around without it.
    fn rank(self, ortie: bool) -> u8 {
        match (self, ortie) {
            (Self::Oauth, true) | (Self::Password, false) => 0,
            (Self::Token, _) => 1,
            (Self::Password, true) | (Self::Oauth, false) => 2,
        }
    }
}

impl fmt::Display for SignIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Password => f.write_str("Password"),
            Self::Token => f.write_str("API token"),
            Self::Oauth => f.write_str("OAuth 2.0"),
        }
    }
}

/// One way to reach the account's calendars.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Route {
    /// The protocol, and so the backend configured.
    pub protocol: Protocol,
    /// The CalDAV context root, or the API's fixed endpoint.
    pub server: Url,
    /// The login the mechanism advertised.
    pub username: Option<String>,
    /// How it signs in.
    pub sign_in: SignIn,
}

/// Searches every route to the calendars reachable from `email`, ranked for
/// whether Ortie is on the `PATH`.
pub fn search(email: &str, ortie: bool) -> Result<Vec<Route>> {
    let client = DiscoveryComposeClientStd::new(discovery_resolver(), discovery_tls());
    let services = BTreeSet::from([
        DiscoveryService::Caldav,
        DiscoveryService::Gcal,
        DiscoveryService::MsgraphCalendar,
    ]);
    let configs = client.compose_all_within(email, services, DISCOVERY_TIMEOUT)?;

    Ok(rank(routes(&configs), ortie))
}

/// The routes `configs` offer, one per sign-in each advertises.
///
/// Every OAuth grant of a server folds into one OAuth route, and a route
/// reading as one already listed (same protocol, server and sign-in) is
/// dropped. A server advertising nothing gets a password route.
fn routes(configs: &[DiscoveryServiceConfig]) -> Vec<Route> {
    let mut routes: Vec<Route> = Vec::new();

    for config in configs {
        let Some(protocol) = Protocol::of(config.service) else {
            continue;
        };

        let DiscoveryEndpoint::Http(raw) = &config.endpoint else {
            continue;
        };

        let Ok(server) = Url::parse(raw) else {
            continue;
        };

        if server.scheme() != "https" {
            continue;
        }

        let mut sign_ins: Vec<SignIn> = config
            .auth
            .iter()
            .map(|method| match method {
                DiscoveryAuthMethod::Password => SignIn::Password,
                DiscoveryAuthMethod::Bearer => SignIn::Token,
                _ => SignIn::Oauth,
            })
            .collect();

        if sign_ins.is_empty() {
            sign_ins.push(SignIn::Password);
        }

        for sign_in in sign_ins {
            let listed = routes.iter().any(|route| {
                route.protocol == protocol && route.server == server && route.sign_in == sign_in
            });

            if listed {
                continue;
            }

            routes.push(Route {
                protocol,
                server: server.clone(),
                username: config.username.clone(),
                sign_in,
            });
        }
    }

    routes
}

/// Orders `routes` so the first is the Auto pick: a provider's own API
/// before CalDAV, servers in the order discovery found them, and within one
/// server the sign-ins by [`SignIn::rank`].
fn rank(routes: Vec<Route>, ortie: bool) -> Vec<Route> {
    let mut keyed: Vec<((bool, usize, u8), Route)> = routes
        .iter()
        .map(|route| {
            let server = routes
                .iter()
                .position(|other| other.protocol == route.protocol && other.server == route.server)
                .unwrap_or_default();

            let key = (!route.protocol.is_api(), server, route.sign_in.rank(ortie));

            (key, route.clone())
        })
        .collect();

    keyed.sort_by_key(|(key, _)| *key);
    keyed.into_iter().map(|(_, route)| route).collect()
}

/// The resolver discovery queries: the `CALENDULA_DNS_RESOLVER` override
/// first, then the system resolver, then the Cloudflare default.
///
/// Preferring the system resolver keeps the domain off a third party; the
/// override works around networks blocking the fallback.
fn discovery_resolver() -> Url {
    if let Ok(resolver) = env::var("CALENDULA_DNS_RESOLVER")
        && let Ok(url) = resolver.parse()
    {
        return url;
    }

    resolver()
}

/// The TLS profile the HTTPS-bound mechanisms use; they only speak HTTP/1.1
/// to `.well-known` endpoints.
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
    use io_pim_discovery::compose::config::DiscoveryConfigSource;

    use crate::wizard::search::*;

    fn config(
        service: DiscoveryService,
        url: &str,
        auth: Vec<DiscoveryAuthMethod>,
    ) -> DiscoveryServiceConfig {
        DiscoveryServiceConfig {
            service,
            endpoint: DiscoveryEndpoint::Http(url.into()),
            username: Some(String::from("jane@example.org")),
            auth,
            source: DiscoveryConfigSource::Pacc,
            resolved: false,
        }
    }

    fn grant(scope: &str) -> DiscoveryAuthMethod {
        DiscoveryAuthMethod::OauthAuthorizationCodeGrant {
            authorization_endpoint: String::from("https://auth.example.org/authorize"),
            token_endpoint: String::from("https://auth.example.org/token"),
            scope: Some(scope.into()),
            resource: None,
            scopes_supported: Vec::new(),
        }
    }

    fn summary(routes: &[Route]) -> Vec<(&'static str, String, SignIn)> {
        routes
            .iter()
            .map(|route| {
                (
                    route.protocol.name(),
                    route.server.to_string(),
                    route.sign_in,
                )
            })
            .collect()
    }

    #[test]
    fn a_server_offering_oauth_through_several_grants_is_one_route() {
        let configs = [config(
            DiscoveryService::Caldav,
            "https://dav.example.org/dav/",
            vec![
                DiscoveryAuthMethod::Password,
                grant("calendar"),
                DiscoveryAuthMethod::OauthIssuer(String::from("https://auth.example.org")),
            ],
        )];

        let sign_ins: Vec<SignIn> = routes(&configs)
            .into_iter()
            .map(|route| route.sign_in)
            .collect();

        assert_eq!(sign_ins, [SignIn::Password, SignIn::Oauth]);
    }

    #[test]
    fn two_routes_reading_the_same_are_one() {
        let configs = [
            config(
                DiscoveryService::Caldav,
                "https://dav.example.org/dav/",
                vec![DiscoveryAuthMethod::Password],
            ),
            config(
                DiscoveryService::Caldav,
                "https://dav.example.org/dav/",
                vec![DiscoveryAuthMethod::Password, DiscoveryAuthMethod::Bearer],
            ),
            config(
                DiscoveryService::Caldav,
                "https://other.example.org/dav/",
                vec![DiscoveryAuthMethod::Password],
            ),
        ];

        assert_eq!(
            summary(&routes(&configs)),
            [
                (
                    "CalDAV",
                    String::from("https://dav.example.org/dav/"),
                    SignIn::Password
                ),
                (
                    "CalDAV",
                    String::from("https://dav.example.org/dav/"),
                    SignIn::Token
                ),
                (
                    "CalDAV",
                    String::from("https://other.example.org/dav/"),
                    SignIn::Password
                ),
            ]
        );
    }

    #[test]
    fn a_server_advertising_nothing_signs_in_with_a_password() {
        let configs = [config(
            DiscoveryService::Caldav,
            "https://dav.example.org/",
            Vec::new(),
        )];

        let sign_ins: Vec<SignIn> = routes(&configs)
            .into_iter()
            .map(|route| route.sign_in)
            .collect();

        assert_eq!(sign_ins, [SignIn::Password]);
    }

    #[test]
    fn a_service_no_backend_serves_is_skipped() {
        let configs = [config(
            DiscoveryService::Carddav,
            "https://dav.example.org/",
            vec![DiscoveryAuthMethod::Password],
        )];

        assert!(routes(&configs).is_empty());
    }

    #[test]
    fn within_one_server_oauth_leads_only_with_ortie() {
        let configs = [config(
            DiscoveryService::Caldav,
            "https://dav.example.org/dav/",
            vec![
                DiscoveryAuthMethod::Password,
                DiscoveryAuthMethod::Bearer,
                grant("calendar"),
            ],
        )];

        let ranked = |ortie| -> Vec<SignIn> {
            rank(routes(&configs), ortie)
                .into_iter()
                .map(|route| route.sign_in)
                .collect()
        };

        assert_eq!(
            ranked(true),
            [SignIn::Oauth, SignIn::Token, SignIn::Password]
        );
        assert_eq!(
            ranked(false),
            [SignIn::Password, SignIn::Token, SignIn::Oauth]
        );
    }

    #[test]
    fn servers_keep_the_order_discovery_found_them_in() {
        let configs = [
            config(
                DiscoveryService::Caldav,
                "https://first.example.org/",
                vec![DiscoveryAuthMethod::Password],
            ),
            config(
                DiscoveryService::Caldav,
                "https://second.example.org/",
                vec![grant("calendar")],
            ),
        ];

        let servers: Vec<String> = rank(routes(&configs), true)
            .into_iter()
            .map(|route| route.server.to_string())
            .collect();

        assert_eq!(
            servers,
            ["https://first.example.org/", "https://second.example.org/"]
        );
    }

    #[cfg(feature = "gcal")]
    #[test]
    fn a_provider_api_ranks_before_its_caldav() {
        let configs = [
            config(
                DiscoveryService::Caldav,
                "https://dav.example.org/caldav/v2/",
                vec![grant("calendar")],
            ),
            config(
                DiscoveryService::Gcal,
                "https://api.example.org/calendar/v3/",
                vec![grant("calendar")],
            ),
        ];

        for ortie in [true, false] {
            let protocols: Vec<Protocol> = rank(routes(&configs), ortie)
                .into_iter()
                .map(|route| route.protocol)
                .collect();

            assert_eq!(protocols, [Protocol::Gcal, Protocol::Caldav]);
        }
    }

    #[cfg(feature = "msgraph")]
    #[test]
    fn graph_calendars_route_to_the_msgraph_backend() {
        let configs = [config(
            DiscoveryService::MsgraphCalendar,
            "https://graph.example.org/v1.0/",
            vec![grant("Calendars.ReadWrite")],
        )];

        let routes = routes(&configs);

        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].protocol, Protocol::Msgraph);
        assert_eq!(routes[0].protocol.service(), "msgraphCalendar");
        assert_eq!(routes[0].sign_in, SignIn::Oauth);
    }
}
