//! pimdir backend: calendula over a local [pimdir] store, an offline
//! cache (a SQLite index plus content-addressed blobs) that a sync
//! engine populates rather than a live server.
//!
//! Reads project the store's shared items through [`PimdirReader`], the
//! role that takes no lock and carries no write, and are
//! availability-aware: an item whose body is not local lists fine from
//! its stored summary but reads as "body not fetched", the cue to sync,
//! rather than as an error. The reader overlays the queue (pimdir SPEC
//! 15.4), so a staged edit or deletion shows on the next read.
//!
//! Writes are staged: an action appended to the store's queue for its
//! owner to apply and push. A staged creation is the one write with
//! nothing to show for it in a listing, having no public id until the
//! owner applies it, which is what `pimdir status` counts.
//!
//! Calendars themselves come from the sync, so the collection verbs
//! (create, update, delete) are not served here: a cache does not
//! invent collections its source does not have.
//!
//! [pimdir]: https://github.com/pimalaya/pimdir
//! [`PimdirReader`]: io_pimdir::PimdirReader

pub mod backend;
pub mod cli;
pub mod client;
pub mod status;
