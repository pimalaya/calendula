//! # Microsoft Graph
//!
//! The Microsoft Graph arm of the shared-API client ([`backend`]): a
//! user's Outlook calendars over io-msgraph, events projected through its
//! `ical` feature; and the Graph-specific command tree ([`cli`]), nested
//! by Graph resource as cardamum's is.

pub mod backend;
pub mod calendars;
pub mod cli;
pub mod client;
pub mod events;
pub mod profile;
pub mod request;
