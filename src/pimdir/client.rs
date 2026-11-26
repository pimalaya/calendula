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

use anyhow::Result;
use io_pimdir::{
    client::{PimdirError, blobs::PimdirBlobs, producer::PimdirProducer, reader::PimdirReader},
    sql::FORMAT_VERSION,
    version::PimdirFormatRefusal,
};
use pimalaya_cli::error::CodedError;

use crate::{config::PimdirConfig, error::ErrorCode, pimdir::backend::CALENDAR_KIND};

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
    /// The expanded store root, which a producer and the index open.
    pub(crate) root: PathBuf,
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
/// needs no configuration. A store whose calendars several accounts hold
/// is what `pimdir.account` answers, and leaving it unset there errors
/// rather than merging their calendars into one view; an account holding
/// no calendar is not counted.
fn resolve_account(reader: &PimdirReader, configured: Option<String>) -> Result<Option<String>> {
    let accounts_of = |kind| {
        reader
            .accounts_of(kind)
            .map_err(|err| store_error(err, "List pimdir collections"))
    };

    let names = |accounts: &[Option<String>]| {
        accounts
            .iter()
            .map(|account| account.as_deref().unwrap_or("<ungrouped>"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    if let Some(account) = &configured {
        let accounts = accounts_of(None)?;

        if !accounts.is_empty() && !accounts.contains(&configured) {
            let message = format!(
                "The pimdir store holds no account `{account}` ({}); fix `pimdir.account`",
                names(&accounts),
            );
            return Err(CodedError::new(ErrorCode::StoreAccountUnknown, message).into());
        }

        return Ok(configured);
    }

    let accounts = accounts_of(Some(CALENDAR_KIND))?;

    match accounts.as_slice() {
        [] | [_] => Ok(accounts.into_iter().next().flatten()),
        _ => {
            let message = format!(
                "The pimdir store holds several accounts of calendars ({}); name one with \
                 `pimdir.account`",
                names(&accounts),
            );
            Err(CodedError::new(ErrorCode::StoreAccountAmbiguous, message).into())
        }
    }
}

/// The pimdir codes calendula relays, by io-pimdir's spelling, which
/// serialises each: Himalaya's, but `tag-refused`, which only a mail tag
/// raises.
const PIMDIR_CODES: &[(&str, ErrorCode)] = &[
    ("store-version", ErrorCode::StoreVersion),
    ("store-uncreated", ErrorCode::StoreUncreated),
    ("store-foreign", ErrorCode::StoreForeign),
    ("store-corrupt", ErrorCode::StoreCorrupt),
    ("store-not-current", ErrorCode::StoreNotCurrent),
    ("store-hash-algo", ErrorCode::StoreHashAlgo),
    ("sync-version", ErrorCode::SyncVersion),
    ("store-busy", ErrorCode::StoreBusy),
    ("store-owned", ErrorCode::StoreOwned),
    ("store-staging", ErrorCode::StoreStaging),
    ("store-in-flight", ErrorCode::StoreInFlight),
    ("store-exposed", ErrorCode::StoreExposed),
    ("store-reset", ErrorCode::StoreReset),
    ("lock-replaced", ErrorCode::LockReplaced),
    ("item-duplicate", ErrorCode::ItemDuplicate),
    ("flag-invalid", ErrorCode::FlagInvalid),
    ("action-malformed", ErrorCode::ActionMalformed),
    ("conflict-unread", ErrorCode::ConflictUnread),
    ("unsupported", ErrorCode::Unsupported),
    ("no-performer", ErrorCode::NoPerformer),
    ("ambiguous-performer", ErrorCode::AmbiguousPerformer),
    ("index-missing", ErrorCode::IndexMissing),
    ("index-outdated", ErrorCode::IndexOutdated),
    ("index-version", ErrorCode::IndexVersion),
    ("index-zone-unknown", ErrorCode::IndexZoneUnknown),
    ("index-fts5-missing", ErrorCode::IndexFts5Missing),
    ("query-refused", ErrorCode::QueryRefused),
    ("query-unanswerable", ErrorCode::QueryUnanswerable),
];

impl ErrorCode {
    /// The code of a pimdir failure, `None` for one the caller cannot act
    /// on (an SQL, I/O or JSON failure).
    pub fn of_pimdir(err: &PimdirError) -> Option<Self> {
        let code = err.code();
        PIMDIR_CODES
            .iter()
            .find(|(pimdir, _)| *pimdir == code)
            .map(|(_, ours)| *ours)
    }
}

/// The error of a failed store operation, `context` naming it, carrying
/// the code of [`ErrorCode`] that io-pimdir's own stable one maps to.
///
/// Only the failures a caller acts on get one, the ones Himalaya relays
/// (`PIMDIR_CODES`): a store state, a sync or a producer holding it, a
/// write a source or the store refuses before anything is queued, an
/// intent no source or several can perform, a search index state, and a
/// search query refused or unanswered; an SQL, I/O or JSON failure gets
/// none. The message is io-pimdir's, but for a store of another format
/// version, worded by [`version_message`].
pub(crate) fn store_error(err: PimdirError, context: impl fmt::Display) -> anyhow::Error {
    let context = context.to_string();
    let message = match &err {
        PimdirError::Version { found } => version_message(found.as_deref(), FORMAT_VERSION),
        err => err.to_string(),
    };

    match ErrorCode::of_pimdir(&err) {
        Some(code) => anyhow::Error::new(CodedError::new(code, message)).context(context),
        None => anyhow::Error::new(err).context(context),
    }
}

/// What a store of another format version than `own` reads as, `found`
/// being the version it names, `None` for one naming none: io-pimdir's
/// one wording ([`PimdirFormatRefusal`]), naming calendula, which orders
/// the two by pimdir STORAGE §4.2 and never advises deleting the store.
fn version_message(found: Option<&str>, own: &str) -> String {
    let refusal = PimdirFormatRefusal {
        found,
        own,
        program: "calendula",
    };

    refusal.to_string()
}

/// A temporary directory holding a pimdir store this process owns for as
/// long as the directory lives.
///
/// A store opened again after its last handle dropped takes its owner lock
/// anew, and a child another test spawns holds the dropped lock until it
/// executes, so that open finds the store owned. The handle held here keeps
/// the lock in this process: every later open shares it.
#[cfg(test)]
pub(crate) struct StoreDir {
    _owner: io_pimdir::client::PimdirStore,
    dir: tempfile::TempDir,
}

#[cfg(test)]
impl std::ops::Deref for StoreDir {
    type Target = tempfile::TempDir;

    fn deref(&self) -> &tempfile::TempDir {
        &self.dir
    }
}

/// A fresh [`StoreDir`], its store created.
#[cfg(test)]
pub(crate) fn store_dir() -> StoreDir {
    let dir = tempfile::tempdir().unwrap();
    let owner = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();

    StoreDir { _owner: owner, dir }
}

/// Records the `caldav` source as syncing `collection` of the store at
/// `root`, under `account` when given, as a complete sync round does.
#[cfg(test)]
pub(crate) fn sync_from(root: &std::path::Path, account: Option<&str>, collection: &str) {
    use io_pimdir::{
        change::PimdirWriteOp,
        collection::{PimdirCheckpoint, PimdirCollectionId, PimdirScope},
    };

    let mut store = io_pimdir::client::PimdirStore::open(root).unwrap();
    if let Some(account) = account {
        store = store.for_account(account);
    }
    store
        .for_source("caldav")
        .unwrap()
        .write(vec![
            PimdirWriteOp::OpenRound {
                collection: PimdirCollectionId(collection.into()),
                scope: PimdirScope::unbounded(),
                band: false,
            },
            PimdirWriteOp::CloseRound {
                collection: PimdirCollectionId(collection.into()),
                coverage: PimdirScope::unbounded(),
                checkpoint: Some(PimdirCheckpoint(b"0".to_vec())),
            },
        ])
        .unwrap();
}

#[cfg(test)]
mod tests {
    use io_pimdir::{client::PimdirError, codec::PimdirActionError, search::query::PimdirQuery};
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
            PimdirError::NotCurrent(io_pimdir::client::PimdirNotCurrent::Version { found: 0 }),
            PimdirError::SyncVersion {
                found: "sync-draft-07".into(),
            },
            PimdirError::Busy,
            PimdirError::Exposed("/tmp/store".into()),
            PimdirError::Owned("/tmp/store".into()),
            PimdirError::Staging("/tmp/store".into()),
            PimdirError::InFlight("/tmp/store".into()),
            PimdirError::Unsupported(io_pimdir::capability::PimdirRefusal {
                capability: "calendar.online-meeting".into(),
                source: "caldav".into(),
                detail: None,
            }),
            PimdirError::NoPerformer {
                capability: "calendar.reply".into(),
            },
            PimdirError::Ambiguous {
                capability: "calendar.reply".into(),
                candidates: vec!["caldav".into(), "gcal".into()],
            },
            PimdirError::Unread {
                collection: "cal".into(),
                handle: "h".into(),
            },
            PimdirError::LockReplaced("/tmp/store/objects.lock".into()),
            PimdirError::Flag("two words".into()),
            PimdirError::Action(PimdirActionError::MissingField("source")),
            PimdirError::IndexMissing("/tmp/store/index.db".into()),
            PimdirError::IndexOutdated("/tmp/store/index.db".into()),
            PimdirError::IndexVersion {
                path: "/tmp/store/index.db".into(),
                version: Some("search-draft-07".into()),
                shape: Some("canonical".into()),
            },
            PimdirError::IndexZone("Mars/Olympus".into()),
            PimdirError::Fts5 { version: "3.31.0" },
            PimdirError::Query(PimdirQuery::parse("frm:alice").unwrap_err()),
        ];

        for err in failures {
            let name = err.code();
            let shown = err.to_string();
            let err = store_error(err, "Open pimdir store");
            let code: ErrorCode = code_of(&err).unwrap_or_else(|| panic!("no code for {shown}"));
            assert_eq!(serde_json::to_value(code).unwrap(), name, "{shown}");
        }
    }

    /// The codes relayed keep io-pimdir's spelling, which is Himalaya's.
    #[test]
    fn a_relayed_pimdir_code_keeps_io_pimdirs_spelling() {
        for (pimdir, ours) in super::PIMDIR_CODES {
            assert_eq!(serde_json::to_value(ours).unwrap(), *pimdir);
        }
    }

    /// A store an earlier sync engine wrote is drained by it, then set
    /// aside, and one a later engine wrote needs a later calendula, the
    /// two ordered by pimdir STORAGE §4.2: under a release, every draft
    /// store is the earlier; neither says to delete the store.
    #[test]
    fn a_store_of_another_version_names_the_drain_or_the_upgrade() {
        let refused = |found: Option<&str>, own: &str| super::version_message(found, own);

        for (found, own) in [
            (None, "storage-draft-14"),
            (Some("storage-draft-10"), "storage-draft-14"),
            (Some("storage-draft-14"), "storage-v1"),
        ] {
            let earlier = refused(found, own);
            assert!(earlier.contains("earlier sync engine"), "{earlier}");
            assert!(earlier.contains("until nothing is queued"), "{earlier}");
            assert!(earlier.contains("move its directory aside"), "{earlier}");
            assert!(!earlier.contains("delet"), "{earlier}");
        }

        for (found, own) in [
            (Some("storage-draft-15"), "storage-draft-14"),
            (Some("storage-v1"), "storage-draft-14"),
        ] {
            let later = refused(found, own);
            assert!(later.contains("upgrade calendula"), "{later}");
            assert!(!later.contains("aside"), "{later}");
            assert!(!later.contains("delet"), "{later}");
        }
    }
}
