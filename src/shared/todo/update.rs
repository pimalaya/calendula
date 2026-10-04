//! # Todo update
//!
//! The `todo update` command, replacing one VTODO of the selected
//! calendar.

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
    ical::read_source,
};

/// Overwrite an existing todo from an iCalendar source.
///
/// The source and `-i` stack: the source replaces the todo's bytes,
/// and `-i` opens the result in the composer. The whole component is
/// replaced, so the source has to carry every property the todo keeps:
/// what it omits is dropped.
///
/// With no source the todo is read from the backend first, and the
/// version it answered guards the write.
///
/// JSON output: `{"id"}`, the todo the backend updated.
#[derive(Debug, Parser)]
pub struct TodoUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,
    /// Gate the update on this ETag, as returned by a previous read.
    ///
    /// The update only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    /// With no source the command sends the ETag it read when this is
    /// omitted.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
    /// The composer the todo is refined in before it is written.
    #[command(flatten)]
    pub composer: IcalComposerArgs,
    /// Todo to update, as `todo list` reports it.
    #[arg(value_name = "TODO-ID")]
    pub todo_id: String,
    /// iCalendar replacing the current contents: a path to a file, raw
    /// iCalendar contents, or `-` for stdin. Omit to start from the
    /// stored todo.
    #[arg(value_name = "ICAL")]
    pub ical: Option<String>,
}

impl TodoUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        if self.ical.is_none() && !self.composer.interactive {
            bail!("Nothing to update; give an iCalendar, or -i to edit the todo");
        }

        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        // NOTE: reading the stored bytes also gives the version they are
        // being changed from, so an edit that took a minute cannot
        // silently overwrite a write that landed during it.
        let (seed, etag) = match &self.ical {
            Some(source) => (read_source(source)?, None),
            None => {
                let item = client.get_item(&calendar_id, &self.todo_id)?;
                (item.contents, item.etag)
            }
        };

        let if_match = self.if_match.or(etag);

        if !self.composer.interactive {
            client.update_item(&calendar_id, &self.todo_id, seed, if_match.as_deref())?;

            return printer.out(TodoUpdateOutput::Applied(TodoUpdatedOutput {
                id: self.todo_id,
            }));
        }

        let composer = IcalComposer {
            command: client.account.item_composer(self.composer.composer)?,
        };

        // NOTE: reading the todo is the one thing that has to happen
        // before the editor, so its connection is dropped here rather
        // than left idle for the minutes an edit takes: a server closes
        // such a connection, and the update below opens a fresh one.
        client.disconnect();

        let Some(draft) = composer.edit(printer, &seed)? else {
            return printer.out(TodoUpdateOutput::Abandoned);
        };

        let updated = client.update_item(
            &calendar_id,
            &self.todo_id,
            draft.contents.clone(),
            if_match.as_deref(),
        );

        draft.finish(updated)?;

        printer.out(TodoUpdateOutput::Applied(TodoUpdatedOutput {
            id: self.todo_id,
        }))
    }
}

/// What `todo update` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum TodoUpdateOutput {
    /// The todo the backend updated.
    Applied(TodoUpdatedOutput),
    /// Nothing, the edit having been abandoned.
    Abandoned,
}

impl fmt::Display for TodoUpdateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Applied(out) => out.fmt(f),
            Self::Abandoned => writeln!(f, "Todo not updated"),
        }
    }
}

/// The todo the backend updated.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TodoUpdatedOutput {
    /// Backend-specific identifier of the updated todo.
    pub id: String,
}

impl fmt::Display for TodoUpdatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Todo `{}` successfully updated", self.id)
    }
}
