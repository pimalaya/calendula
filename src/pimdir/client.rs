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
use io_pimdir::client::{
    PimdirError, blobs::PimdirBlobs, producer::PimdirProducer, reader::PimdirReader,
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
/// has to create or bring off an earlier draft, one newer than calendula,
/// a database another program wrote, one whose two version stamps
/// disagree, one naming its bodies by a hash calendula does not know, one
/// another process holds. An older store is read as it stands, so it fails
/// nothing. The message is io-pimdir's, which never advises deleting the
/// store.
pub(crate) fn store_error(err: PimdirError, context: impl fmt::Display) -> anyhow::Error {
    let code = match err.code() {
        "store-uncreated" => ErrorCode::StoreUncreated,
        "store-newer" => ErrorCode::StoreNewer,
        "store-foreign" => ErrorCode::StoreForeign,
        "store-stale" => ErrorCode::StoreStale,
        "store-corrupt" => ErrorCode::StoreCorrupt,
        "store-hash-algo" => ErrorCode::StoreHashAlgo,
        "store-busy" => ErrorCode::StoreBusy,
        _ => return anyhow!("{context}: {err}"),
    };

    CodedError::new(code, format!("{context}: {err}")).into()
}

#[cfg(test)]
mod tests {
    use io_pimdir::client::PimdirError;
    use pimalaya_cli::error::code_of;

    use crate::{error::ErrorCode, pimdir::client::store_error};

    /// Every store failure a caller acts on carries the very code io-pimdir
    /// names for it, so a script reads one vocabulary whichever program met
    /// the store.
    #[test]
    fn a_store_failure_carries_the_code_io_pimdir_names() {
        let failures = [
            PimdirError::Uncreated,
            PimdirError::Version { found: 99 },
            PimdirError::Foreign,
            PimdirError::Stale { missing: "items" },
            PimdirError::VersionMismatch {
                user_version: 1,
                store_meta: 2,
            },
            PimdirError::HashAlgo {
                found: "md5".into(),
                declared: None,
            },
            PimdirError::Busy,
        ];

        for err in failures {
            let name = err.code();
            let shown = err.to_string();
            let err = store_error(err, "Open pimdir store");
            let code: ErrorCode = code_of(&err).unwrap_or_else(|| panic!("no code for {shown}"));
            assert_eq!(serde_json::to_value(code).unwrap(), name, "{shown}");
        }
    }
}
