//! # Query report command
//!
//! Runs the RFC 4791 `calendar-query` REPORT and renders the matching
//! items.

use anyhow::Result;
use chrono::NaiveDate;
use clap::Parser;
use io_webdav::rfc4918::{escape_attr, escape_text};
use jiff::tz::TimeZone;
use pimalaya_cli::printer::Printer;

use crate::{
    caldav::{
        backend::unreadable_row,
        client::CaldavClient,
        report::entries::{CaldavItemEntriesOutput, EntryRow},
    },
    error::partial_when,
    shared::{event::list::local_zone, item::CalendarTimeRange},
};

/// The component a window or a prop-filter narrows when no `--component`
/// names one: calendula is the events CLI.
const DEFAULT_COMPONENT: &str = "VEVENT";

/// Query the items of a calendar (RFC 4791 §7.8).
///
/// The `calendar-query` REPORT. Without filter flags every item comes
/// back. `--component` keeps one component kind; `--from` and `--to`
/// narrow it to a window of inclusive days (a `time-range`); each other
/// filter flag adds one prop-filter, which the server evaluates, every one
/// having to match: `--match SUMMARY standup --not-defined RRULE`. A text
/// match is a substring match (RFC 4791 §9.7.5). A window or a prop-filter
/// given without `--component` filters VEVENT. Param-filters, nested
/// components and anything else the flags cannot express go through
/// `report raw`.
///
/// JSON output: `{"items": [{"id", "etag", "contents", "encoding"}],
/// "truncated", "errors": [{"id", "code", "error", "sources"}]}`: the error
/// of each item the server answered without its data, `[]` when none, the
/// command then exiting 3. A truncated result is `truncated`, no error.
#[derive(Debug, Parser)]
pub struct CaldavReportQueryCommand {
    /// Identifier of the calendar to query.
    #[arg(value_name = "CALENDAR")]
    pub calendar_id: String,
    /// Keep the items holding this component kind (VEVENT, VTODO,
    /// VJOURNAL).
    #[arg(long, value_name = "KIND")]
    pub component: Option<String>,
    /// Keep the components overlapping this day or a later one.
    #[arg(long, value_name = "DATE")]
    pub from: Option<NaiveDate>,
    /// Keep the components overlapping this day or an earlier one.
    #[arg(long, value_name = "DATE")]
    pub to: Option<NaiveDate>,
    /// Match a property value holding a text.
    #[arg(long = "match", num_args = 2, value_names = ["PROP", "VALUE"])]
    pub matches: Vec<String>,
    /// Match a property value not holding a text.
    #[arg(long = "not-match", num_args = 2, value_names = ["PROP", "VALUE"])]
    pub not_matches: Vec<String>,
    /// Require a property to be present.
    #[arg(long = "defined", value_name = "PROP")]
    pub defined: Vec<String>,
    /// Require a property to be absent.
    #[arg(long = "not-defined", value_name = "PROP")]
    pub not_defined: Vec<String>,
    /// Collation of every match, the server's `i;ascii-casemap` when
    /// omitted.
    #[arg(long, value_name = "NAME")]
    pub collation: Option<String>,
}

impl CaldavReportQueryCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let filter = self.comp_filter(&local_zone())?;
        let ok = client.list_items(&self.calendar_id, &filter)?;

        let output = CaldavItemEntriesOutput {
            rows: ok.items.into_iter().map(EntryRow::from).collect(),
            truncated: ok.truncated,
            errors: ok.unreadable.into_iter().map(unreadable_row).collect(),
        };

        printer.out(&output)?;
        partial_when(&output.errors)
    }

    /// The VCALENDAR child filter the flags build: one component filter
    /// holding the window, then one prop-filter per filter flag (RFC 4791
    /// §9.7.1), or none at all.
    fn comp_filter(&self, zone: &TimeZone) -> Result<String> {
        let mut inner = String::new();

        if let Some(range) = CalendarTimeRange::from_days_in(self.from, self.to, zone)? {
            inner.push_str(&range.to_caldav_filter());
        }

        let matches = self.matches.as_chunks::<2>().0.iter();
        let not_matches = self.not_matches.as_chunks::<2>().0.iter();
        for ([name, value], negate) in matches
            .map(|args| (args, false))
            .chain(not_matches.map(|args| (args, true)))
        {
            let mut text_match = String::from("<C:text-match");

            if let Some(collation) = &self.collation {
                let collation = escape_attr(collation);
                text_match.push_str(&format!(" collation=\"{collation}\""));
            }

            if negate {
                text_match.push_str(" negate-condition=\"yes\"");
            }

            let value = escape_text(value);
            inner.push_str(&prop_filter(
                name,
                &format!("{text_match}>{value}</C:text-match>"),
            ));
        }

        for name in &self.defined {
            inner.push_str(&prop_filter(name, ""));
        }

        for name in &self.not_defined {
            inner.push_str(&prop_filter(name, "<C:is-not-defined/>"));
        }

        let component = match &self.component {
            Some(component) => component.to_ascii_uppercase(),
            None if inner.is_empty() => return Ok(String::new()),
            None => String::from(DEFAULT_COMPONENT),
        };

        let component = escape_attr(&component);
        Ok(format!(
            "<C:comp-filter name=\"{component}\">{inner}</C:comp-filter>"
        ))
    }
}

/// A prop-filter of the property `name` holding `test`, an empty one
/// matching when the property is defined.
fn prop_filter(name: &str, test: &str) -> String {
    let name = escape_attr(&name.to_ascii_uppercase());

    match test.is_empty() {
        true => format!("<C:prop-filter name=\"{name}\"/>"),
        false => format!("<C:prop-filter name=\"{name}\">{test}</C:prop-filter>"),
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use jiff::tz::TimeZone;

    use super::CaldavReportQueryCommand;

    fn filter(args: &[&str]) -> String {
        let argv = ["query", "cal"].iter().chain(args);
        let cmd = CaldavReportQueryCommand::try_parse_from(argv).unwrap();
        cmd.comp_filter(&TimeZone::UTC).unwrap()
    }

    /// No flag asks for every item, a bare component keeps its kind.
    #[test]
    fn no_flag_asks_for_every_item() {
        assert_eq!(filter(&[]), "");
        assert_eq!(
            filter(&["--component", "vtodo"]),
            "<C:comp-filter name=\"VTODO\"></C:comp-filter>"
        );
    }

    /// The window comes first, then one prop-filter per flag, in VEVENT
    /// when no component is named, every value escaped.
    #[test]
    fn maps_each_flag_onto_the_component_filter() {
        let built = filter(&[
            "--from",
            "2026-11-16",
            "--to",
            "2026-11-16",
            "--match",
            "summary",
            "R&D <x>",
            "--not-match",
            "LOCATION",
            "home",
            "--defined",
            "ATTENDEE",
            "--not-defined",
            "RRULE",
            "--collation",
            "i;octet",
        ]);

        assert_eq!(
            built,
            "<C:comp-filter name=\"VEVENT\">\
             <C:time-range start=\"20261116T000000Z\" end=\"20261117T000000Z\" />\
             <C:prop-filter name=\"SUMMARY\">\
             <C:text-match collation=\"i;octet\">R&amp;D &lt;x&gt;</C:text-match>\
             </C:prop-filter>\
             <C:prop-filter name=\"LOCATION\">\
             <C:text-match collation=\"i;octet\" negate-condition=\"yes\">home</C:text-match>\
             </C:prop-filter>\
             <C:prop-filter name=\"ATTENDEE\"/>\
             <C:prop-filter name=\"RRULE\"><C:is-not-defined/></C:prop-filter>\
             </C:comp-filter>"
        );
    }

    /// A crossed window is refused by name.
    #[test]
    fn refuses_a_crossed_window() {
        let argv = ["query", "cal", "--from", "2026-11-17", "--to", "2026-11-16"];
        let cmd = CaldavReportQueryCommand::try_parse_from(argv).unwrap();
        assert!(
            cmd.comp_filter(&TimeZone::UTC)
                .unwrap_err()
                .to_string()
                .contains("--to")
        );
    }
}
