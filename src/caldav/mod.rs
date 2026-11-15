//! # CalDAV
//!
//! The CalDAV backend: the shared-API adapter over io-webdav, the
//! connected client, and a `caldav` command per WebDAV method (RFC 4918,
//! 4791, 6578).

pub mod backend;
pub mod cli;
pub mod client;
pub mod delete;
pub mod discover;
pub mod get;
pub mod mkcalendar;
pub mod propfind;
pub mod proppatch;
pub mod put;
pub mod report;
