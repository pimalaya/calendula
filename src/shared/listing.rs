//! # Listing
//!
//! The frame a paged listing prints its rows in, as Himalaya prints its
//! envelopes: a header naming where the rows come from, a blank line, the
//! rows, another blank line, then a footer saying where the page stands
//! and how to reach the next one. `event list`, `event find`, `item list`
//! and `gcal instances` print through it.

use std::fmt;

use pimalaya_cli::sanitize;

/// The header and footer around the rows of a listing.
#[derive(Clone, Debug, Default)]
pub struct Listing {
    /// The fields above the rows, their values defused.
    pub header: Vec<(&'static str, String)>,
    /// The page listed, `None` when the listing is not paged.
    pub page: Option<u32>,
    /// The size of a page, `None` when the listing is not paged.
    pub page_size: Option<u32>,
    /// How many items the page holds.
    pub count: usize,
    /// What the footer calls one item, then several.
    pub nouns: [&'static str; 2],
}

impl Listing {
    /// A listing of the account `name` read through `backend`, the header
    /// naming both.
    pub fn new(name: &str, backend: &str, nouns: [&'static str; 2]) -> Self {
        let account = format!("{} ({backend})", sanitize::text(name));

        Self {
            header: vec![("Account", account)],
            nouns,
            ..Default::default()
        }
    }

    /// Adds a field to the header, its value defused.
    pub fn push(&mut self, label: &'static str, value: &str) {
        self.header
            .push((label, sanitize::text(value).into_owned()));
    }

    /// Writes the header, the `rows` and the footer, `next_page` being
    /// the page following this one.
    pub fn write(
        &self,
        f: &mut fmt::Formatter<'_>,
        rows: &str,
        next_page: Option<u32>,
    ) -> fmt::Result {
        write_fields(f, &self.header)?;
        writeln!(f)?;

        if !rows.is_empty() {
            f.write_str(rows)?;
            writeln!(f)?;
        }

        write_fields(f, &self.footer(next_page))
    }

    /// The fields under the rows: the page with the items it holds, and
    /// the flag reaching the next page while one follows.
    fn footer(&self, next_page: Option<u32>) -> Vec<(&'static str, String)> {
        let [one, several] = self.nouns;
        let count = self.count as u64;
        let (page, first) = match (self.page, self.page_size) {
            (Some(page), Some(size)) => {
                let first = u64::from(page.max(1) - 1) * u64::from(size) + 1;
                (page.to_string(), first)
            }
            _ => (String::from("all"), 1),
        };

        let range = match count {
            0 => String::from("empty"),
            1 => format!("{one} {first}"),
            n => format!("{several} {first} to {}", first + n - 1),
        };

        let last = match (self.page, next_page) {
            (Some(_), None) if count > 0 => ", the last",
            _ => "",
        };

        let mut fields = vec![("Page", format!("{page}, {range}{last}"))];

        if let Some(next) = next_page {
            fields.push(("Next page", format!("--page {next}")));
        }

        fields
    }
}

/// Writes `Label:  value` lines, values in one column on every page: two
/// spaces past the longest label, `Next page:`.
fn write_fields(f: &mut fmt::Formatter<'_>, fields: &[(&str, String)]) -> fmt::Result {
    const WIDTH: usize = "Next page:".len() + 2;

    for (label, value) in fields {
        let label = format!("{label}:");
        writeln!(f, "{label:<WIDTH$}{value}")?;
    }

    Ok(())
}
