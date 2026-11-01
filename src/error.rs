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
/// Every code so far is the pimdir backend's, so a build without it
/// raises none, yet keeps the whole vocabulary in its schema.
#[cfg_attr(not(feature = "pimdir"), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
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
    /// The pimdir store's schema is newer than this calendula's: the
    /// caller upgrades calendula.
    StoreNewer,
    /// The pimdir database is not a pimdir store, another program having
    /// written it: the caller points calendula at the store its sync
    /// engine fills.
    StoreForeign,
    /// The pimdir store was written by an earlier draft: its sync engine,
    /// run or upgraded, brings it to the current schema.
    StoreStale,
    /// The pimdir store's two version stamps disagree.
    StoreCorrupt,
    /// The pimdir store names its bodies by a hash this calendula does not
    /// know: the caller upgrades calendula.
    StoreHashAlgo,
    /// Another process holds the pimdir store: the caller retries.
    StoreBusy,
}
