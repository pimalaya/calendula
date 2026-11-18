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
use io_msgraph::v1::{
    client::{MsgraphClientStd, MsgraphClientStdConnectOptions, MsgraphClientStdError},
    send::{MSGRAPH_API_BASE, MsgraphSend},
};
use pimalaya_config::secret::SecretResolver;
use secrecy::ExposeSecret;
use serde_json::Value;
use url::Url;

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

impl MsgraphClient {
    /// The resource at `segments` under the account's owner (`me` or
    /// `users/<id>`), each segment percent-encoded, as Graph sent it.
    pub fn raw_get(&mut self, segments: &[&str]) -> Result<Value> {
        let mut url = Url::parse(MSGRAPH_API_BASE)?;
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|()| anyhow!("Cannot build a Graph URL"))?;
            path.pop_if_empty();
            match self.user_id.as_str() {
                "me" => path.push("me"),
                user => path.push("users").push(user),
            };
            path.extend(segments);
        }

        let auth = self.auth.clone();
        Ok(self.run(MsgraphSend::<Value>::get(&auth, url))?.response)
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

/// `err` coded by `refusal` when it is Graph's answer, else as it is.
pub fn coded(
    err: anyhow::Error,
    refusal: fn(MsgraphClientStdError) -> anyhow::Error,
) -> anyhow::Error {
    match err.downcast::<MsgraphClientStdError>() {
        Ok(err) => refusal(err),
        Err(err) => err,
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
    use clap::Parser;
    use io_msgraph::v1::client::MsgraphClientStd;
    use pimalaya_cli::printer::CapturePrinter;
    use serde_json::json;

    use crate::{
        error::{self, ErrorCode},
        msgraph::{
            calendars::{get::MsgraphCalendarGetCommand, list::MsgraphCalendarListCommand},
            client::MsgraphClient,
            events::{
                delete::MsgraphEventDeleteCommand, delta::MsgraphEventDeltaCommand,
                get::MsgraphEventGetCommand, list::MsgraphEventListCommand,
            },
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

    const NEXT: &str = "https://graph.microsoft.com/v1.0/me/events?$skiptoken=X";
    const DELTA: &str = "https://graph.microsoft.com/v1.0/me/calendarView/delta?$deltatoken=D";

    /// The client over a Graph server answering each of `bodies` 200, and
    /// the requests it receives.
    fn answering(bodies: &[&str]) -> (MsgraphClient, script::Requests) {
        let answers = bodies
            .iter()
            .map(|body| script::response(200, "application/json", body));
        let (script, requests) = Script::new(answers);
        let client = MsgraphClient::new(MsgraphClientStd::new(script, "token", Default::default()));
        (client, requests)
    }

    /// The request line of the `n`th request.
    fn request_line(requests: &script::Requests, n: usize) -> String {
        let requests = requests.lock().unwrap();
        let request = String::from_utf8_lossy(&requests[n]).into_owned();
        request.lines().next().unwrap_or_default().to_owned()
    }

    /// A Graph listing prints Graph's `@odata.nextLink` as `nextPage`,
    /// never beside it, and `--page-token` takes it back as is; the last
    /// page prints `nextPage` `null` (pimalaya-cli output.md, *Paging
    /// resumes*).
    #[test]
    fn a_graph_listing_pages_by_next_page() {
        let first = format!(r#"{{"value":[{{"id":"E1"}}],"@odata.nextLink":"{NEXT}"}}"#);
        let last = r#"{"value":[{"id":"E2"}]}"#;

        let (client, _) = answering(&[&first]);
        let mut printer = CapturePrinter::default();
        let list = MsgraphEventListCommand::try_parse_from(["list"]).unwrap();
        list.execute(&mut printer, client).unwrap();
        let json = printer.into_single().unwrap();
        assert_eq!(json["nextPage"], NEXT, "{json}");
        assert!(json.get("@odata.nextLink").is_none(), "{json}");

        let (client, requests) = answering(&[last]);
        let mut printer = CapturePrinter::default();
        let list = MsgraphEventListCommand::try_parse_from(["list", "--page-token", NEXT]).unwrap();
        list.execute(&mut printer, client).unwrap();
        let json = printer.into_single().unwrap();
        assert_eq!(json["nextPage"], json!(null), "{json}");
        assert!(request_line(&requests, 0).contains("skiptoken=X"));

        // NOTE: the link carries the calendar and the query.
        let both = ["list", "--page-token", NEXT, "-k", "cal"];
        assert!(MsgraphEventListCommand::try_parse_from(both).is_err());

        let calendars = first.replace("E1", "C1");
        let (client, _) = answering(&[&calendars, last]);
        let mut printer = CapturePrinter::default();
        let list = MsgraphCalendarListCommand::try_parse_from(["list"]).unwrap();
        list.execute(&mut printer, client).unwrap();
        let json = printer.into_single().unwrap();
        assert_eq!(json["nextPage"], NEXT, "{json}");
        assert!(json.get("@odata.nextLink").is_none(), "{json}");
        assert_eq!(json["calendars"][0]["id"], "C1", "{json}");
    }

    /// A delta page prints its `@odata.nextLink` as `nextPage`, resumed
    /// through `--page-token`, and keeps the round's `@odata.deltaLink`
    /// raw on its last page.
    #[test]
    fn a_graph_delta_pages_by_next_page_and_keeps_its_delta_link() {
        let first = format!(r#"{{"value":[{{"id":"E1"}}],"@odata.nextLink":"{NEXT}"}}"#);
        let last = format!(r#"{{"value":[{{"id":"E2"}}],"@odata.deltaLink":"{DELTA}"}}"#);

        let args = ["delta", "--from", "2026-11-01", "--to", "2026-11-30"];
        let (client, _) = answering(&[&first]);
        let mut printer = CapturePrinter::default();
        let delta = MsgraphEventDeltaCommand::try_parse_from(args).unwrap();
        delta.execute(&mut printer, client).unwrap();
        let json = printer.into_single().unwrap();
        assert_eq!(json["nextPage"], NEXT, "{json}");
        assert!(json.get("@odata.nextLink").is_none(), "{json}");
        assert!(json.get("@odata.deltaLink").is_none(), "{json}");

        let (client, requests) = answering(&[&last]);
        let mut printer = CapturePrinter::default();
        let args = ["delta", "--page-token", NEXT];
        let delta = MsgraphEventDeltaCommand::try_parse_from(args).unwrap();
        delta.execute(&mut printer, client).unwrap();
        let json = printer.into_single().unwrap();
        assert_eq!(json["nextPage"], json!(null), "{json}");
        assert_eq!(json["@odata.deltaLink"], DELTA, "{json}");
        assert!(request_line(&requests, 0).contains("skiptoken=X"));
    }
}
