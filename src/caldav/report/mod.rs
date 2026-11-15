//! # REPORT commands
//!
//! One module per typed CalDAV REPORT (`calendar-query`,
//! `calendar-multiget`, `sync-collection`), plus a raw escape hatch and
//! the item list they share.

pub mod cli;
pub mod entries;
pub mod multiget;
pub mod query;
pub mod raw;
pub mod sync;
