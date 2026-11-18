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
//! `refusal_code` reader, and an account pimalaya-config could not take
//! one of the `account-*` codes, read from its `AccountError`: [`code`]
//! walks the chain through them, and pimalaya-cli's `with_code` hands the
//! code to the failure object, which reads coded errors alone.

use anyhow::Error;
use pimalaya_cli::error::{CodedError, PartialOutcome, code_of, with_code};
use pimalaya_config::toml::AccountError;
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
    /// No account of the configuration has the name `-a` gives, `default`
    /// included: the caller names one the configuration holds, or omits
    /// `-a` to take the account marked default.
    AccountMissing,
    /// No `-a` names an account and none is marked `default = true`: the
    /// caller names one or marks one.
    AccountDefaultMissing,
    /// No `-a` names an account and several are marked `default = true`,
    /// the configuration layers merged: the caller unmarks all but one.
    AccountDefaultSeveral,
    /// The account's backend is one this build leaves out, its block
    /// loaded unread: the caller rebuilds with its cargo feature, or picks
    /// another backend of the account with `--backend`.
    BackendNotBuilt,
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
    /// A create names a calendar that already exists: a vdir directory of
    /// that name, a CalDAV collection at that URL, a Graph calendar of that
    /// name. Nothing was written.
    CalendarDuplicate,
    /// A command naming no calendar found no `calendar.default` either:
    /// the caller names one with `-k` or sets the key. Nothing was read.
    CalendarDefaultMissing,
    /// The backend answered for an item calendula could not read: an
    /// `errors` entry, or the failure of a command reading that one item.
    ItemUnreadable,
    /// The server refused the item written as no iCalendar it takes (a
    /// CalDAV `valid-calendar-data`, `supported-calendar-data`,
    /// `valid-calendar-object-resource` or `supported-calendar-component`
    /// precondition, a refusal naming the iCalendar, a 400 answering a
    /// Google or Graph event write, a JMAP `invalidProperties`): nothing
    /// was written, and the caller fixes the item rather than its rights.
    ItemInvalid,
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
    /// A reply or a cancel named an event that invites nobody it could
    /// address: a reply one naming no `ORGANIZER` to answer or no
    /// `ATTENDEE` to answer as, a cancel one naming no `ATTENDEE` to
    /// notify. Nothing was queued.
    InvitationMissing,
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
    /// The pimdir store holds no schema yet and the verb creates none: its
    /// sync engine creates it.
    StoreNotCurrent,
    /// The pimdir store names its bodies by a hash this calendula does not
    /// know: the caller upgrades calendula.
    StoreHashAlgo,
    /// The pimdir store's sync state was written by another version of
    /// its sync engine.
    SyncVersion,
    /// Another process holds the pimdir store: the caller retries.
    StoreBusy,
    /// A sync owns the pimdir store: the caller retries once it ends.
    StoreOwned,
    /// A producer is staging a body in the pimdir store: the caller
    /// retries.
    StoreStaging,
    /// A pimdir store verb is between two of its chunks: the caller
    /// retries.
    StoreInFlight,
    /// A path of the pimdir store is a link or another user can reach it:
    /// the caller restricts it, a store being its user's alone.
    StoreExposed,
    /// The pimdir store is not the one that answered a queue row's handle,
    /// a copy restored since: nothing was done with the row.
    StoreReset,
    /// The lock file of the pimdir store was removed or replaced under a
    /// write: nothing was queued, and the caller retries.
    LockReplaced,
    /// A flag set holds a flag the pimdir store cannot: refused before
    /// anything is queued.
    FlagInvalid,
    /// A queue action's payload is one its schema refuses: refused before
    /// anything is queued.
    ActionMalformed,
    /// The pimdir conflict holds a body nobody has read: the caller
    /// fetches it first.
    ConflictUnread,
    /// A source of the pimdir store does not support the write: nothing
    /// was queued.
    Unsupported,
    /// No source of the pimdir store can perform the intent: nothing was
    /// queued.
    NoPerformer,
    /// Several sources of the pimdir store can perform the intent and none
    /// is chosen: nothing was queued.
    AmbiguousPerformer,
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
    let mut entry = ItemError::new(id, &with_code(err, code));
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
#[cfg_attr(
    not(any(feature = "jmap", feature = "gcal", feature = "pimdir")),
    allow(dead_code)
)]
pub fn item_not_found(message: impl Into<String>) -> Error {
    CodedError::new(ErrorCode::ItemNotFound, message.into()).into()
}

/// `err` read as the item `item_id` missing from `calendar_id`, when coded
/// `item-not-found`: ``Item `<id>` not found in calendar `<calendar>` ``,
/// as Himalaya words a missing message (see [`missing`]).
pub fn item_missing(err: Error, item_id: &str, calendar_id: &str) -> Error {
    missing(err, || {
        format!("Item `{item_id}` not found in calendar `{calendar_id}`")
    })
}

/// `err` read as the event `event_id` missing from `calendar_id`, when
/// coded `item-not-found`: ``Event `<id>` not found in calendar
/// `<calendar>` ``, then how an event is named, a UID being no id (see
/// [`missing`]).
pub fn event_missing(err: Error, event_id: &str, calendar_id: &str) -> Error {
    missing(err, || {
        format!(
            "Event `{event_id}` not found in calendar `{calendar_id}`; an event is \
             named by the id `event list` shows, `event search <UID>` finding it by \
             its iCalendar UID"
        )
    })
}

/// `err`, a read of the item `item_id` of `calendar_id` failing, under
/// that read, unless coded `item-not-found`: the not-found sentence then
/// names it, the backend's answer its one source.
#[cfg_attr(
    not(any(
        feature = "caldav",
        feature = "jmap",
        feature = "gcal",
        feature = "msgraph",
        feature = "vdir"
    )),
    allow(dead_code)
)]
pub fn read_failed(err: impl Into<Error>, item_id: &str, calendar_id: &str) -> Error {
    let err = err.into();

    if code(&err) == Some(ErrorCode::ItemNotFound) {
        return err;
    }

    err.context(format!(
        "Read item `{item_id}` from calendar `{calendar_id}`"
    ))
}

/// `err` under the not-found `sentence`, when coded `item-not-found`, one
/// sentence whatever the backend: its code kept, the backend's own words
/// under it as its sources. Any other failure is returned as it is.
fn missing(err: Error, sentence: impl FnOnce() -> String) -> Error {
    if code(&err) != Some(ErrorCode::ItemNotFound) {
        return err;
    }

    let sentence = sentence();

    // NOTE: the pimdir backend words its miss so already.
    match err.to_string() == sentence {
        true => err,
        false => err.context(sentence),
    }
}

/// A refusal under `calendar-not-found`: the calendar a command names does
/// not exist.
#[cfg_attr(
    not(any(
        feature = "caldav",
        feature = "jmap",
        feature = "pimdir",
        feature = "vdir"
    )),
    allow(dead_code)
)]
pub fn calendar_not_found(message: impl Into<String>) -> Error {
    CodedError::new(ErrorCode::CalendarNotFound, message.into()).into()
}

/// The family sentence of a create naming the calendar `name` that exists
/// already.
#[cfg_attr(
    not(any(feature = "caldav", feature = "msgraph", feature = "vdir")),
    allow(dead_code)
)]
fn calendar_exists(name: &str) -> String {
    format!("Cannot create calendar `{name}`: it exists already")
}

/// A refusal under `calendar-duplicate`: a create names the calendar
/// `name` that the store holds already.
#[cfg_attr(not(feature = "vdir"), allow(dead_code))]
pub fn calendar_duplicate(name: &str) -> Error {
    CodedError::new(ErrorCode::CalendarDuplicate, calendar_exists(name)).into()
}

/// The failure of a create naming the calendar `name` the server answered
/// exists already, under `calendar-duplicate`, `cause` what it answered
/// kept in `sources`.
#[cfg_attr(not(any(feature = "caldav", feature = "msgraph")), allow(dead_code))]
pub fn existing_calendar(name: &str, cause: impl Into<Error>) -> Error {
    with_code(cause.into(), |_| Some(ErrorCode::CalendarDuplicate)).context(calendar_exists(name))
}

/// A refusal under `calendar-default-missing`: a command naming no calendar
/// found no `calendar.default` either.
pub fn calendar_default_missing() -> Error {
    let message = "No default calendar found, name one with `-k <CALENDAR-ID>` or set \
                   `calendar.default`";
    CodedError::new(ErrorCode::CalendarDefaultMissing, message).into()
}

/// A refusal under `backend-not-built`: the account's `backend` block
/// loaded unread, this build leaving that backend out, naming the first of
/// the account's `built` backends, which `--backend` reaches instead.
pub fn backend_not_built(backend: &str, built: &[&str]) -> Error {
    let mut message = crate::backend::not_built(backend);

    if let Some(other) = built.first() {
        message.push_str(&format!(
            "; this account's {other} backend is built (`--backend {other}`)"
        ));
    }

    CodedError::new(ErrorCode::BackendNotBuilt, message).into()
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
/// the one of the account pimalaya-config could not take, else the server
/// refusal's under it.
pub fn code(err: &Error) -> Option<ErrorCode> {
    code_of(err)
        .or_else(|| account_code(err))
        .or_else(|| refusal_code(err))
}

/// The code of the [`AccountError`] in `err`'s chain: the account `-a`
/// names, or the one marked default, pimalaya-config could not take.
fn account_code(err: &Error) -> Option<ErrorCode> {
    let err = err
        .chain()
        .find_map(|cause| cause.downcast_ref::<AccountError>())?;

    Some(match err {
        AccountError::Missing { .. } => ErrorCode::AccountMissing,
        AccountError::NoDefault => ErrorCode::AccountDefaultMissing,
        AccountError::SeveralDefaults(_) => ErrorCode::AccountDefaultSeveral,
    })
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
        #[cfg(feature = "jmap")]
        crate::jmap::error::refusal_code,
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
