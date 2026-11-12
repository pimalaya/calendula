//! # Colors
//!
//! The `calendula gcal colors` command, showing the calendar and event
//! colour palettes.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::gcal::client::GcalClient;

/// Show the two colour palettes.
///
/// Google talks in colour ids rather than hex values, so this is what turns
/// a `colorId` into something readable, and what tells you which id to
/// reach for.
///
/// JSON output: `{"colors": [{"palette", "id", "background",
/// "foreground"}]}`.
#[derive(Debug, Parser)]
pub struct GcalColorsCommand;

impl GcalColorsCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: GcalClient) -> Result<()> {
        let palettes = client.colors_get()?.response;

        let rows = palettes
            .calendar
            .into_iter()
            .map(|entry| (entry, "calendar"))
            .chain(palettes.event.into_iter().map(|entry| (entry, "event")))
            .map(|((id, definition), palette)| ColorRow {
                palette,
                id,
                background: definition.background,
                foreground: definition.foreground,
            })
            .collect();

        printer.out(GcalColorsOutput { rows })
    }
}

/// The palettes.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GcalColorsOutput {
    /// Both palettes flattened, one entry each.
    #[serde(rename = "colors")]
    pub rows: Vec<ColorRow>,
}

/// One palette entry.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ColorRow {
    /// Which palette the entry belongs to: `calendar` or `event`.
    pub palette: &'static str,
    /// The id a `colorId` field carries.
    pub id: String,
    /// The background colour, as `#rrggbb`.
    pub background: Option<String>,
    /// The foreground colour, as `#rrggbb`.
    pub foreground: Option<String>,
}

impl fmt::Display for GcalColorsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::new(["PALETTE", "ID", "BACKGROUND", "FOREGROUND"]);

        for color in &self.rows {
            let background = color.background.as_deref().unwrap_or_default();
            let foreground = color.foreground.as_deref().unwrap_or_default();

            columns.push([
                String::from(color.palette),
                sanitize::text(&color.id).into_owned(),
                sanitize::text(background).into_owned(),
                sanitize::text(foreground).into_owned(),
            ]);
        }

        write!(f, "{columns}")
    }
}
