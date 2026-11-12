//! # Error codes
//!
//! A stable, machine-readable name for the failures a caller is expected
//! to act on, printed as `code` beside the message under `--json`.
//!
//! The message stays for humans and may be reworded at any time; the
//! code is the contract. A failure with no code prints as before, with
//! no `code` field. The mechanism (`CodedError`, `code_of`, the report)
//! lives in `pimalaya_cli::error`; only the vocabulary is calendula's.

use schemars::JsonSchema;
use serde::Serialize;

/// The stable codes, each serialised in kebab-case.
///
/// Every code but `capability-missing` is the pimdir backend's, so a
/// build without it raises none of them, yet keeps the whole vocabulary
/// in its schema.
#[cfg_attr(not(feature = "pimdir"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The backend the account resolves to does not serve a capability
    /// the command reaches, refused before connecting: the caller picks
    /// another backend or leaves the option out.
    CapabilityMissing,
    /// The item is listed but its body is not local yet: a sync will
    /// bring it, so the caller waits rather than reports a loss.
    BodyPending,
    /// A write named, by `--if-match`, a version the item no longer has:
    /// the caller reads it again rather than overwrites a change.
    PreconditionFailed,
    /// A reply or a cancel named one occurrence, which its performer
    /// cannot act on alone: the caller acts on the whole series or not
    /// at all.
    OccurrenceUnsupported,
    /// A pimdir create names a `UID` its calendar already holds or has
    /// queued: the caller updates that item rather than creates it again.
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
