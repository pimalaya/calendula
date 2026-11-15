//! # Event list command
//!
//! Lists one page of the events of a calendar.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::events::list::MsgraphEventsListParams;
use pimalaya_cli::{clap::args::PageTokenArg, printer::Printer};

use crate::msgraph::{
    client::{MsgraphClient, calendar_refusal},
    events::render::MsgraphEventsOutput,
};

/// List the events of the default calendar, or of `--calendar` (one Graph
/// page): the lone events and the series masters, unexpanded.
///
/// JSON output: `{"events": [<raw Graph event>...], "nextPage"}`,
/// `nextPage` Graph's `@odata.nextLink`, `null` on the last page, which
/// `--page-token` takes back.
#[derive(Debug, Parser)]
pub struct MsgraphEventListCommand {
    /// Calendar id; omit for the default calendar.
    #[arg(
        short = 'k',
        long = "calendar",
        value_name = "CALENDAR-ID",
        conflicts_with = "page_token"
    )]
    pub calendar: Option<String>,
    /// Maximum number of events in the page (`$top`).
    #[arg(short = 's', long, value_name = "N", conflicts_with = "page_token")]
    pub top: Option<u32>,
    /// Comma-separated properties to return (`$select`).
    #[arg(long, value_name = "CSV", conflicts_with = "page_token")]
    pub select: Option<String>,
    /// OData filter expression (`$filter`).
    #[arg(long, value_name = "EXPR", conflicts_with = "page_token")]
    pub filter: Option<String>,
    /// The page to resume at, carrying the calendar and the query.
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphEventListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let page = match &self.page_token.token {
            Some(link) => client.events_list_from_link(link)?,
            None => {
                let params = MsgraphEventsListParams {
                    top: self.top,
                    select: self.select.as_deref(),
                    filter: self.filter.as_deref(),
                    ..Default::default()
                };

                client
                    .events_list(self.calendar.as_deref(), &params)
                    .map_err(calendar_refusal)?
            }
        }
        .response;

        printer.out(MsgraphEventsOutput {
            events: page.value,
            next_page: page.next_link,
        })
    }
}
