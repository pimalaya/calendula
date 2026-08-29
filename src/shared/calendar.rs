//! # Calendar
//!
//! The calendar collection shared by every backend, and the `calendar`
//! command family built on it.
//!
//! [`Calendar`] is calendula's own least-common-denominator shape, no library
//! sitting between the CLI and the io-* crates. Each backend adapter projects
//! its native collection onto it, and [`CalendarDiff`] carries a partial
//! update back.

pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod update;

use serde::{Deserialize, Serialize};

/// A calendar collection.
///
/// Partial-coverage fields stay optional, populated only by the backends that
/// know them.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Calendar {
    /// Backend-specific identifier the other commands address it by.
    pub id: String,
    /// Human-readable display name, falling back to the id.
    pub name: String,
    /// Free-form description, when the backend exposes one.
    #[serde(default)]
    pub description: Option<String>,
    /// ASCII `#RRGGBB` color marker, when the backend exposes one.
    #[serde(default)]
    pub color: Option<String>,
}

/// A partial update applied to a [`Calendar`].
///
/// `None` leaves a field untouched and `Some` replaces it. The nested
/// `Option` on the clearable fields distinguishes setting a value from
/// clearing it.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarDiff {
    /// The new display name.
    #[serde(default)]
    pub name: Option<String>,
    /// The new description, or `Some(None)` to clear it.
    #[serde(default)]
    pub description: Option<Option<String>>,
    /// The new color, or `Some(None)` to clear it.
    #[serde(default)]
    pub color: Option<Option<String>>,
}
