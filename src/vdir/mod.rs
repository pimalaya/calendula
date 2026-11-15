//! # Vdir
//!
//! The vdir backend: the shared-API adapter over io-vdir, the client
//! rooted at a home directory, and the `vdir` commands over its collections
//! and their raw items.

pub mod backend;
pub mod cli;
pub mod client;
pub mod create;
pub mod delete;
pub mod item;
pub mod list;
pub mod rename;
