//! # JMAP client
//!
//! Connects io-jmap's std client from a [`JmapConfig`] and opens its
//! session, refused when the session serves no calendars.

use anyhow::{Result, bail};
use base64_codec::codec::Base64Codec;
use io_jmap::{
    calendars::JMAP_CALENDARS_CAPABILITY,
    client::{JmapClientStd, JmapClientStdConnectOptions},
    rfc8620::session::JmapSession,
};
use pimalaya_config::secret::SecretResolver;
use secrecy::{ExposeSecret, SecretString};

use crate::config::{JmapAuthConfig, JmapConfig, ProxyConfig, parse_server};

/// Connects to the JMAP server and opens its session, a bare server name
/// discovered through `/.well-known/jmap`.
///
/// The credential goes through `resolver`, so an account naming one
/// credential command from several of its backends spawns it once.
pub fn connect(config: JmapConfig, resolver: &mut SecretResolver) -> Result<JmapClientStd> {
    let http_auth = http_auth(config.auth, resolver)?;
    let url = parse_server(&config.server, "https", &["http", "https", "jmap", "jmaps"])?;
    let options = JmapClientStdConnectOptions {
        tls: config.tls.into_tls(config.alpn),
        proxy: ProxyConfig::resolve(config.proxy, resolver)?,
    };

    let mut client = JmapClientStd::connect(&url, http_auth, options)?;
    let session = client.session_get(&url)?;
    check_session(session)?;

    Ok(client)
}

/// Refuses a session that does not advertise JMAP for Calendars, or names
/// no account for it, rather than let every call fail later.
pub fn check_session(session: &JmapSession) -> Result<()> {
    let advertised = session.capabilities.contains_key(JMAP_CALENDARS_CAPABILITY);

    if !advertised || session.account_for(JMAP_CALENDARS_CAPABILITY).is_err() {
        bail!(
            "The JMAP session does not advertise `{JMAP_CALENDARS_CAPABILITY}` with an account \
             for it: this server or account serves no calendars"
        );
    }

    Ok(())
}

/// Renders a [`JmapAuthConfig`] as the `Authorization` header value.
pub fn http_auth(config: JmapAuthConfig, resolver: &mut SecretResolver) -> Result<SecretString> {
    match config {
        JmapAuthConfig::Header(header) => Ok(resolver.resolve(header)?),
        JmapAuthConfig::Bearer { token } => {
            let token = resolver.resolve(token)?;
            Ok(format!("Bearer {}", token.expose_secret()).into())
        }
        JmapAuthConfig::Basic { username, password } => {
            let password = resolver.resolve(password)?;
            let creds = format!("{}:{}", username, password.expose_secret());
            let mut encoded = Vec::new();
            Base64Codec::default().encode_vec(creds.as_bytes(), &mut encoded);
            let encoded = String::from_utf8(encoded).unwrap_or_default();
            Ok(format!("Basic {encoded}").into())
        }
    }
}

#[cfg(test)]
mod tests {
    use pimalaya_config::secret::Secret;
    use serde_json::json;

    use super::*;

    /// A session without the calendars capability, or naming no account
    /// for it, is refused naming the capability.
    #[test]
    fn a_session_without_calendars_is_refused_naming_the_capability() {
        let session = |capabilities: serde_json::Value, primary: serde_json::Value| {
            serde_json::from_value::<JmapSession>(json!({
                "username": "", "accounts": {}, "capabilities": capabilities,
                "primaryAccounts": primary, "apiUrl": "https://jmap.example.org/api/",
                "downloadUrl": "", "uploadUrl": "", "eventSourceUrl": "", "state": "",
            }))
            .unwrap()
        };
        let calendars = JMAP_CALENDARS_CAPABILITY;

        let contacts = session(
            json!({"urn:ietf:params:jmap:core": {}, "urn:ietf:params:jmap:contacts": {}}),
            json!({"urn:ietf:params:jmap:contacts": "a1"}),
        );
        let unaccounted = session(json!({ calendars: {} }), json!({}));

        for session in [contacts, unaccounted] {
            let err = check_session(&session).unwrap_err().to_string();
            assert!(err.contains(&format!("`{calendars}`")), "{err}");
        }

        let served = session(json!({ calendars: {} }), json!({ calendars: "a1" }));
        check_session(&served).unwrap();
    }

    #[test]
    fn basic_auth_is_standard_padded_base64() {
        let auth = JmapAuthConfig::Basic {
            username: String::from("user"),
            password: Secret::Raw(SecretString::from("pa\u{e9}ss")),
        };
        let header = http_auth(auth, &mut SecretResolver::new()).unwrap();
        assert_eq!(header.expose_secret(), "Basic dXNlcjpwYcOpc3M=");
    }
}
