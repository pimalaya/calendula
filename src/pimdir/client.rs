//! calendula wrapper around [`io_pimdir`]'s store and blob reader.

use std::path::PathBuf;

use anyhow::{Result, anyhow};
use io_pimdir::{PimdirBlobs, PimdirStore};

use crate::config::PimdirConfig;

/// A live pimdir client: an opened store (as one source) plus a
/// connection-independent blob reader over the same directory.
pub struct PimdirClient {
    pub(crate) store: PimdirStore,
    pub(crate) blobs: PimdirBlobs,
    /// The replica source this client opened the store as; a staged
    /// write is attributed to it.
    pub(crate) source: String,
}

impl PimdirClient {
    /// Opens, creating if absent, the pimdir store at the configured
    /// root.
    ///
    /// Reads are source-independent; the source only labels this
    /// client's writes. When `pimdir.source` is unset it is
    /// auto-detected: a store synced as a single source (the ordinary
    /// one-device case) has exactly one, so writes are attributed
    /// without configuration, falling back to `local` when the store
    /// has none or several.
    pub fn new(config: PimdirConfig) -> Result<Self> {
        // NOTE: `root` is a PathBuf carrying the raw `~/…` verbatim, and
        // opening it unexpanded would silently create an empty store at
        // a literal ./~/… relative to the cwd, which reads back as an
        // empty calendar list rather than as an error.
        let root = shellexpand::full(&config.root.to_string_lossy())
            .map(|expanded| PathBuf::from(expanded.into_owned()))
            .unwrap_or_else(|_| config.root.clone());

        let open = |source: &str| {
            PimdirStore::open(&root, source)
                .map_err(|err| anyhow!("Open pimdir store `{}`: {err}", root.display()))
        };

        let source = match config.source.clone() {
            Some(source) => source,
            None => {
                let probe = open("probe")?;
                match probe.distinct_sources()?.as_slice() {
                    [only] => only.clone(),
                    _ => String::from("local"),
                }
            }
        };

        let store = open(&source)?;
        let blobs = PimdirBlobs::open(&root);

        Ok(Self {
            store,
            blobs,
            source,
        })
    }

    /// The source this client attributes its writes to.
    pub fn source(&self) -> &str {
        &self.source
    }
}
