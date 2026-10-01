//! # Google Calendar
//!
//! The Google Calendar API v3 backend, and the `gcal` command family.
//!
//! The three other backends speak iCalendar natively; this one does not.
//! Google stores a JSON event and exposes no per-event iCalendar
//! representation, so [`project`] synthesizes the document of record on read
//! and re-projects it on write, per cardamum's projection policy.
//!
//! The [`cli`] family covers what the shared surface cannot: sharing rules,
//! availability, recurrence expansion, server-side parsing, the colour
//! palettes and the account settings.

pub mod acl;
pub mod backend;
pub mod calendars;
pub mod cli;
pub mod client;
pub mod colors;
pub mod free_busy;
pub mod instances;
pub mod move_event;
pub mod quick_add;
pub mod render;
pub mod settings;
