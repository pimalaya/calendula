//! # Account
//!
//! The `account` command group, reading the accounts a configuration
//! declares: listing them, checking that their backends answer, and
//! listing what those backends serve.

pub mod capability;
pub mod check;
pub mod cli;
pub mod context;
pub mod list;
