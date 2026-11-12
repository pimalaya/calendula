//! # Unified API
//!
//! The cross-protocol surface: calendula's own types, the dispatcher over
//! the backends, and the command families built on them.
//!
//! The surface is the union of what the backends serve, each command
//! requiring the capabilities it reaches before anything connects; an
//! operation a backend serves with another result stays in its protocol
//! module.

pub mod arg;
pub mod build;
pub mod calendar;
pub mod client;
pub mod composer;
pub mod event;
pub mod ical;
pub mod item;
pub mod listing;
pub mod receipt;
pub mod uuid;
