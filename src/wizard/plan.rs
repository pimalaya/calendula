//! # Plan
//!
//! What the wizard sets up for the calendar: Auto, the first route
//! [`super::search`] ranked, picked without a question and shown in a review
//! in plain words. Every other route found, and a typed server URL, are one
//! change away.

use std::fmt;

use anyhow::{Context, Result, bail};
use pimalaya_cli::{
    prompt,
    wizard::review::{self, ReviewAction, ReviewRow},
};
use url::{Host, Url};

use crate::wizard::search::{Protocol, Route, SignIn};

/// The review row name of the one domain calendula sets up.
const DOMAIN: &str = "Calendar";

/// The ranked routes discovery found, and the one picked.
pub struct Plan {
    /// The ranked routes, the first being Auto; empty for a typed server.
    pub routes: Vec<Route>,
    /// The route the sign-in goes to.
    pub pick: Route,
    /// Whether Ortie is on the `PATH`, which words the OAuth sign-in.
    pub ortie: bool,
}

impl Plan {
    /// Shows the review until the user continues, applying each change.
    pub fn review(&mut self) -> Result<()> {
        loop {
            let rows = [ReviewRow {
                name: String::from(DOMAIN),
                line: self.pick.sentence(self.ortie),
            }];

            match review::prompt(&rows)? {
                ReviewAction::Continue => return Ok(()),
                ReviewAction::Change(_) => self.change()?,
            }
        }
    }

    /// Offers Auto, every route found and a custom server URL, then picks
    /// the answer. An empty server URL keeps the pick.
    fn change(&mut self) -> Result<()> {
        let labels = labels(&self.routes);
        let mut changes = Vec::new();

        if let Some(auto) = labels.first() {
            changes.push(Change::Auto(auto.clone()));
        }

        for (index, label) in labels.into_iter().enumerate() {
            changes.push(Change::Route { index, label });
        }

        changes.push(Change::Custom);

        let prompt = format!("Change {}:", DOMAIN.to_lowercase());

        match prompt::item(prompt, changes, None)? {
            Change::Auto(_) => self.pick = self.routes[0].clone(),
            Change::Route { index, .. } => self.pick = self.routes[index].clone(),
            Change::Custom => {
                if let Some(route) = prompt_custom()? {
                    self.pick = route;
                }
            }
        }

        Ok(())
    }
}

impl Route {
    /// The review line, in plain words: the protocol, the host when the
    /// endpoint is not a provider's fixed one, and how it signs in.
    pub fn sentence(&self, ortie: bool) -> String {
        let how = match self.sign_in {
            SignIn::Password => "with your password",
            SignIn::Token => "with an API token",
            SignIn::Oauth if ortie => "through your browser",
            SignIn::Oauth => "with a token from a broker",
        };

        let name = self.protocol.name();

        if self.protocol.is_api() {
            return format!("{name}, {how}");
        }

        format!("{name} at {}, {how}", self.host())
    }

    /// The change list entry, as `CalDAV · Password`.
    fn label(&self) -> String {
        format!("{} · {}", self.protocol.name(), self.sign_in)
    }

    /// The server host, with its port when the URL names one.
    fn host(&self) -> String {
        let host = self.server.host_str().unwrap_or_default();

        match self.server.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_owned(),
        }
    }
}

/// Prompts a server URL until one is accepted, then how it signs in.
///
/// An empty answer is `None`, which the caller reads as a stop.
pub fn prompt_custom() -> Result<Option<Route>> {
    let server = loop {
        let input = prompt::text::<&str>("Server URL:", None)?;
        let input = input.trim();

        if input.is_empty() {
            return Ok(None);
        }

        match server_url(input) {
            Ok(server) => break server,
            Err(err) => eprintln!("{err:#}"),
        }
    };

    custom_route(server).map(Some)
}

/// The route to a typed CalDAV server, asking how it signs in: a password,
/// selected, or an API token.
pub fn custom_route(server: Url) -> Result<Route> {
    let sign_ins = [SignIn::Password, SignIn::Token];
    let sign_in = prompt::item("Sign in with:", sign_ins, Some(SignIn::Password))?;

    Ok(Route {
        protocol: Protocol::Caldav,
        server,
        username: None,
        sign_in,
    })
}

/// Parses a typed CalDAV server URL, encrypted servers only.
///
/// `caldav` and `caldavs` read as `https`, as a DAV endpoint is often
/// written down, and a URL without a scheme as an `https` one. Plain `http`
/// is accepted on a loopback host only, a password never crossing a network
/// unencrypted.
pub fn server_url(input: &str) -> Result<Url> {
    let (scheme, rest) = input.split_once("://").unwrap_or(("https", input));

    let scheme = match scheme.to_ascii_lowercase().as_str() {
        "https" | "caldav" | "caldavs" => "https",
        "http" => "http",
        other => bail!("Unsupported server scheme `{other}`, CalDAV runs over `https`"),
    };

    let url = Url::parse(&format!("{scheme}://{rest}"))
        .with_context(|| format!("Invalid server URL `{input}`"))?;

    if scheme == "http" && !loopback(&url) {
        bail!("Unencrypted server `{input}` refused, only `https` is accepted");
    }

    Ok(url)
}

/// Whether `url` names this machine: `localhost` or a loopback address.
fn loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    }
}

/// One entry of the change list.
#[derive(Eq, PartialEq)]
enum Change {
    Auto(String),
    Route { index: usize, label: String },
    Custom,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto(label) => write!(f, "Auto ({label})"),
            Self::Route { label, .. } => f.write_str(label),
            Self::Custom => f.write_str("Custom server URL"),
        }
    }
}

/// The change list labels of `routes`, the host added where two would read
/// the same, the whole URL where the host still would.
fn labels(routes: &[Route]) -> Vec<String> {
    let labels: Vec<String> = routes.iter().map(Route::label).collect();

    routes
        .iter()
        .zip(&labels)
        .map(|(route, label)| {
            let twins: Vec<&Route> = routes
                .iter()
                .zip(&labels)
                .filter(|(_, other)| *other == label)
                .map(|(twin, _)| twin)
                .collect();

            if twins.len() == 1 {
                return label.clone();
            }

            let host = route.host();

            if twins.iter().filter(|twin| twin.host() == host).count() == 1 {
                return format!("{label} · {host}");
            }

            format!("{label} · {}", route.server)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::wizard::plan::*;

    fn route(protocol: Protocol, server: &str, sign_in: SignIn) -> Route {
        Route {
            protocol,
            server: server.parse().unwrap(),
            username: None,
            sign_in,
        }
    }

    #[test]
    fn a_review_line_names_the_host_and_the_sign_in_in_plain_words() {
        let caldav = |sign_in| route(Protocol::Caldav, "https://dav.example.org/dav/", sign_in);

        assert_eq!(
            caldav(SignIn::Password).sentence(true),
            "CalDAV at dav.example.org, with your password"
        );
        assert_eq!(
            caldav(SignIn::Token).sentence(true),
            "CalDAV at dav.example.org, with an API token"
        );
        assert_eq!(
            caldav(SignIn::Oauth).sentence(true),
            "CalDAV at dav.example.org, through your browser"
        );
        assert_eq!(
            caldav(SignIn::Oauth).sentence(false),
            "CalDAV at dav.example.org, with a token from a broker"
        );

        let port = route(
            Protocol::Caldav,
            "https://localhost:5232/",
            SignIn::Password,
        );
        assert_eq!(
            port.sentence(false),
            "CalDAV at localhost:5232, with your password"
        );
    }

    #[cfg(feature = "gcal")]
    #[test]
    fn a_provider_api_reads_without_its_host() {
        let gcal = route(
            Protocol::Gcal,
            "https://api.example.org/calendar/v3/",
            SignIn::Oauth,
        );

        assert_eq!(
            gcal.sentence(true),
            "Google Calendar API, through your browser"
        );
    }

    #[test]
    fn a_change_label_names_the_server_only_where_two_would_read_the_same() {
        let routes = [
            route(
                Protocol::Caldav,
                "https://dav.example.org/a/",
                SignIn::Password,
            ),
            route(
                Protocol::Caldav,
                "https://dav.example.org/a/",
                SignIn::Oauth,
            ),
            route(
                Protocol::Caldav,
                "https://other.example.org/",
                SignIn::Oauth,
            ),
            route(
                Protocol::Caldav,
                "https://twin.example.org/a/",
                SignIn::Token,
            ),
            route(
                Protocol::Caldav,
                "https://twin.example.org/b/",
                SignIn::Token,
            ),
        ];

        assert_eq!(
            labels(&routes),
            [
                "CalDAV · Password",
                "CalDAV · OAuth 2.0 · dav.example.org",
                "CalDAV · OAuth 2.0 · other.example.org",
                "CalDAV · API token · https://twin.example.org/a/",
                "CalDAV · API token · https://twin.example.org/b/",
            ]
        );
    }

    #[test]
    fn a_dav_scheme_or_none_reads_as_https() {
        for input in [
            "https://dav.example.org/dav/",
            "caldavs://dav.example.org/dav/",
            "caldav://dav.example.org/dav/",
            "CalDAV://dav.example.org/dav/",
            "dav.example.org/dav/",
        ] {
            assert_eq!(
                server_url(input).unwrap().as_str(),
                "https://dav.example.org/dav/",
                "{input}"
            );
        }

        assert_eq!(
            server_url("dav.example.org:8443").unwrap().as_str(),
            "https://dav.example.org:8443/"
        );
    }

    #[test]
    fn a_plain_http_server_is_refused_by_name() {
        let err = server_url("http://dav.example.org/")
            .unwrap_err()
            .to_string();

        assert!(err.contains("`http://dav.example.org/`"), "{err}");
        assert!(err.contains("only `https`"), "{err}");
    }

    #[test]
    fn a_plain_http_server_on_this_machine_is_kept() {
        for input in [
            "http://localhost:5232/",
            "http://127.0.0.1:5232/",
            "http://[::1]:5232/",
        ] {
            assert_eq!(server_url(input).unwrap().as_str(), input);
        }

        assert!(server_url("http://localhost.example.org/").is_err());
    }

    #[test]
    fn a_scheme_other_than_dav_is_refused_by_name() {
        let err = server_url("imaps://mail.example.org")
            .unwrap_err()
            .to_string();

        assert!(err.contains("Unsupported server scheme `imaps`"), "{err}");
    }

    #[test]
    fn a_url_naming_no_host_is_refused() {
        assert!(server_url("https://").is_err());
        assert!(server_url("caldavs://").is_err());
    }
}
