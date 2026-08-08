//! Calendar collection types shared by every backend, plus the
//! `calendar` command family built on them.
//!
//! [`Calendar`] is calendula's own least-common-denominator shape
//! across the backends it targets (vdir, CalDAV, pimdir); no library
//! sits between the CLI and the io-* crates. Each backend adapter
//! projects its native collection onto it, and [`CalendarDiff`]
//! carries a partial update back.

pub mod cli;
pub mod create;
pub mod delete;
pub mod list;
pub mod update;

use serde::{Deserialize, Serialize};

/// A calendar collection.
///
/// Partial-coverage fields stay optional and are populated only by the
/// backends that know them: vdir reads them from its metadata marker
/// files, CalDAV from the collection properties, pimdir from the
/// stored collection row.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Calendar {
    /// Backend-specific identifier: the collection directory name
    /// (vdir), the last path segment of the collection URL (CalDAV),
    /// or the stored collection id (pimdir).
    pub id: String,
    /// Human-readable display name, falling back to the id when the
    /// backend carries none.
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
/// Every field is optional: `None` leaves the field untouched, and
/// `Some` replaces it. The nested `Option` on the clearable fields
/// distinguishes "set to this value" from "clear it".
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
