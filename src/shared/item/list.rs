//! # Item list
//!
//! The `item list` command, printing every iCalendar object of a calendar
//! one line each, whatever its component kind.

use std::fmt;

use anyhow::{Context, Result};
use clap::Parser;
use humansize::{BINARY, format_size};
use pimalaya_cli::{clap::args::PageTokenArg, columns::Columns, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    item::{CalendarItem, CalendarItemQuery},
    listing::Listing,
};

/// List the raw iCalendar items of a calendar.
///
/// Every component kind is listed, VEVENT included; use `event list` for the
/// events-only view with its summary and time columns.
///
/// Each line shows the entity tag, the size and the id of an item, between
/// a header naming the account and the calendar and a footer saying where
/// the page stands.
///
/// JSON output: `{"items": [{"id", "calendarId", "etag", "contents"}],
/// "nextPage"}`, `nextPage` being the number of the page following this
/// one as a string, which `--page-token` takes back, or `null` once the
/// listing is complete.
#[derive(Debug, Parser)]
pub struct ItemListCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,

    /// 1-indexed page number. Defaults to 1.
    #[arg(short, long, value_name = "N")]
    pub page: Option<u32>,

    /// Number of items per page, zero listing every one.
    #[arg(short = 's', long, value_name = "N")]
    pub page_size: Option<u32>,

    #[command(flatten)]
    pub page_token: PageTokenArg,
}

impl ItemListCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let page = match &self.page_token.token {
            Some(token) => Some(
                token
                    .parse::<u32>()
                    .with_context(|| format!("Invalid page token `{token}`"))?,
            ),
            None => self.page,
        };

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let page_size = self
            .page_size
            .or(Some(client.account.items_list_page_size()))
            .filter(|size| *size > 0);
        let page = page_size.map(|_| page.unwrap_or(1).max(1));

        let query = CalendarItemQuery::default();
        let listed = client.list_items(&calendar_id, page, page_size, query)?;

        let nouns = ["item", "items"];
        let mut listing = Listing::new(&client.name, client.backend_name(), nouns);
        listing.push("Calendar", &calendar_id);
        listing.page = page;
        listing.page_size = page_size;
        listing.count = listed.items.len();

        let next_page = match page {
            Some(page) if listed.more => Some((page + 1).to_string()),
            _ => None,
        };

        printer.out(ItemListOutput {
            listing,
            items: listed.items,
            next_page,
        })
    }
}

/// The items of one page and the number of the next.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemListOutput {
    /// The header and footer the items print between.
    #[serde(skip)]
    pub listing: Listing,
    /// The items of the page, in the order the backend listed them.
    pub items: Vec<CalendarItem>,
    /// The number of the page following this one, given back through
    /// `--page-token`, `null` once the listing is complete.
    pub next_page: Option<String>,
}

impl fmt::Display for ItemListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut columns = Columns::default();

        for item in &self.items {
            let etag = item.etag.as_deref().unwrap_or_default();

            columns.push([
                sanitize::text(etag).into_owned(),
                size_of(item),
                sanitize::text(&item.id).into_owned(),
            ]);
        }

        self.listing
            .write(f, &columns.to_string(), self.next_page.as_deref())
    }
}

/// The size of an item's octets, empty when it has no body.
///
/// A backend may list an item whose body it does not hold yet.
fn size_of(item: &CalendarItem) -> String {
    if item.contents.is_empty() {
        return String::new();
    }

    format_size(item.contents.len() as u64, BINARY)
}

#[cfg(test)]
mod tests {
    use crate::shared::{
        item::{CalendarItem, list::ItemListOutput},
        listing::Listing,
    };

    #[test]
    fn items_print_one_line_each_the_id_last() {
        let mut listing = Listing::new("perso", "caldav", ["item", "items"]);
        listing.push("Calendar", "personal");
        listing.page = Some(1);
        listing.page_size = Some(25);
        listing.count = 2;

        let output = ItemListOutput {
            listing,
            items: vec![
                CalendarItem {
                    id: "standup.ics".into(),
                    etag: Some("\"1a2b\"".into()),
                    contents: vec![b'x'; 2048],
                    ..Default::default()
                },
                CalendarItem {
                    id: "pending.ics".into(),
                    ..Default::default()
                },
            ],
            next_page: None,
        };

        let expected = "\
Account:    perso (caldav)
Calendar:   personal

\"1a2b\"  2 KiB  standup.ics
-       -      pending.ics

Page:       1, items 1 to 2, the last
";
        assert_eq!(output.to_string(), expected);
    }
}
