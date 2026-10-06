//! # Event create
//!
//! The `event create` command, storing one VEVENT into the selected
//! calendar, from an iCalendar source, from field flags, from the
//! composer, or from any combination of the three.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::{CalendarIdArg, IcalComposerArgs},
    client::CalendarClient,
    composer::IcalComposer,
    event::fields::EventFieldsArgs,
    ical::{IcalFamily, blank_item, ensure_valid, read_source},
};

/// Create a new event from an iCalendar source.
///
/// The source, the field flags and `-i` stack: the source is the event
/// to start from, the flags set the properties they name on it, and `-i`
/// opens the result in the composer. With no source the event is minted
/// fresh, carrying nothing but a `UID`, a `DTSTAMP` and a `DTSTART` of
/// now.
///
/// The flags cover the common fields; a recurrence, an alarm, a
/// conference and a hand-written `VTIMEZONE` are left to the composer,
/// the complete surface. An event built from flags alone is checked
/// before it is written; a source goes to the backend as written.
///
/// The bytes are stored as given, so they carry the UID the event is
/// addressed by afterwards.
///
/// JSON output: `{"id"}`, the identifier the backend assigned.
#[derive(Debug, Parser)]
pub struct EventCreateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,
    /// The composer the event is refined in before it is written.
    #[command(flatten)]
    pub composer: IcalComposerArgs,
    /// The properties the command sets on the event.
    #[command(flatten)]
    pub fields: EventFieldsArgs,
    /// iCalendar to store: a path to a file, raw iCalendar contents, or
    /// `-` for stdin. Omit to mint one.
    #[arg(value_name = "ICAL")]
    pub ical: Option<String>,
}

impl EventCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        if self.ical.is_none() && self.fields.is_empty() && !self.composer.interactive {
            bail!("Nothing to create; give an iCalendar, a field flag, or -i to compose one");
        }

        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        let base = match &self.ical {
            Some(source) => read_source(source)?,
            None => blank_item(IcalFamily::Event)?,
        };

        let seed = self.fields.apply(&base)?;

        if !self.composer.interactive {
            // NOTE: only an event calendula minted is checked. An
            // iCalendar given on the command line goes to the backend as
            // it was written, which is what the projections promise too.
            if self.ical.is_none() {
                ensure_valid(&seed)?;
            }

            let id = client.create_item(&calendar_id, seed)?;
            return printer.out(EventCreateOutput::Created(EventCreatedOutput { id }));
        }

        let composer = IcalComposer {
            command: client.account.item_composer(self.composer.composer)?,
        };

        // NOTE: nothing has reached the network yet, the client opening
        // on the first call that needs it, so the editor runs with no
        // connection held and the create below opens one.
        let Some(draft) = composer.edit(printer, &seed)? else {
            return printer.out(EventCreateOutput::Abandoned);
        };

        let created = client.create_item(&calendar_id, draft.contents.clone());
        let id = draft.finish(created)?;

        printer.out(EventCreateOutput::Created(EventCreatedOutput { id }))
    }
}

/// What `event create` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum EventCreateOutput {
    /// The event the backend created.
    Created(EventCreatedOutput),
    /// Nothing, the edit having been abandoned.
    Abandoned,
}

impl fmt::Display for EventCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created(out) => out.fmt(f),
            Self::Abandoned => writeln!(f, "Event not created"),
        }
    }
}

/// The event the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventCreatedOutput {
    /// Backend-assigned identifier of the new event.
    pub id: String,
}

impl fmt::Display for EventCreatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Event `{}` successfully created", self.id)
    }
}
