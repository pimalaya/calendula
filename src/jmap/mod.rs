//! # JMAP
//!
//! The JMAP arm of the shared-API client ([`backend`]): an account's
//! calendars over io-jmap's JMAP for Calendars (draft-ietf-jmap-calendars),
//! events projected through ical-rs's JSCalendar conversion ([`project`]).
//! No protocol-specific command tree sits beside it yet.

pub mod backend;
pub mod calendar_set;
pub mod client;
pub mod error;
pub mod project;
