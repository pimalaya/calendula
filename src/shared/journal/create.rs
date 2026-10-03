//! # Journal create
//!
//! The `journal create` command, storing one VJOURNAL into the selected
//! calendar.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::note::Noted;
use crate::shared::{
    arg::{CalendarIdArg, IcalComposerArgs},
    client::CalendarClient,
    composer::IcalComposer,
    ical::{IcalFamily, blank_item, read_source},
};

/// Create a new journal entry from an iCalendar source.
///
/// The source and `-i` stack: the source is the journal entry to start from,
/// and `-i` opens it in the composer. With no source the entry is minted fresh, carrying nothing but a `UID` and a `DTSTAMP`.
///
/// The bytes are stored as given, so they carry the UID the journal entry is
/// addressed by afterwards.
///
/// JSON output: `{"id"}`, the identifier the backend assigned.
#[derive(Debug, Parser)]
pub struct JournalCreateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,
    /// The composer the journal entry is refined in before it is written.
    #[command(flatten)]
    pub composer: IcalComposerArgs,
    /// iCalendar to store: a path to a file, raw iCalendar contents, or
    /// `-` for stdin. Omit to mint one.
    #[arg(value_name = "ICAL")]
    pub ical: Option<String>,
}

impl JournalCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        if self.ical.is_none() && !self.composer.interactive {
            bail!("Nothing to create; give an iCalendar, or -i to compose one");
        }

        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        let seed = match &self.ical {
            Some(source) => read_source(source)?,
            None => blank_item(IcalFamily::Journal)?,
        };

        if !self.composer.interactive {
            let id = client.create_item(&calendar_id, seed)?;
            return printer.out(Noted {
                output: JournalCreateOutput::Created(JournalCreatedOutput { id }),
                notes: client.take_notes(),
            });
        }

        let composer = IcalComposer {
            command: client.account.item_composer(self.composer.composer)?,
        };

        // NOTE: nothing has reached the network yet, the client opening
        // on the first call that needs it, so the editor runs with no
        // connection held and the create below opens one.
        let Some(draft) = composer.edit(printer, &seed)? else {
            return printer.out(JournalCreateOutput::Abandoned);
        };

        let created = client.create_item(&calendar_id, draft.contents.clone());
        let id = draft.finish(created)?;

        printer.out(Noted {
            output: JournalCreateOutput::Created(JournalCreatedOutput { id }),
            notes: client.take_notes(),
        })
    }
}

/// What `journal create` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum JournalCreateOutput {
    /// The journal entry the backend created.
    Created(JournalCreatedOutput),
    /// Nothing, the edit having been abandoned.
    Abandoned,
}

impl fmt::Display for JournalCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created(out) => out.fmt(f),
            Self::Abandoned => writeln!(f, "Journal not created"),
        }
    }
}

/// The journal entry the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JournalCreatedOutput {
    /// Backend-assigned identifier of the new journal entry.
    ///
    /// On pimdir this is the link id the queued create was staged under,
    /// the store having no id of its own until a sync applies it.
    pub id: String,
}

impl fmt::Display for JournalCreatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Journal `{}` successfully created", self.id)
    }
}
