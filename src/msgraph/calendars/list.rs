//! # Calendar list command
//!
//! Lists the calendars of the user.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::calendars::list::MsgraphCalendarsListParams;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{calendars::render::MsgraphCalendarsOutput, client::MsgraphClient};

/// List the calendars of the user (one Graph page).
///
/// JSON output: `{"calendars": [<raw Graph calendar>...],
/// "@odata.nextLink"}`, the link present when a page follows.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarListCommand {
    /// Maximum number of calendars in the page (`$top`).
    #[arg(short = 's', long, value_name = "N")]
    pub top: Option<u32>,
}

impl MsgraphCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let params = MsgraphCalendarsListParams {
            top: self.top,
            ..Default::default()
        };
        let page = client.calendars_list(&params)?.response;

        printer.out(MsgraphCalendarsOutput {
            calendars: page.value,
            next_link: page.next_link,
        })
    }
}
