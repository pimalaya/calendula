//! calendula wrapper around [`io_pimdir`]'s reader and producer roles.
//!
//! The store belongs to the sync engine, not to calendula. Reads go
//! through [`PimdirReader`], the role that takes no lock (pimdir SPEC
//! 8) and carries no write at all, so a sync in flight neither blocks
//! calendula nor is blocked by it. Writes go through
//! [`PimdirProducer`], which takes the shared lock for the length of
//! one enqueue: a producer stages actions for the owner to apply and
//! never mutates the index itself.
//!
//! The reader folds the queue's pending actions over the committed rows
//! (pimdir SPEC 15.4), so an action this process staged reads back
//! before the store's owner applies it.

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use io_pimdir::{PimdirBlobs, PimdirProducer, PimdirReader};

use crate::config::PimdirConfig;

/// The process name each staged action records (pimdir SPEC 15.1),
/// diagnostic only: it says who asked, never who applies.
const PRODUCER: &str = "calendula";

/// A live pimdir client: a lock-free reader over the store, plus a blob
/// reader over the same directory.
///
/// A write opens a producer of its own and drops it, so this handle
/// never holds anything a sync has to wait on.
pub struct PimdirClient {
    pub(crate) reader: PimdirReader,
    pub(crate) blobs: PimdirBlobs,
    /// The expanded store root, which a producer is opened against.
    root: PathBuf,
    /// The account the collections are grouped under, `None` in a
    /// single-account store.
    pub(crate) account: Option<String>,
}

impl PimdirClient {
    /// Opens the pimdir store at the configured root to read.
    ///
    /// The store must exist: a reader creates nothing, the schema being
    /// the owner's to write, so a root holding no store fails here
    /// rather than listing an empty calendar set.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        // NOTE: `root` is a PathBuf carrying the raw `~/…` verbatim, and
        // opening it unexpanded would look for a store at a literal
        // ./~/… relative to the cwd.
        let root = shellexpand::full(&config.root.to_string_lossy())
            .map(|expanded| PathBuf::from(expanded.into_owned()))
            .unwrap_or_else(|_| config.root.clone());

        let reader = PimdirReader::open(&root)
            .map(PimdirReader::with_pending)
            .map_err(|err| anyhow!("Open pimdir store `{}`: {err}", root.display()))?;
        let blobs = reader.blobs();

        Ok(Self {
            reader,
            blobs,
            root,
            account: config.account.clone(),
        })
    }

    /// Opens a producer for one staging window: the enqueue-only role,
    /// which takes the store's shared lock rather than the owner's
    /// exclusive one, so several run at once and none keeps a sync out.
    ///
    /// Opened per write and dropped with it, since what the lock buys is
    /// the window between a body reaching the blob tree and the queue
    /// row pinning it, which a collector must not run inside.
    pub(crate) fn producer(&self) -> Result<PimdirProducer> {
        let producer = PimdirProducer::open(&self.root, PRODUCER)
            .map_err(|err| anyhow!("Stage into pimdir store `{}`: {err}", self.root.display()))?;

        Ok(match self.account.clone() {
            Some(account) => producer.for_account(account),
            None => producer,
        })
    }
}
