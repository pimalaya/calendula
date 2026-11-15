//! # Raw JSON
//!
//! The output type of the protocol `request` passthrough, printing the
//! payload a backend answered as is, and the URL such a request goes to.

use core::fmt;

use anyhow::{Result, bail};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use url::Url;

/// The URL a raw request to the API at `base` goes to: `path` joined onto
/// `base`, or a full URL on the origin of `base`, `provider` naming the API.
///
/// A full URL on another origin is refused before anything is sent, so
/// the bearer token never leaves for another host.
pub fn request_url(base: &str, path: &str, provider: &str) -> Result<Url> {
    let base = Url::parse(base)?;

    if !path.starts_with("http") {
        return Ok(base.join(path.trim_start_matches('/'))?);
    }

    let url = Url::parse(path)?;

    if url.origin() != base.origin() {
        bail!(
            "Cannot send a {provider} request to `{}`: a full URL must be on `{}`, so the \
             token never leaves for another host",
            url.origin().ascii_serialization(),
            base.origin().ascii_serialization(),
        );
    }

    Ok(url)
}

/// A raw JSON payload: pretty on stdout, verbatim under `response` with
/// `--json`.
///
/// Wrapped rather than transparent, so the payload is a JSON object
/// whatever the backend answered: an array, or null for a 204.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RawJsonOutput {
    /// The response the backend answered, null when it carried no body.
    pub response: Value,
}

impl fmt::Display for RawJsonOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: a 204 carries no body, which parses as null. Printing
        // "null" would read as a value the server sent; it sent nothing.
        if self.response.is_null() {
            return Ok(());
        }

        let pretty = serde_json::to_string_pretty(&self.response).map_err(|_| fmt::Error)?;
        writeln!(f, "{pretty}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// A full URL off the API's origin is refused, so the token stays
    /// with the provider, where it was sent wherever the URL named.
    #[test]
    fn a_full_url_off_the_api_origin_is_refused() {
        let graph = "https://graph.microsoft.com/v1.0/";
        let google = "https://www.googleapis.com/calendar/v3/";

        let joined = request_url(graph, "/me/calendars", "Microsoft Graph").unwrap();
        assert_eq!(
            joined.as_str(),
            "https://graph.microsoft.com/v1.0/me/calendars"
        );
        let full = "https://graph.microsoft.com/v1.0/me/events?%24skip=10";
        assert_eq!(
            request_url(graph, full, "Microsoft Graph")
                .unwrap()
                .as_str(),
            full
        );

        for (base, url) in [
            (graph, "https://evil.example.org/v1.0/me"),
            (graph, "http://graph.microsoft.com/v1.0/me"),
            (graph, "https://graph.microsoft.com:8443/v1.0/me"),
            (google, "https://graph.microsoft.com/v1.0/me"),
            (
                google,
                "https://www.googleapis.com.evil.example.org/calendar/v3/colors",
            ),
        ] {
            let err = request_url(base, url, "API").unwrap_err().to_string();
            assert!(
                err.contains("never leaves for another host"),
                "{url}: {err}"
            );
        }
    }

    #[test]
    fn an_empty_response_prints_nothing() {
        assert_eq!(
            RawJsonOutput {
                response: Value::Null
            }
            .to_string(),
            ""
        );
        assert_eq!(
            RawJsonOutput {
                response: json!({"id": "x"})
            }
            .to_string(),
            "{\n  \"id\": \"x\"\n}\n"
        );
    }
}
