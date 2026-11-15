//! # PROPPATCH command
//!
//! The `calendula caldav proppatch` command, setting the DAV properties of
//! a calendar collection with a `PROPPATCH`.

use anyhow::{Result, bail};
use clap::Parser;
use io_webdav::rfc4791::calendar::CaldavCalendarPatch;
use pimalaya_cli::printer::Printer;

use crate::{caldav::client::CaldavClient, shared::receipt::IdReceipt};

/// PROPPATCH: set properties on a calendar collection.
///
/// Only the flags you pass are sent (a PROPPATCH `set`), so unset
/// properties are left untouched; clearing a property is not exposed
/// (`calendar update -d ""` clears one).
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CaldavProppatchCommand {
    /// Identifier (last URL segment) of the calendar to patch.
    #[arg(value_name = "ID")]
    pub id: String,

    /// New human-readable name (DAV:displayname).
    #[arg(short, long, value_name = "TEXT")]
    pub name: Option<String>,

    /// New free-form description.
    #[arg(short, long, value_name = "TEXT")]
    pub description: Option<String>,

    /// New ASCII `#RRGGBB` color marker.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,

    /// New default time zone, as a whole VTIMEZONE block.
    #[arg(long, value_name = "VTIMEZONE")]
    pub tz: Option<String>,
}

impl CaldavProppatchCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        if self.name.is_none()
            && self.description.is_none()
            && self.color.is_none()
            && self.tz.is_none()
        {
            bail!("Nothing to patch: pass at least one of --name, --description, --color or --tz");
        }

        let patch = CaldavCalendarPatch {
            id: self.id.clone(),
            display_name: self.name.map(Some),
            description: self.description.map(Some),
            color: self.color.map(Some),
            tz: self.tz.map(Some),
        };

        client.update_calendar(&patch)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Calendar properties patched",
        })
    }
}
