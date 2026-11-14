//! # Error codes
//!
//! A stable, machine-readable name for the failures a caller is expected
//! to act on, printed as `code` beside the message under `--json`.
//!
//! The message stays for humans and may be reworded at any time; the
//! code is the contract. A failure with no code prints as before, with
//! no `code` field. The mechanism (`CodedError`, `code_of`, the report)
//! lives in `pimalaya_cli::error`; only the vocabulary is calendula's.
//!
//! A server's refusal takes a code of the vocabulary Himalaya shares,
//! `permission-denied`, `quota-exceeded` and `rate-limited`, read from the
//! typed error of the protocol that answered it by that protocol's
//! `refusal_code` reader: [`code`] walks the chain through them, and
//! [`with_code`] hands the code to the failure object, which reads coded
//! errors alone.

use anyhow::Error;
use pimalaya_cli::error::{CodedError, PartialOutcome, code_of};
use schemars::JsonSchema;
use serde::Serialize;

/// The stable codes, each serialised in kebab-case.
///
/// Declared in every build, whatever the backends compiled in, so the
/// `calendula-error` schema a caller reads does not change with them: a
/// build raises only the codes of its backends.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The backend the account resolves to does not serve a capability
    /// the command reaches, refused before connecting: the caller picks
    /// another backend or leaves the option out.
    CapabilityMissing,
    /// A command would read its input off stdin inside a session, which
    /// reads its commands there: the caller passes the input as text.
    StdinTaken,
    /// The item a command names is not in the calendar: the failure of a
    /// command reading or writing that one item, or an `errors` entry.
    ItemNotFound,
    /// The calendar a command names does not exist: nothing was read or
    /// written.
    CalendarNotFound,
    /// The backend answered for an item calendula could not read: an
    /// `errors` entry, or the failure of a command reading that one item.
    ItemUnreadable,
    /// The server refused the request to the account: an HTTP 403 that
    /// is no rate limit and, on CalDAV, names no precondition but
    /// `need-privileges` (RFC 3744 §7.1.1).
    PermissionDenied,
    /// The write would exceed the account's quota: an HTTP 507, a CalDAV
    /// `quota-not-exceeded` (RFC 4331 §6), Graph's `ErrorQuotaExceeded`.
    QuotaExceeded,
    /// The server throttled the request: an HTTP 429, or a Google rate
    /// limit or quota. calendula retries nothing, so this is the first
    /// one met.
    RateLimited,
    /// The item is listed but its body is not local yet: a sync will
    /// bring it, so the caller waits rather than reports a loss.
    BodyPending,
    /// A write named, by `--if-match`, a version the item no longer has
    /// (an HTTP 412, Graph's `changeKey` read before the write, a pimdir
    /// version that moved): the caller reads it again rather than
    /// overwrites a change.
    PreconditionFailed,
    /// A reply or a cancel named one occurrence, which its performer
    /// cannot act on alone: the caller acts on the whole series or not
    /// at all.
    OccurrenceUnsupported,
    /// A create names a `UID` its calendar already holds (a CalDAV
    /// `no-uid-conflict`, RFC 4791 §5.3.2) or, on pimdir, has queued: the
    /// caller updates that item rather than creates it again.
    ItemDuplicate,
    /// The pimdir store has no schema yet: its sync engine creates it.
    StoreUncreated,
    /// The pimdir store is of another format version than this
    /// calendula's, never migrated: an earlier sync engine's store is
    /// reinstalled by that engine, a newer one's needs a newer calendula.
    StoreVersion,
    /// The pimdir database is not a pimdir store, another program having
    /// written it: the caller points calendula at the store its sync
    /// engine fills.
    StoreForeign,
    /// The pimdir store's two version stamps disagree.
    StoreCorrupt,
    /// The pimdir store names its bodies by a hash this calendula does not
    /// know: the caller upgrades calendula.
    StoreHashAlgo,
    /// Another process holds the pimdir store: the caller retries.
    StoreBusy,
    /// A path of the pimdir store is a link or another user can reach it:
    /// the caller restricts it, a store being its user's alone.
    StoreExposed,
    /// The lock file of the pimdir store was removed or replaced under a
    /// write: nothing was queued, and the caller retries.
    LockReplaced,
    /// A flag set holds a flag the pimdir store cannot: refused before
    /// anything is queued.
    FlagInvalid,
    /// A queue action's payload is one its schema refuses: refused before
    /// anything is queued.
    ActionMalformed,
    /// The pimdir store has no search index yet: its sync engine builds
    /// it.
    IndexMissing,
    /// The pimdir search index was built otherwise or from another store:
    /// its sync engine rebuilds it.
    IndexOutdated,
    /// The pimdir search index is of another draft or shape: its sync
    /// engine rebuilds one of an earlier draft, and leaves any other.
    IndexVersion,
    /// The zone the pimdir search index reads floating times in, or the one
    /// a search reads days in, is not one of the IANA time zone database.
    IndexZoneUnknown,
    /// The SQLite calendula links lacks what the pimdir search index
    /// needs (FTS5, 3.43 or later).
    IndexFts5Missing,
    /// The pimdir search query is malformed or names an unknown field or
    /// value: nothing was searched.
    QueryRefused,
    /// The pimdir search index could not answer an accepted query, an
    /// implementation fault.
    QueryUnanswerable,
}

/// The error of one item a command could not list or act on.
pub type ItemError = pimalaya_cli::error::ItemError<ErrorCode>;

/// The `errors` entry of the item `id` a listing could not read: coded by
/// the refusal `err` carries, else `item-unreadable`.
pub fn unreadable(id: Option<&str>, err: Error) -> ItemError {
    let mut entry = ItemError::new(id, &with_code(err));
    entry.code.get_or_insert(ErrorCode::ItemUnreadable);
    entry
}

/// The end of a command that printed its result with `errors`: a partial
/// outcome, exit 3, when one is listed.
pub fn partial_when(errors: &[ItemError]) -> Result<(), Error> {
    match errors.is_empty() {
        true => Ok(()),
        false => Err(PartialOutcome::default().into()),
    }
}

/// A refusal under `item-not-found`: the item a command names is not in
/// the calendar.
pub fn item_not_found(message: impl Into<String>) -> Error {
    CodedError::new(ErrorCode::ItemNotFound, message.into()).into()
}

/// A refusal under `calendar-not-found`: the calendar a command names does
/// not exist.
#[cfg_attr(
    not(any(feature = "caldav", feature = "pimdir", feature = "vdir")),
    allow(dead_code)
)]
pub fn calendar_not_found(message: impl Into<String>) -> Error {
    CodedError::new(ErrorCode::CalendarNotFound, message.into()).into()
}

/// A failure an item request met, carrying `code` while keeping the text
/// of `err`, which a protocol reader cannot code alone: it does not know
/// which request a refusal answered.
#[cfg_attr(
    not(any(
        feature = "caldav",
        feature = "gcal",
        feature = "msgraph",
        feature = "vdir"
    )),
    allow(dead_code)
)]
pub fn recoded(code: ErrorCode, err: impl std::fmt::Display) -> Error {
    CodedError::new(code, err.to_string()).into()
}

/// The code of `err`: the one a coded error in its chain carries, else
/// the server refusal's under it.
pub fn code(err: &Error) -> Option<ErrorCode> {
    code_of(err).or_else(|| refusal_code(err))
}

/// `err` carrying its [`code`] where the failure object reads it.
///
/// The failure object reads the code of a [`CodedError`] alone, which a
/// server refusal is not: one a reader codes is rebuilt around such an
/// error, each message of its chain kept, so its text and its `sources`
/// print as before. Any other error is returned as it is.
pub fn with_code(err: Error) -> Error {
    if code_of::<ErrorCode>(&err).is_some() {
        return err;
    }

    let Some(code) = refusal_code(&err) else {
        return err;
    };

    let mut messages: Vec<String> = err.chain().map(ToString::to_string).collect();
    let root = messages.pop().unwrap_or_default();
    let root = Error::from(CodedError::new(code, root));

    messages
        .into_iter()
        .rev()
        .fold(root, |err, message| err.context(message))
}

/// A reader of one protocol's typed errors, coding the refusals it knows.
type Refusal = fn(&(dyn std::error::Error + 'static)) -> Option<ErrorCode>;

/// The code of the server refusal `err`'s chain carries, read from the
/// typed error of the protocol that answered it: the first cause a
/// protocol codes.
fn refusal_code(err: &Error) -> Option<ErrorCode> {
    let refusals: &[Refusal] = &[
        #[cfg(feature = "caldav")]
        crate::caldav::client::refusal_code,
        #[cfg(feature = "gcal")]
        crate::gcal::client::refusal_code,
        #[cfg(feature = "msgraph")]
        crate::msgraph::backend::refusal_code,
        #[cfg(feature = "vdir")]
        crate::vdir::client::refusal_code,
    ];

    err.chain()
        .find_map(|cause| refusals.iter().find_map(|refusal| refusal(cause)))
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::error::ErrorReport;
    use serde_json::json;

    use super::*;

    /// A coded error is the failure object's as it is, and an uncoded
    /// one no reader knows stays uncoded.
    #[test]
    fn with_code_leaves_a_coded_or_an_unknown_error_alone() {
        let err = with_code(item_not_found("Gone").context("Read item `a`"));
        assert_eq!(code_of(&err), Some(ErrorCode::ItemNotFound));
        assert_eq!(format!("{err:#}"), "Read item `a`: Gone");

        let err = with_code(anyhow::anyhow!("boom").context("Read item `a`"));
        let json = serde_json::to_value(ErrorReport::<ErrorCode>::from(err)).unwrap();
        assert_eq!(
            json,
            json!({"code": null, "error": "Read item `a`", "sources": ["boom"], "backtrace": null})
        );
    }
}
