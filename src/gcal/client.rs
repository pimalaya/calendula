//! # Google Calendar client
//!
//! calendula's wrapper around [`io_gcal`]'s std client, building a connected
//! client from a [`GcalConfig`].
//!
//! Resolves the bearer token, running its command when it is one, then opens
//! the TLS connection through pimalaya-stream and the proxy the account
//! names. There is no discovery step: the Calendar API lives at one
//! well-known base URL.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::Duration,
};

use anyhow::{Result, anyhow};
use io_gcal::v3::{
    client::{GcalClientStd, GcalClientStdError},
    query::append_query_pairs,
    send::{GCAL_API_BASE, GcalApiError, GcalSend, GcalSendError, gcal_api_url},
};
use pimalaya_config::secret::SecretResolver;
use pimalaya_stream::{
    stream::{Stream, TlsConnectOptions},
    tls::Tls,
};
use secrecy::ExposeSecret;
use serde::Serialize;
use serde_json::{Map, Value};
use url::Url;

use crate::{
    account::context::Account,
    cli::resolve_account,
    config::{GcalConfig, ProxyConfig, default_gcal_alpn},
    error::{ErrorCode, recoded},
};

/// How long a read waits for the Calendar API, as io-gcal's own connect
/// waits.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// A connected client bundled with the merged runtime [`Account`].
pub struct GcalClient {
    inner: GcalClientStd,
    /// The name of the account the client runs for.
    pub name: String,
    /// The account the protocol-specific subcommands take defaults from.
    pub account: Account,
}

impl GcalClient {
    /// Wraps an already-connected client.
    pub fn new(inner: GcalClientStd, name: String, account: Account) -> Self {
        Self {
            inner,
            name,
            account,
        }
    }
}

impl GcalClient {
    /// The resource at `segments` under the Calendar API, as Google sent
    /// it.
    pub fn raw_get(&mut self, segments: &[&str]) -> Result<Value> {
        let url = gcal_api_url(segments)?;
        let send = GcalSend::<Value>::get(&self.inner.auth, url);
        Ok(self.inner.run(send)?.response)
    }

    /// What the Calendar API answers a `POST` of `body` to `segments`, as
    /// Google sent it.
    pub fn raw_post<B: Serialize>(&mut self, segments: &[&str], body: &B) -> Result<Value> {
        let url = gcal_api_url(segments)?;
        let send = GcalSend::<Value>::post_json(&self.inner.auth, url, body)?;
        Ok(self.inner.run(send)?.response)
    }

    /// Every item of the listing at `segments` under the Calendar API,
    /// queried by `params`, each as Google sent it: the pages followed
    /// through `nextPageToken`, the items of each read from `items`, a
    /// page leaving the key out holding none.
    pub fn raw_items<P: Serialize>(&mut self, segments: &[&str], params: &P) -> Result<Vec<Value>> {
        let mut base = gcal_api_url(segments)?;
        append_query_pairs(&mut base, params);

        let mut items = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut url = base.clone();

            if let Some(token) = &page_token {
                url.query_pairs_mut().append_pair("pageToken", token);
            }

            let send = GcalSend::<Map<String, Value>>::get(&self.inner.auth, url);
            let mut page = self.inner.run(send)?.response;

            if let Some(Value::Array(page_items)) = page.remove("items") {
                items.extend(page_items);
            }

            match page.remove("nextPageToken") {
                Some(Value::String(next)) => page_token = Some(next),
                _ => return Ok(items),
            }
        }
    }
}

impl Deref for GcalClient {
    type Target = GcalClientStd;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GcalClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Loads the configuration, picks the active account, then connects.
///
/// Bails when the account carries no `[gcal]` block.
pub fn build_gcal_client(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
) -> Result<GcalClient> {
    let (config, name, mut account_config) = resolve_account(config_paths, account_name)?;

    let gcal_config = account_config
        .gcal
        .take()
        .ok_or_else(|| anyhow!("Google Calendar configuration is missing for account `{name}`"))?;

    let account = Account::from(config).merge(Account::from(account_config));
    let inner = connect(&gcal_config, &mut SecretResolver::new())?;

    Ok(GcalClient::new(inner, name, account))
}

/// Opens a connected Google Calendar client, resolving the bearer token
/// through `resolver` so a command another backend of the same account
/// names too is spawned once.
pub fn connect(config: &GcalConfig, resolver: &mut SecretResolver) -> Result<GcalClientStd> {
    let token = resolver.resolve(config.auth.token.clone())?;
    let proxy = ProxyConfig::resolve(config.proxy.clone(), resolver)?;

    // NOTE: io-gcal's own connect takes no proxy, so the stream is opened
    // here as it opens it, through the proxy the account names.
    let url = Url::parse(GCAL_API_BASE)?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("The Calendar API base `{url}` has no host"))?;
    let options = TlsConnectOptions {
        tls: build_tls(config),
        proxy,
        ..Default::default()
    };

    let stream = Stream::connect_tls(host, url.port().unwrap_or(443), options)?;
    stream.set_read_timeout(Some(READ_TIMEOUT))?;

    Ok(GcalClientStd::new(stream, token.expose_secret()))
}

/// The TLS profile the backend connects with.
///
/// The account's `gcal.alpn` wins, else [`default_gcal_alpn`] pins the
/// HTTP/1.1 io-http is the only client for.
fn build_tls(config: &GcalConfig) -> Tls {
    let alpn = config.alpn.clone().unwrap_or_else(default_gcal_alpn);
    config.tls.clone().into_tls(alpn)
}

/// The code of a Calendar refusal in an error chain: [`google_code`]'s,
/// else a 412 `precondition-failed`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    let GcalClientStdError::Send(GcalSendError::Api(err)) =
        cause.downcast_ref::<GcalClientStdError>()?
    else {
        return None;
    };

    match err.status {
        412 => Some(ErrorCode::PreconditionFailed),
        _ => google_code(err),
    }
}

/// The failure of a request naming one event: a 404 or a 410 is
/// `item-not-found`, the event gone or never there.
pub fn item_refusal(err: GcalClientStdError) -> anyhow::Error {
    match &err {
        GcalClientStdError::Send(send) if matches!(send.status(), Some(404 | 410)) => {
            recoded(ErrorCode::ItemNotFound, err)
        }
        _ => err.into(),
    }
}

/// The failure of an event write (`events.insert`, `events.import`,
/// `events.update`) answered 400, coded `item-invalid`: calendula builds
/// every parameter of that request, so a 400 refuses the event it sends,
/// a field missing or a value Google does not take. The answer is handed
/// back otherwise.
pub fn invalid_event(err: GcalClientStdError) -> Result<anyhow::Error, GcalClientStdError> {
    match &err {
        GcalClientStdError::Send(send) if send.status() == Some(400) => {
            Ok(recoded(ErrorCode::ItemInvalid, err))
        }
        _ => Err(err),
    }
}

/// The code of a Google refusal: a 429, or a 403 naming a rate limit or a
/// quota ([`throttles`]), is `rate-limited`; any other 403 is
/// `permission-denied`.
///
/// Neverest's (neverest/src/throttle.rs), which reads it after its waits,
/// calendula taking none.
fn google_code(err: &GcalApiError) -> Option<ErrorCode> {
    match err.status {
        429 => Some(ErrorCode::RateLimited),
        403 if throttles(err) => Some(ErrorCode::RateLimited),
        403 => Some(ErrorCode::PermissionDenied),
        _ => None,
    }
}

/// Whether a 403 throttles, the part of neverest's `google_refusal` a 403
/// reaches: a reason or a wording naming a rate limit or a quota, per
/// minute or per day, a missing credential's daily limit aside.
fn throttles(err: &GcalApiError) -> bool {
    let has_reason = |reason: &str| {
        err.reasons
            .iter()
            .chain(&err.detail_reasons)
            .any(|r| r == reason)
    };

    let mut text = err.message.clone();
    for reason in err.reasons.iter().chain(&err.detail_reasons) {
        text.push(' ');
        text.push_str(reason);
    }
    let squeezed = squeeze(&text);

    let rate = err.google_status.as_deref() == Some("RESOURCE_EXHAUSTED")
        || [
            "rateLimitExceeded",
            "userRateLimitExceeded",
            "quotaExceeded",
            "RATE_LIMIT_EXCEEDED",
        ]
        .iter()
        .any(|reason| has_reason(reason))
        || squeezed.contains("ratelimitexceeded")
        || squeezed.contains("quotaexceeded");

    let daily = has_reason("dailyLimitExceeded")
        || ((squeezed.contains("dailylimit") || squeezed.contains("perday"))
            && !squeezed.contains("unauthenticated"));
    let minute = squeezed.contains("perminute") || squeezed.contains("per100seconds");

    daily || minute || rate
}

/// The ASCII letters and digits of a text, lowercased.
fn squeeze(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The refusal of a Calendar API error in a chain, for the `gcal`
/// commands: the status, the first reason and the message Google
/// answered.
pub(crate) fn refusal(
    cause: &(dyn std::error::Error + 'static),
) -> Option<crate::shared::refusal::Refusal> {
    let GcalClientStdError::Send(GcalSendError::Api(err)) =
        cause.downcast_ref::<GcalClientStdError>()?
    else {
        return None;
    };

    Some(crate::shared::refusal::Refusal {
        status: Some(err.status),
        reason: err.reasons.first().cloned(),
        message: err.message.clone(),
        code: None,
        retry_after: err.retry_after.map(|delay| delay.as_secs()),
    })
}
