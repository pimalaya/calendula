//! # Pimdir client
//!
//! calendula's wrapper around [`io_pimdir`]'s reader and producer
//! roles. The store belongs to the sync engine, not to calendula.
//!
//! Reads go through [`PimdirReader`], which takes no lock (pimdir
//! STORAGE §8) and carries no write, so a sync in flight neither blocks
//! calendula nor is blocked by it. Writes go through [`PimdirProducer`]:
//! a write opens one, writes its body and the row pinning it through it
//! in one step under its shared staging lock, and drops it; it never
//! mutates the index.
//!
//! A store failure carries the stable code io-pimdir gives it
//! ([`PimdirError::code`]) when it is one a caller acts on, see
//! [`store_error`].
//!
//! The reader folds the queue's pending actions over the committed rows
//! (pimdir STORAGE §15.4), so an action this process staged reads back
//! before the store's owner applies it.

use std::{fmt, path::PathBuf};

use anyhow::{Result, anyhow};
use io_pimdir::{
    client::{PimdirError, blobs::PimdirBlobs, producer::PimdirProducer, reader::PimdirReader},
    sql::FORMAT_VERSION,
};
use pimalaya_cli::error::CodedError;

use crate::{config::PimdirConfig, error::ErrorCode};

/// The process name each staged action records (pimdir STORAGE §15.1).
///
/// Diagnostic only: it says who asked, never who applies.
const PRODUCER: &str = "calendula";

/// A live pimdir client: a lock-free reader over the store and blobs.
///
/// A write opens a producer of its own and drops it, so this handle
/// never holds anything a sync has to wait on.
pub struct PimdirClient {
    /// The lock-free reader every read goes through.
    pub(crate) reader: PimdirReader,
    /// The blob store item bodies are read from, read-only: a body is
    /// staged through a producer, under its lock.
    pub(crate) blobs: PimdirBlobs,
    /// The expanded store root, which a producer is opened against.
    root: PathBuf,
    /// The account collections are grouped under, configured or derived,
    /// `None` when the store groups none.
    pub(crate) account: Option<String>,
}

impl PimdirClient {
    /// Opens the pimdir store at the configured root to read.
    ///
    /// The store must exist: a reader creates nothing, the schema being
    /// the owner's to write, so a root holding no store fails here
    /// rather than listing an empty calendar set.
    ///
    /// The root arrives shell-expanded, [`PimdirConfig::root`] doing it
    /// at deserialize.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        let root = config.root.clone();

        let reader = PimdirReader::open(&root)
            .map(PimdirReader::with_pending)
            .map_err(|err| {
                store_error(err, format_args!("Open pimdir store `{}`", root.display()))
            })?;
        let blobs = reader.blobs();
        let account = resolve_account(&reader, config.account)?;

        Ok(Self {
            reader,
            blobs,
            root,
            account,
        })
    }

    /// Opens a producer for one write.
    ///
    /// The enqueue-only role takes the shared staging lock, not the
    /// owner's exclusive one, so it keeps no sync out; it keeps a
    /// collector out from the body it stages until the row that pins it
    /// commits, which is what a write needs, and holds nothing while idle.
    pub(crate) fn producer(&self) -> Result<PimdirProducer> {
        let producer = PimdirProducer::open(&self.root, PRODUCER).map_err(|err| {
            store_error(
                err,
                format_args!("Stage into pimdir store `{}`", self.root.display()),
            )
        })?;

        Ok(match self.account.clone() {
            Some(account) => producer.for_account(account),
            None => producer,
        })
    }
}

/// The account this client reads, configured or derived (pimdir STORAGE
/// §9.2).
///
/// A store one account syncs groups its collections under that name, or
/// under none in a store predating the grouping, so the common case
/// needs no configuration. A store several accounts share is what
/// `pimdir.account` answers, and leaving it unset there errors rather
/// than merging their calendars into one view.
fn resolve_account(reader: &PimdirReader, configured: Option<String>) -> Result<Option<String>> {
    if configured.is_some() {
        return Ok(configured);
    }

    let mut accounts: Vec<Option<String>> = reader
        .list_collections()
        .map_err(|err| store_error(err, "List pimdir collections"))?
        .into_iter()
        .map(|collection| collection.account)
        .collect();
    accounts.sort();
    accounts.dedup();

    match accounts.as_slice() {
        [] | [_] => Ok(accounts.into_iter().next().flatten()),
        _ => {
            let names: Vec<&str> = accounts
                .iter()
                .map(|account| account.as_deref().unwrap_or("<ungrouped>"))
                .collect();
            Err(anyhow!(
                "The pimdir store holds several accounts ({}); name one with `pimdir.account`",
                names.join(", "),
            ))
        }
    }
}

/// The error of a failed store operation, `context` naming it, carrying
/// the code of [`ErrorCode`] that io-pimdir's own stable one maps to.
///
/// Only the failures a caller acts on get one: a store its sync engine
/// has to create, one of another format version, a database another
/// program wrote, one whose two version stamps disagree, one naming its
/// bodies by a hash calendula does not know, one another process holds,
/// one another user can reach, and an action or a flag the store refuses
/// before anything is queued. The message is io-pimdir's, but for a store
/// of another format version: a store is never migrated (pimdir STORAGE
/// §6), so one an earlier sync engine wrote is reinstalled by the engine
/// that owns it, which moves it aside, and one a newer engine wrote needs
/// a newer calendula. No message advises deleting the store.
pub(crate) fn store_error(err: PimdirError, context: impl fmt::Display) -> anyhow::Error {
    let code = match err.code() {
        "store-uncreated" => ErrorCode::StoreUncreated,
        "store-version" => ErrorCode::StoreVersion,
        "store-foreign" => ErrorCode::StoreForeign,
        "store-corrupt" => ErrorCode::StoreCorrupt,
        "store-hash-algo" => ErrorCode::StoreHashAlgo,
        "store-busy" => ErrorCode::StoreBusy,
        "store-exposed" => ErrorCode::StoreExposed,
        "flag-invalid" => ErrorCode::FlagInvalid,
        "action-malformed" => ErrorCode::ActionMalformed,
        _ => return anyhow!("{context}: {err}"),
    };

    let message = match &err {
        PimdirError::Version { found } => version_message(found.as_deref(), context),
        err => format!("{context}: {err}"),
    };

    CodedError::new(code, message).into()
}

/// What a store of another format version than calendula's reads as,
/// `found` being the version it names, `None` for one naming none.
///
/// A store naming none, as every draft before storage-draft-08 wrote, or
/// an earlier draft, is an earlier sync engine's: its queued writes live
/// in it alone, so it is synced by that engine until nothing is queued
/// before the current one reinstalls it. Any other is a newer one's.
fn version_message(found: Option<&str>, context: impl fmt::Display) -> String {
    let draft = |version: &str| {
        version
            .strip_prefix("storage-draft-")
            .and_then(|number| number.parse::<u32>().ok())
    };
    let earlier = match found {
        None => true,
        Some(found) => matches!(
            (draft(found), draft(FORMAT_VERSION)),
            (Some(found), Some(ours)) if found < ours
        ),
    };
    let found = found.unwrap_or("of no format version");

    match earlier {
        true => format!(
            "{context}: the pimdir store is {found}, written by an earlier sync engine, and \
             calendula reads {FORMAT_VERSION} alone; a store is never migrated, so sync it \
             with the neverest that wrote it until nothing is queued, then reinstall it \
             with `neverest sync --reset --force -a <account>`, which moves it aside, \
             deleting nothing, and syncs a new one"
        ),
        false => format!(
            "{context}: the pimdir store is {found}, written by a newer sync engine than \
             calendula's {FORMAT_VERSION}; upgrade calendula"
        ),
    }
}

#[cfg(test)]
mod tests {
    use io_pimdir::{client::PimdirError, codec::PimdirActionError};
    use pimalaya_cli::error::code_of;

    use crate::{error::ErrorCode, pimdir::client::store_error};

    /// Every store failure a caller acts on carries the very code io-pimdir
    /// names for it, so a script reads one vocabulary whichever program met
    /// the store.
    #[test]
    fn a_store_failure_carries_the_code_io_pimdir_names() {
        let failures = [
            PimdirError::Uncreated,
            PimdirError::Version {
                found: Some("storage-draft-07".into()),
            },
            PimdirError::Version { found: None },
            PimdirError::Foreign,
            PimdirError::VersionMismatch {
                user_version: 1,
                store_meta: 2,
            },
            PimdirError::HashAlgo {
                found: "md5".into(),
            },
            PimdirError::Busy,
            PimdirError::Exposed("/tmp/store".into()),
            PimdirError::Flag("two words".into()),
            PimdirError::Action(PimdirActionError::MissingField("source")),
        ];

        for err in failures {
            let name = err.code();
            let shown = err.to_string();
            let err = store_error(err, "Open pimdir store");
            let code: ErrorCode = code_of(&err).unwrap_or_else(|| panic!("no code for {shown}"));
            assert_eq!(serde_json::to_value(code).unwrap(), name, "{shown}");
        }
    }

    /// A store an earlier sync engine wrote is reinstalled by it, in the
    /// order that keeps its queued writes, and one a newer engine wrote
    /// needs a newer calendula; neither says to delete the store.
    #[test]
    fn a_store_of_another_version_names_the_reinstall_or_the_upgrade() {
        let refused = |found: Option<&str>| {
            store_error(
                PimdirError::Version {
                    found: found.map(Into::into),
                },
                "Open pimdir store",
            )
            .to_string()
        };

        for earlier in [refused(None), refused(Some("storage-draft-07"))] {
            assert!(earlier.contains("earlier sync engine"), "{earlier}");
            assert!(earlier.contains("until nothing is queued"), "{earlier}");
            assert!(
                earlier.contains("neverest sync --reset --force"),
                "{earlier}"
            );
            assert!(earlier.contains("deleting nothing"), "{earlier}");
        }

        for newer in [
            refused(Some("storage-draft-09")),
            refused(Some("storage-v1.0.0")),
        ] {
            assert!(newer.contains("upgrade calendula"), "{newer}");
            assert!(!newer.contains("move"), "{newer}");
            assert!(!newer.contains("delet"), "{newer}");
        }
    }
}
