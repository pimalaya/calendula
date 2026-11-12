//! # Pimdir count
//!
//! The `pimdir count` command.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::pimdir::{
    backend::PimdirBackend,
    index::{PimdirCalendarCoverage, PimdirCalendarZones, parse_query, search_clock},
};

/// Count the account's events a query matches.
///
/// Takes what `pimdir search` takes.
///
/// JSON output: `{"count", "coverage": {"complete", "bodiless",
/// "unindexed", "behind", "since", "until", "filling", "horizonLeft",
/// "cut"}}`.
#[derive(Debug, Parser)]
pub struct PimdirCountCommand {
    /// Calendar to count in alone; every calendar when omitted.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,

    /// The query, its words joined by spaces.
    #[arg(value_name = "QUERY", required = true)]
    #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
    pub query: Vec<String>,
}

impl PimdirCountCommand {
    pub fn execute(self, printer: &mut impl Printer, backend: PimdirBackend) -> Result<()> {
        let query = parse_query(&self.query, printer.is_json())?;
        if let Some(calendar) = &self.calendar {
            backend.known_collection(calendar)?;
        }

        let count = backend
            .client
            .count_items(query, self.calendar.as_deref(), &search_clock())?;

        printer.out(PimdirCountOutput {
            count: count.count,
            coverage: count.coverage,
            zones: count.zones,
        })
    }
}

/// The `pimdir count` output.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCountOutput {
    /// How many events the query matches.
    pub count: u64,
    /// What the count could not see.
    pub coverage: PimdirCalendarCoverage,
    /// The zones, when the index reads floating times in another.
    #[serde(skip)]
    pub zones: Option<PimdirCalendarZones>,
}

impl fmt::Display for PimdirCountOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} event(s)", self.count)?;
        write!(f, "{}", self.coverage)?;
        if let Some(zones) = &self.zones {
            write!(f, "{zones}")?;
        }

        Ok(())
    }
}
