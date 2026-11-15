//! # Raw JSON
//!
//! The output type of the protocol `request` passthrough, printing the
//! payload a backend answered as is.

use core::fmt;

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

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
