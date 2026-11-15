//! # MKCALENDAR command
//!
//! The `calendula caldav mkcalendar` command, creating a calendar
//! collection under the resolved calendar home-set (RFC 4791 §5.3.1).

use anyhow::Result;
use clap::Parser;
use io_webdav::rfc4791::calendar::CaldavCalendar;
use pimalaya_cli::printer::Printer;

use crate::{caldav::client::CaldavClient, shared::receipt::IdReceipt};

/// MKCALENDAR: create a calendar collection (RFC 4791 §5.3.1).
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct CaldavMkcalendarCommand {
    /// Identifier (last URL segment) of the calendar to create.
    #[arg(value_name = "ID")]
    pub id: String,

    /// Optional human-readable name (DAV:displayname).
    #[arg(short, long, alias = "display-name", value_name = "TEXT")]
    pub name: Option<String>,

    /// Optional free-form description.
    #[arg(short, short_alias = 'D', long, value_name = "TEXT")]
    pub description: Option<String>,

    /// Optional ASCII `#RRGGBB` color marker.
    #[arg(short = 'C', long, value_name = "HEX")]
    pub color: Option<String>,

    /// Component kinds the calendar accepts (RFC 4791 5.2.3), such as
    /// VEVENT or VTODO. Repeat the flag for several.
    ///
    /// A server fixes this at creation and refuses to change it
    /// afterwards.
    #[arg(long = "component", value_name = "KIND")]
    pub components: Vec<String>,

    /// Default time zone, as a whole VTIMEZONE block.
    #[arg(long, value_name = "VTIMEZONE")]
    pub tz: Option<String>,
}

impl CaldavMkcalendarCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let calendar = CaldavCalendar {
            id: self.id.clone(),
            display_name: self.name,
            description: self.description,
            color: self.color,
            components: self.components.into_iter().collect(),
            tz: self.tz,
            ..Default::default()
        };

        client.create_calendar(&calendar)?;

        printer.out(IdReceipt {
            id: self.id,
            done: "Calendar created",
        })
    }
}
