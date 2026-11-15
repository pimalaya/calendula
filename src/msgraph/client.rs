//! # Microsoft Graph client
//!
//! calendula's wrapper around [`io_msgraph`]'s std client, building a
//! connected client from a [`MsgraphConfig`] for the backend and the
//! `msgraph` commands alike.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow};
use io_msgraph::v1::client::{
    MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError,
};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;

use crate::{
    cli::resolve_account,
    config::{MsgraphConfig, ProxyConfig, default_msgraph_alpn},
    error::{ErrorCode, recoded},
};

/// A connected Graph client, for the protocol-specific subcommands.
pub struct MsgraphClient {
    inner: MsgraphClientStd,
}

impl MsgraphClient {
    /// Wraps an already-connected client.
    pub fn new(inner: MsgraphClientStd) -> Self {
        Self { inner }
    }
}

impl Deref for MsgraphClient {
    type Target = MsgraphClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MsgraphClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loads the configuration, picks the active account, then connects.
///
/// Bails when the account carries no `[msgraph]` block.
pub fn build_msgraph_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<MsgraphClient> {
    let (_, name, mut account_config) = resolve_account(config_paths, account_name)?;

    let config = account_config
        .msgraph
        .take()
        .ok_or_else(|| anyhow!("Microsoft Graph configuration is missing for account `{name}`"))?;

    let inner = connect(&config, &mut SecretResolver::new())?;
    Ok(MsgraphClient::new(inner))
}

/// Connects to Microsoft Graph with the configured bearer token, resolved
/// through `resolver` so a command another backend of the same account
/// names too is spawned once.
pub fn connect(config: &MsgraphConfig, resolver: &mut SecretResolver) -> Result<MsgraphClientStd> {
    let token = resolver.resolve(config.auth.token.clone())?;
    let alpn = config.alpn.clone().unwrap_or_else(default_msgraph_alpn);
    let options = MsgraphClientStdConnectOptions {
        tls: config.tls.clone().into_tls(alpn),
        proxy: ProxyConfig::resolve(config.proxy.clone(), resolver)?,
        user_id: config.user_id.clone(),
    };

    MsgraphClientStd::connect(token.expose_secret(), options)
        .context("Cannot connect to Microsoft Graph")
}

/// The failure of a request naming one event: `item-not-found` when Graph
/// holds no such event ([`is_missing`]), its message kept.
pub fn event_refusal(err: MsgraphClientStdError) -> anyhow::Error {
    match is_missing(&err) {
        true => recoded(ErrorCode::ItemNotFound, err),
        false => err.into(),
    }
}

/// The failure of a request naming one calendar: `calendar-not-found` when
/// Graph holds no such calendar ([`is_missing`]), its message kept.
pub fn calendar_refusal(err: MsgraphClientStdError) -> anyhow::Error {
    match is_missing(&err) {
        true => recoded(ErrorCode::CalendarNotFound, err),
        false => err.into(),
    }
}

/// Whether a request naming one resource found none: a 404, or the 400
/// Graph answers an id it reads as no id of its own
/// (`ErrorInvalidIdMalformed`).
pub fn is_missing(err: &MsgraphClientStdError) -> bool {
    let MsgraphClientStdError::Send(send) = err else {
        return false;
    };

    send.api().is_some_and(|api| {
        api.status == 404 || (api.status == 400 && api.code == "ErrorInvalidIdMalformed")
    })
}

#[cfg(test)]
mod tests {
    use io_msgraph::v1::client::MsgraphClientStd;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use crate::{
        error::{self, ErrorCode},
        msgraph::{
            calendars::get::MsgraphCalendarGetCommand,
            client::MsgraphClient,
            events::{delete::MsgraphEventDeleteCommand, get::MsgraphEventGetCommand},
        },
        script::{self, Script},
    };

    /// The client over a Graph server answering `status` with an error
    /// envelope of `code`.
    fn refusing(status: u16, code: &str) -> MsgraphClient {
        let body = json!({"error": {"code": code, "message": "Refused"}}).to_string();
        let (script, _) = Script::new([script::response(status, "application/json", &body)]);
        MsgraphClient::new(MsgraphClientStd::new(script, "token", Default::default()))
    }

    /// A command naming an event Graph does not hold carries
    /// `item-not-found`, one naming a calendar `calendar-not-found`, as the
    /// unified commands code them, Graph's message kept; another refusal
    /// keeps its own code.
    #[test]
    fn a_msgraph_command_on_a_missing_resource_is_coded() {
        let printer = &mut CapturePrinter::default();

        let get = MsgraphEventGetCommand {
            event_id: String::from("AAMk"),
        };
        let err = get
            .execute(printer, refusing(404, "ErrorItemNotFound"))
            .unwrap_err();
        assert_eq!(error::code(&err), Some(ErrorCode::ItemNotFound), "{err:#}");
        assert!(format!("{err:#}").contains("Refused"), "{err:#}");

        let delete = |status, code| {
            let delete = MsgraphEventDeleteCommand {
                event_id: String::from("AAMk"),
            };
            let err = delete
                .execute(&mut CapturePrinter::default(), refusing(status, code))
                .unwrap_err();
            error::code(&err)
        };
        assert_eq!(
            delete(400, "ErrorInvalidIdMalformed"),
            Some(ErrorCode::ItemNotFound)
        );
        assert_eq!(
            delete(403, "ErrorAccessDenied"),
            Some(ErrorCode::PermissionDenied)
        );

        let get = MsgraphCalendarGetCommand {
            calendar_id: String::from("AAMk"),
        };
        let err = get
            .execute(printer, refusing(404, "ErrorItemNotFound"))
            .unwrap_err();
        let code = error::code(&err);
        assert_eq!(code, Some(ErrorCode::CalendarNotFound), "{err:#}");
    }
}
