//! # Item list
//!
//! The `item list` command, printing every iCalendar object of a calendar
//! one line each, whatever its component kind.

use std::fmt;

use anyhow::{Context, Result};
use clap::Parser;
use humansize::{BINARY, format_size};
use pimalaya_cli::{
    clap::args::PageTokenArg,
    listing::{Listing, ListingPage},
    printer::Printer,
    sanitize,
};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    error::{self, ItemError},
    shared::{
        arg::CalendarIdArg,
        client::CalendarClient,
        item::{CalendarItem, CalendarItemQuery},
    },
};

/// List the raw iCalendar items of a calendar.
///
/// Every component kind is listed, VEVENT included; use `event list` for the
/// events-only view with its summary and time columns.
///
/// Each line shows the entity tag, the size and the id of an item, under a
/// header naming the account, the calendar and where the page stands. An
/// item the backend could not read is named by an `Error:` line at the
/// end, and the command exits 3.
///
/// JSON output: `{"items": [{"id", "calendarId", "etag", "contents"}],
/// "nextPage", "errors"}`, `nextPage` being the number of the page
/// following this one as a string, which `--page-token` takes back, or
/// `null` once the listing is complete, and `errors` naming each item it
/// could not read, and a listing the server cut short by a `null` id:
/// `[{"id", "code", "error", "sources"}]`.
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
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemList])?;

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

        let next_page = match page {
            Some(page) if listed.more => Some((page + 1).to_string()),
            _ => None,
        };

        let mut listing = Listing::new(&client.name, Some(client.backend_name()));
        listing.head("Calendar", &calendar_id);
        let held = listing_page(page, page_size);
        let count = listed.items.len();
        listing.page(held, count, ["item", "items"], next_page.as_deref());

        let output = ItemListOutput {
            listing,
            items: listed.items,
            next_page,
            errors: listed.errors,
        };

        printer.out(&output)?;
        error::partial_when(&output.errors)
    }
}

/// The page an item listing holds: the numbered `page` of `size` items,
/// the whole listing when either is unset.
pub(crate) fn listing_page(page: Option<u32>, size: Option<u32>) -> ListingPage {
    match (page, size) {
        (Some(page), Some(size)) => ListingPage::Numbered {
            page: page.into(),
            size: size.into(),
        },
        _ => ListingPage::All,
    }
}

/// The items of one page and the number of the next.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemListOutput {
    /// The header the items print under, its page placed.
    #[serde(skip)]
    pub listing: Listing<'static>,
    /// The items of the page, in the order the backend listed them.
    pub items: Vec<CalendarItem>,
    /// The number of the page following this one, given back through
    /// `--page-token`, `null` once the listing is complete.
    pub next_page: Option<String>,
    /// The error of each item the page could not read, in the order met.
    pub errors: Vec<ItemError>,
}

impl fmt::Display for ItemListOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut listing = self.listing.clone();

        for item in &self.items {
            let etag = item.etag.as_deref().unwrap_or_default();

            listing.items.push([
                sanitize::text(etag).into_owned(),
                size_of(item),
                sanitize::text(&item.id).into_owned(),
            ]);
        }

        for error in &self.errors {
            listing.errors.push(error);
        }

        write!(f, "{listing}")
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
    use std::io;

    use anyhow::anyhow;
    use pimalaya_cli::listing::{Listing, ListingPage};

    use crate::{
        error,
        shared::item::{CalendarItem, list::ItemListOutput},
    };

    #[test]
    fn items_print_one_line_each_the_id_last() {
        let mut listing = Listing::new("perso", Some("caldav"));
        listing.head("Calendar", "personal");
        let page = ListingPage::Numbered { page: 1, size: 25 };
        listing.page(page, 2, ["item", "items"], None);

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
            errors: Vec::new(),
        };

        let expected = "\
Account:    perso (caldav)
Calendar:   personal
Page:       1, items 1 to 2, the last

\"1a2b\"  2 KiB  standup.ics
-       -      pending.ics
";
        assert_eq!(output.to_string(), expected);
    }

    /// An item the backend could not read prints as an `Error:` line last,
    /// built as vdir and pimdir build their entries.
    #[test]
    fn an_unread_item_prints_last() {
        let mut listing = Listing::new("perso", Some("pimdir"));
        listing.head("Calendar", "personal");
        let page = ListingPage::Numbered { page: 2, size: 1 };
        listing.page(page, 1, ["item", "items"], Some("3"));

        let denied = io::Error::from(io::ErrorKind::PermissionDenied);
        let file = anyhow::Error::from(denied).context("Cannot read item file /home/c.ics");
        let blob = anyhow!("Body blob missing for item `7` in calendar `personal`");

        let output = ItemListOutput {
            listing,
            items: vec![CalendarItem {
                id: "7".into(),
                ..Default::default()
            }],
            next_page: Some(String::from("3")),
            errors: vec![
                error::unreadable(Some("c"), file),
                error::unreadable(Some("7"), blob),
            ],
        };

        let expected = "\
Account:    perso (pimdir)
Calendar:   personal
Page:       2, item 2
Next page:  --page 3

-  -  7

Error:  c  item-unreadable  Cannot read item file /home/c.ics
Error:  7  item-unreadable  Body blob missing for item `7` in calendar `personal`
";
        assert_eq!(output.to_string(), expected);
    }
}
