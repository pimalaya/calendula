//! # Sign-in
//!
//! Signs the picked [`Route`] in and tests it as soon as it is typed. A
//! password or an API token is typed once, after the endpoints it goes to
//! are named, then stored by the wizard once the test passed. An OAuth
//! sign-in is handed off to Ortie, or to a broker command when Ortie is not
//! installed.
//!
//! A refusal offers to try again, to change the login (a password only), or
//! to go back to the review.

use std::fmt;

use anyhow::{Error, Result};
use pimalaya_cli::{
    prompt,
    spinner::Spinner,
    wizard::{
        keyring::{self, SecretChoice},
        ortie::{self, OrtieSignInError},
    },
};
use pimalaya_config::secret::Secret;

#[cfg(feature = "gcal")]
use crate::config::{GcalAuthConfig, GcalConfig};
#[cfg(feature = "msgraph")]
use crate::config::{MsgraphAuthConfig, MsgraphConfig};
use crate::{
    account::check::{all_ok, check_account},
    backend::Backend,
    config::{AccountConfig, CaldavAuthConfig, CaldavConfig},
    wizard::{
        search::{Protocol, Route, SignIn},
        secret,
    },
};

/// Where Ortie, the Pimalaya token broker, is found.
const ORTIE_URL: &str = "https://github.com/pimalaya/ortie";

/// Who signs in: the account name and what the first prompt told of the
/// user.
pub struct Identity {
    /// The account name, from which the Ortie account and the keyring
    /// entry derive.
    pub name: String,
    /// The typed address, when a full one was typed.
    pub address: Option<String>,
    /// The typed address's domain, or the typed bare domain.
    pub domain: Option<String>,
}

impl Identity {
    /// Signs `route` in until its test passes, answering the account, or
    /// `None` when the user goes back to the review.
    pub fn sign_in(&self, route: &Route) -> Result<Option<AccountConfig>> {
        match route.sign_in {
            SignIn::Password => {
                let login = match self.login(route) {
                    Some(login) => login,
                    None => prompt::text::<&str>("Login:", None)?,
                };

                self.sign_in_typed(route, "Password", Some(login))
            }
            SignIn::Token => self.sign_in_typed(route, "API token", None),
            SignIn::Oauth => self.sign_in_oauth(route),
        }
    }

    /// The login: the advertised username when it is an address, else the
    /// typed address. `None` when neither is known, the one case it is asked.
    fn login(&self, route: &Route) -> Option<String> {
        route
            .username
            .clone()
            .filter(|username| is_address(username))
            .or_else(|| self.address.clone())
    }

    /// The keyring entry a typed secret is stored at.
    fn key(&self, route: &Route) -> String {
        format!("calendula-{}-{}", self.name, route.protocol.backend())
    }

    /// Signs a password, when a `login` goes with it, or an API token in:
    /// typed once, tested, then stored where the user picks.
    fn sign_in_typed(
        &self,
        route: &Route,
        label: &str,
        mut login: Option<String>,
    ) -> Result<Option<AccountConfig>> {
        let endpoint = route.server.to_string();

        loop {
            let value = keyring::prompt_password(label, &[&endpoint], self.domain.as_deref())?;
            let typed = Secret::Raw(value.clone());

            if passes(&account(route, login.as_deref(), typed)) {
                let choice = keyring::prompt_store(label, &self.key(route), &value)?;
                let stored = secret::to_secret(choice)?;
                return Ok(Some(account(route, login.as_deref(), stored)));
            }

            match Refusal::prompt(login.is_some())? {
                Refusal::Retry => continue,
                Refusal::Login => login = Some(prompt::text("Login:", login.as_deref())?),
                Refusal::Back => return Ok(None),
            }
        }
    }

    fn sign_in_oauth(&self, route: &Route) -> Result<Option<AccountConfig>> {
        let input = self
            .address
            .as_deref()
            .or(self.domain.as_deref())
            .unwrap_or_default();
        let name = format!("calendula-{}", self.name);
        let services = [route.protocol.service()];

        loop {
            let token = match ortie::sign_in(input, &name, &services) {
                Ok(argv) => Some(secret::to_secret(SecretChoice::Command(argv))?),
                Err(OrtieSignInError::NotInstalled) => {
                    eprintln!("A token broker issues this token, such as Ortie: {ORTIE_URL}");
                    let label = format!("{} token", route.protocol.name());
                    Some(secret::configure_token(&label, &self.key(route), true)?)
                }
                Err(err) => {
                    eprintln!("{:#}", Error::from(err));
                    None
                }
            };

            if let Some(token) = token {
                let account = account(route, None, token);

                if passes(&account) {
                    return Ok(Some(account));
                }
            }

            if let Refusal::Back = Refusal::prompt(false)? {
                return Ok(None);
            }
        }
    }
}

/// What to do after a refused sign-in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Refusal {
    Retry,
    Login,
    Back,
}

impl Refusal {
    /// Asks what to do, offering to change the login when `login` is set.
    fn prompt(login: bool) -> Result<Self> {
        let mut refusals = vec![Self::Retry];

        if login {
            refusals.push(Self::Login);
        }

        refusals.push(Self::Back);

        Ok(prompt::item("What next?", refusals, None)?)
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retry => f.write_str("Try again"),
            Self::Login => f.write_str("Change the login"),
            Self::Back => f.write_str("Back to the review"),
        }
    }
}

/// The account `route` configures, signing in with `secret`: a password
/// when a `login` goes with it, else a bearer token.
fn account(route: &Route, login: Option<&str>, secret: Secret) -> AccountConfig {
    match route.protocol {
        Protocol::Caldav => {
            let auth = match login {
                Some(username) => CaldavAuthConfig::Basic {
                    username: username.to_owned(),
                    password: secret,
                },
                None => CaldavAuthConfig::Bearer { token: secret },
            };

            AccountConfig {
                caldav: Some(CaldavConfig {
                    discover: None,
                    server: Some(route.server.to_string()),
                    home: None,
                    tls: Default::default(),
                    auth,
                }),
                ..Default::default()
            }
        }
        #[cfg(feature = "gcal")]
        Protocol::Gcal => AccountConfig {
            gcal: Some(GcalConfig {
                tls: Default::default(),
                alpn: None,
                auth: GcalAuthConfig { token: secret },
            }),
            ..Default::default()
        },
        #[cfg(feature = "msgraph")]
        Protocol::Msgraph => AccountConfig {
            msgraph: Some(MsgraphConfig {
                user_id: String::from("me"),
                tls: Default::default(),
                alpn: None,
                auth: MsgraphAuthConfig { token: secret },
            }),
            ..Default::default()
        },
    }
}

/// Tests `account` with the check `account check` runs, under a spinner,
/// naming on stderr each backend that failed and why.
fn passes(account: &AccountConfig) -> bool {
    let spinner = Spinner::start("Testing account configuration");
    let checks = check_account(account, Backend::Auto);

    if all_ok(&checks) {
        spinner.success("Account configuration is valid");
        return true;
    }

    spinner.failure("Account configuration test failed");

    for check in &checks {
        if let Some(err) = &check.error {
            eprintln!("{}: {err}", check.backend);
        }
    }

    false
}

/// Whether a value is a full `local@domain` address, both parts non-empty.
pub fn is_address(value: &str) -> bool {
    value
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && !domain.is_empty())
}

#[cfg(test)]
mod tests {
    use pimalaya_config::command::CommandConfig;

    use crate::wizard::sign_in::*;

    fn route(protocol: Protocol, username: Option<&str>) -> Route {
        Route {
            protocol,
            server: "https://dav.example.org/dav/".parse().unwrap(),
            username: username.map(String::from),
            sign_in: SignIn::Password,
        }
    }

    fn identity(address: Option<&str>) -> Identity {
        Identity {
            name: String::from("example"),
            address: address.map(String::from),
            domain: Some(String::from("example.org")),
        }
    }

    fn ortie() -> Secret {
        Secret::Command(CommandConfig::Argv {
            program: String::from("ortie"),
            args: ["token", "show", "-a", "calendula-example"]
                .map(String::from)
                .to_vec(),
        })
    }

    #[test]
    fn an_advertised_address_wins_the_login_over_the_typed_one() {
        let typed = identity(Some("jane@example.org"));

        let advertised = route(Protocol::Caldav, Some("jane.doe@example.org"));
        assert_eq!(
            typed.login(&advertised).as_deref(),
            Some("jane.doe@example.org")
        );

        let bare = route(Protocol::Caldav, Some("jane"));
        assert_eq!(typed.login(&bare).as_deref(), Some("jane@example.org"));
    }

    #[test]
    fn without_any_address_the_login_is_asked() {
        let domain = identity(None);

        assert_eq!(domain.login(&route(Protocol::Caldav, Some("jane"))), None);
        assert_eq!(
            domain.login(&route(Protocol::Caldav, Some("@example.org"))),
            None
        );
        assert_eq!(domain.login(&route(Protocol::Caldav, None)), None);
    }

    #[test]
    fn a_typed_secret_is_stored_under_the_app_account_and_backend() {
        assert_eq!(
            identity(None).key(&route(Protocol::Caldav, None)),
            "calendula-example-caldav"
        );
    }

    #[test]
    fn a_caldav_route_writes_its_server_and_the_typed_sign_in() {
        let pass = Secret::Command(CommandConfig::Argv {
            program: String::from("pass"),
            args: vec![
                String::from("show"),
                String::from("calendula-example-caldav"),
            ],
        });

        let password = account(
            &route(Protocol::Caldav, None),
            Some("jane@example.org"),
            pass,
        )
        .render("example")
        .unwrap();

        assert_eq!(
            password,
            "[accounts.example]\n\
             caldav.server = \"https://dav.example.org/dav/\"\n\
             caldav.auth.basic.password.command = [\"pass\", \"show\", \"calendula-example-caldav\"]\n\
             caldav.auth.basic.username = \"jane@example.org\"\n"
        );

        let token = account(&route(Protocol::Caldav, None), None, ortie())
            .render("example")
            .unwrap();

        assert_eq!(
            token,
            "[accounts.example]\n\
             caldav.server = \"https://dav.example.org/dav/\"\n\
             caldav.auth.bearer.token.command = [\"ortie\", \"token\", \"show\", \"-a\", \"calendula-example\"]\n"
        );
    }

    #[cfg(feature = "gcal")]
    #[test]
    fn a_gcal_route_writes_the_token_alone() {
        let document = account(&route(Protocol::Gcal, None), None, ortie())
            .render("example")
            .unwrap();

        assert_eq!(
            document,
            "[accounts.example]\n\
             gcal.auth.token.command = [\"ortie\", \"token\", \"show\", \"-a\", \"calendula-example\"]\n"
        );
    }

    #[cfg(feature = "msgraph")]
    #[test]
    fn a_msgraph_route_writes_the_token_and_leaves_the_user_to_its_default() {
        let document = account(&route(Protocol::Msgraph, None), None, ortie())
            .render("example")
            .unwrap();

        assert_eq!(
            document,
            "[accounts.example]\n\
             msgraph.auth.token.command = [\"ortie\", \"token\", \"show\", \"-a\", \"calendula-example\"]\n"
        );
    }
}
