//! # Event build
//!
//! The `event build` command: an event composed and printed, sent nowhere.

use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    build::IcalBuildArgs,
    event::{Event, expand},
    ical::IcalFamily,
    item::CalendarItem,
};

/// Build an event and print it, writing it nowhere.
///
/// The source and `-i` stack as they do on `event create`, and the event
/// comes back on stdout instead of going to a backend, which is how an
/// event is judged before it is sent. No account is resolved unless
/// `-i` needs the configured composer.
///
/// Given a source alone, this is also how a loose iCalendar (an
/// invitation attached to a mail) is read the way a calendar's events
/// are: zones resolved, the end computed, people listed.
///
/// JSON output: `{"contents", "method", "events": [...]}`, the raw
/// iCalendar as text, the calendar's `METHOD` and its events shaped as
/// `event read` prints them; or `{"message"}` with `-o`.
#[derive(Debug, Parser)]
pub struct EventBuildCommand {
    #[command(flatten)]
    pub args: IcalBuildArgs,
}

impl EventBuildCommand {
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
    ) -> Result<()> {
        self.args
            .execute(printer, config_paths, account_name, IcalFamily::Event)
    }
}

/// The built event, and what it projects to.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventBuildOutput {
    /// The raw iCalendar, as text.
    pub contents: String,
    /// The calendar's `METHOD` uppercased (`REQUEST`, `CANCEL`,
    /// `REPLY`...), `null` when it carries none.
    pub method: Option<String>,
    /// Every VEVENT it carries, in source order, a series once at its own
    /// start; the item id is empty, the etag `null`.
    pub events: Vec<Event>,
}

impl EventBuildOutput {
    /// Projects the built bytes.
    pub fn of(contents: String) -> Self {
        let item = CalendarItem {
            contents: contents.clone().into_bytes(),
            ..Default::default()
        };

        Self {
            method: expand::method(&item.contents),
            events: Event::project(&item),
            contents,
        }
    }
}

impl fmt::Display for EventBuildOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}
