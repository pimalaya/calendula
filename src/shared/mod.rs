//! # Shared API
//!
//! The cross-protocol surface: calendula's own least-common-denominator
//! types, the dispatcher over the backends, and the command families built
//! on them.
//!
//! An operation only reaches this surface when every compiled backend can
//! serve it; what only one of them exposes lives in its protocol module.

pub mod arg;
pub mod calendar;
pub mod client;
pub mod event;
pub mod ical;
pub mod item;
pub mod journal;
pub mod table;
pub mod todo;
