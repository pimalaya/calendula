//! # Event list command
//!
//! Lists one page of the events of a calendar.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::events::list::MsgraphEventsListParams;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, calendar_refusal},
    events::render::MsgraphEventsOutput,
};

/// List the events of the default calendar, or of `--calendar` (one Graph
/// page): the lone events and the series masters, unexpanded.
///
/// JSON output: `{"events": [<raw Graph event>...], "@odata.nextLink"}`,
/// the link present when a page follows.
#[derive(Debug, Parser)]
pub struct MsgraphEventListCommand {
    /// Calendar id; omit for the default calendar.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,
    /// Maximum number of events in the page (`$top`).
    #[arg(short = 's', long, value_name = "N")]
    pub top: Option<u32>,
    /// Comma-separated properties to return (`$select`).
    #[arg(long, value_name = "CSV")]
    pub select: Option<String>,
    /// OData filter expression (`$filter`).
    #[arg(long, value_name = "EXPR")]
    pub filter: Option<String>,
}

impl MsgraphEventListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let params = MsgraphEventsListParams {
            top: self.top,
            select: self.select.as_deref(),
            filter: self.filter.as_deref(),
            ..Default::default()
        };
        let page = client
            .events_list(self.calendar.as_deref(), &params)
            .map_err(calendar_refusal)?
            .response;

        printer.out(MsgraphEventsOutput {
            events: page.value,
            next_link: page.next_link,
        })
    }
}
