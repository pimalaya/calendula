//! pimdir backend: calendula over a local [pimdir] store, an offline
//! cache (a SQLite index plus content-addressed blobs) that a sync
//! engine populates rather than a live server.
//!
//! Reads project the store's shared items through [`io_pimdir`]'s
//! client read API and are availability-aware: an item whose body is
//! not local lists fine from its stored summary but reads as "body not
//! fetched", the cue to sync, rather than as an error. Writes are
//! staged [`io_replica`] mutations a later sync propagates; they are
//! attributed to the configured [`source`], which must match the sync
//! source for the change to reach a server.
//!
//! Calendars themselves come from the sync, so the collection verbs
//! (create, update, delete) are not served here: a cache does not
//! invent collections its source does not have.
//!
//! [pimdir]: https://github.com/pimalaya/pimdir
//! [`source`]: crate::config::PimdirConfig::source

pub mod backend;
pub mod cli;
pub mod client;
pub mod hash;
pub mod meta;
pub mod status;
