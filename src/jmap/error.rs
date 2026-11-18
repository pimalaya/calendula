//! # JMAP refusals
//!
//! Renders the per-object errors of a JMAP write (RFC 8620 section 5.3)
//! as prose and codes them, and codes the refusals a JMAP error chain
//! carries, as cardamum's jmap/error.rs does.

use std::fmt;

use anyhow::Error;
use io_jmap::{
    calendars::{
        calendar::set::JmapCalendarSetItemError, calendar_event::set::JmapCalendarEventSetItemError,
    },
    client::JmapClientStdError,
    rfc8620::{error::JmapMethodError, send::JmapSendError},
};

use crate::error::ErrorCode;

/// The parts of a JMAP SetError worth showing.
pub trait JmapSetError {
    /// The error type as JMAP spells it, e.g. `invalidProperties`.
    fn type_name(&self) -> &str;
    /// The optional human-readable detail the server sent along.
    fn description(&self) -> Option<&str>;
    /// The property names at fault, empty outside `invalidProperties`.
    fn properties(&self) -> &[String];
    /// The code of a `notFound`, naming what the set found missing.
    fn not_found(&self) -> ErrorCode;
    /// The code of an `invalidProperties`, none when no caller acts on it.
    fn invalid(&self) -> Option<ErrorCode> {
        None
    }

    /// The code of the error, read from its type: none for a type no
    /// caller acts on, its type kept in the message.
    fn code(&self) -> Option<ErrorCode> {
        match self.type_name() {
            "forbidden" => Some(ErrorCode::PermissionDenied),
            "notFound" => Some(self.not_found()),
            "overQuota" => Some(ErrorCode::QuotaExceeded),
            "rateLimit" => Some(ErrorCode::RateLimited),
            "invalidProperties" => self.invalid(),
            _ => None,
        }
    }
}

/// Renders a SetError as the suffix of a refusal message.
pub fn format_set_error<E: JmapSetError>(err: &E) -> String {
    let mut message = format!(": {}", err.type_name());

    if !err.properties().is_empty() {
        message.push_str(" (`");
        message.push_str(&err.properties().join("`, `"));
        message.push_str("`)");
    }

    if let Some(description) = err.description() {
        message.push_str(": ");
        message.push_str(description.trim_end_matches(['.', '\n']));
    }

    message
}

/// The failure of a write the server refused, `prefix` naming the write,
/// followed by the rendered error and coded by its type.
pub fn set_error<E: JmapSetError>(prefix: &str, err: &E) -> Error {
    JmapSetRefusal {
        message: format!("{prefix}{}", format_set_error(err)),
        code: err.code(),
    }
    .into()
}

/// A write the server refused, worded and coded by its SetError.
#[derive(Debug)]
pub struct JmapSetRefusal {
    message: String,
    code: Option<ErrorCode>,
}

impl fmt::Display for JmapSetRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for JmapSetRefusal {}

/// The code of a JMAP refusal in an error chain: a refused write by its
/// SetError type, a method error `forbidden` and an HTTP 403
/// `permission-denied`, an HTTP 429 `rate-limited`.
pub(crate) fn refusal_code(cause: &(dyn std::error::Error + 'static)) -> Option<ErrorCode> {
    if let Some(err) = cause.downcast_ref::<JmapSetRefusal>() {
        return err.code;
    }

    if let Some(JmapMethodError::Forbidden { .. }) = cause.downcast_ref::<JmapMethodError>() {
        return Some(ErrorCode::PermissionDenied);
    }

    let send = match cause.downcast_ref::<JmapClientStdError>() {
        Some(JmapClientStdError::Send(err)) => err,
        Some(_) => return None,
        None => cause.downcast_ref::<JmapSendError>()?,
    };

    match send {
        JmapSendError::HttpStatus(status) | JmapSendError::Request { status, .. } => match status {
            429 => Some(ErrorCode::RateLimited),
            403 => Some(ErrorCode::PermissionDenied),
            _ => None,
        },
        _ => None,
    }
}

impl JmapSetError for JmapCalendarEventSetItemError {
    fn not_found(&self) -> ErrorCode {
        ErrorCode::ItemNotFound
    }

    fn invalid(&self) -> Option<ErrorCode> {
        Some(ErrorCode::ItemInvalid)
    }

    fn type_name(&self) -> &str {
        match self {
            Self::Forbidden { .. } => "forbidden",
            Self::OverQuota { .. } => "overQuota",
            Self::TooLarge { .. } => "tooLarge",
            Self::RateLimit { .. } => "rateLimit",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Singleton { .. } => "singleton",
            Self::NoSupportedScheduleMethods { .. } => "noSupportedScheduleMethods",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::Forbidden { description }
            | Self::OverQuota { description }
            | Self::TooLarge { description }
            | Self::RateLimit { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. }
            | Self::Singleton { description }
            | Self::NoSupportedScheduleMethods { description }
            | Self::Other { description, .. } => description.as_deref(),
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

impl JmapSetError for JmapCalendarSetItemError {
    fn not_found(&self) -> ErrorCode {
        ErrorCode::CalendarNotFound
    }

    fn type_name(&self) -> &str {
        match self {
            Self::CalendarHasEvent { .. } => "calendarHasEvent",
            Self::Forbidden { .. } => "forbidden",
            Self::NotFound { .. } => "notFound",
            Self::InvalidPatch { .. } => "invalidPatch",
            Self::WillDestroy { .. } => "willDestroy",
            Self::InvalidProperties { .. } => "invalidProperties",
            Self::Other { type_name, .. } if type_name.is_empty() => "unknown",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Self::CalendarHasEvent { description }
            | Self::Forbidden { description }
            | Self::NotFound { description }
            | Self::InvalidPatch { description }
            | Self::WillDestroy { description }
            | Self::InvalidProperties { description, .. }
            | Self::Other { description, .. } => description.as_deref(),
        }
    }

    fn properties(&self) -> &[String] {
        match self {
            Self::InvalidProperties { properties, .. } => properties,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// A SetError reads as JMAP names it, its properties and description
    /// after it, and is coded by its type.
    #[test]
    fn a_set_error_is_worded_and_coded_by_its_type() {
        let err: JmapCalendarEventSetItemError = serde_json::from_value(json!({
            "type": "invalidProperties",
            "description": "unknown property.",
            "properties": ["recurrenceRules", "x"],
        }))
        .unwrap();

        let err = set_error("JMAP CalendarEvent create rejected", &err);
        assert_eq!(
            err.to_string(),
            "JMAP CalendarEvent create rejected: invalidProperties (`recurrenceRules`, `x`): \
             unknown property"
        );
        assert_eq!(crate::error::code(&err), Some(ErrorCode::ItemInvalid));

        let code = |kind: &str| {
            let err: JmapCalendarEventSetItemError =
                serde_json::from_value(json!({"type": kind})).unwrap();
            crate::error::code(&set_error("x", &err))
        };
        assert_eq!(code("forbidden"), Some(ErrorCode::PermissionDenied));
        assert_eq!(code("notFound"), Some(ErrorCode::ItemNotFound));
        assert_eq!(code("overQuota"), Some(ErrorCode::QuotaExceeded));
        assert_eq!(code("rateLimit"), Some(ErrorCode::RateLimited));
        assert_eq!(code("tooLarge"), None);

        let err: JmapCalendarSetItemError =
            serde_json::from_value(json!({"type": "notFound"})).unwrap();
        let err = set_error("JMAP Calendar destroy rejected", &err);
        assert_eq!(err.to_string(), "JMAP Calendar destroy rejected: notFound");
        assert_eq!(crate::error::code(&err), Some(ErrorCode::CalendarNotFound));
    }
}
