//! # Calendar list command
//!
//! Lists the calendars of the user.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::calendars::list::MsgraphCalendarsListParams;
use pimalaya_cli::{clap::args::PageTokenArg, printer::Printer};

use crate::msgraph::{calendars::render::MsgraphCalendarsOutput, client::MsgraphClient};

/// List the calendars of the user (one Graph page).
///
/// JSON output: `{"calendars": [<raw Graph calendar>...], "nextPage"}`,
/// `nextPage` Graph's `@odata.nextLink`, `null` on the last page, which
/// `--page-token` takes back.
#[derive(Debug, Parser)]
pub struct MsgraphCalendarListCommand {
    /// Maximum number of calendars in the page (`$top`).
    #[arg(
        long,
        short_alias = 's',
        value_name = "N",
        conflicts_with = "page_token"
    )]
    pub top: Option<u32>,
    /// The page to resume at, carrying the query.
    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl MsgraphCalendarListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let page = match &self.page_token.token {
            Some(link) => client.page_get(link, None)?,
            None => {
                let params = MsgraphCalendarsListParams {
                    top: self.top,
                    ..Default::default()
                };

                client.calendars_list(&params)?
            }
        }
        .response;

        printer.out(MsgraphCalendarsOutput {
            calendars: page.value,
            next_page: page.next_link,
        })
    }
}
