//! # Item add
//!
//! The `item add` command, storing one iCalendar object into the selected
//! calendar.

use core::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    capability::Capability,
    shared::{
        arg::{CalendarIdArg, IcalComposerArgs},
        client::CalendarClient,
        composer::IcalComposer,
        ical::{IcalFamily, blank_item, read_source},
        item::ItemOrigin,
    },
};

/// Add a new iCalendar item to a calendar, from an iCalendar source.
///
/// The source and `-i` stack: the source is the item to start from,
/// and `-i` opens it in the composer. `item` names no component kind,
/// so it mints nothing: with no source the command bails, naming
/// `event` for an event, unfog for a to-do and journal for a journal
/// entry.
///
/// The bytes are stored as given, so they carry the UID the item is
/// addressed by afterwards.
///
/// JSON output: `{"id", "queued"}`, the identifier the backend assigned
/// and whether a sync engine applies the write later.
#[derive(Debug, Parser)]
pub struct ItemCreateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,
    /// The composer the item is refined in before it is written.
    #[command(flatten)]
    pub composer: IcalComposerArgs,
    /// iCalendar to store: a path to a file, raw iCalendar contents, or
    /// `-` for stdin. Required: `item` mints nothing.
    #[arg(value_name = "ICAL")]
    pub ical: Option<String>,
}

impl ItemCreateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: &mut CalendarClient) -> Result<()> {
        client.require(&[Capability::ItemAdd])?;

        let seed = match &self.ical {
            Some(source) => read_source(source)?,
            None => blank_item(IcalFamily::Item)?,
        };

        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let origin = ItemOrigin::of_source(self.ical.as_deref());

        if !self.composer.interactive {
            let id = client.create_item(&calendar_id, seed, origin, None)?;
            return printer.out(ItemCreateOutput::Created(ItemCreatedOutput {
                id,
                queued: client.queues_writes(),
            }));
        }

        let composer = IcalComposer {
            command: client.account.item_composer(self.composer.composer)?,
        };

        // NOTE: nothing has reached the network yet, the client opening
        // on the first call that needs it, so the editor runs with no
        // connection held and the create below opens one.
        let Some(draft) = composer.edit(printer, &seed)? else {
            return printer.out(ItemCreateOutput::Abandoned {});
        };

        let created = client.create_item(&calendar_id, draft.contents.clone(), origin, None);
        let id = draft.finish(created)?;

        printer.out(ItemCreateOutput::Created(ItemCreatedOutput {
            id,
            queued: client.queues_writes(),
        }))
    }
}

/// What `item add` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ItemCreateOutput {
    /// The item the backend created.
    Created(ItemCreatedOutput),
    /// Nothing, the edit having been abandoned: an empty object rather
    /// than null, so every payload of the command is a JSON object.
    Abandoned {},
}

impl fmt::Display for ItemCreateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created(out) => out.fmt(f),
            Self::Abandoned {} => writeln!(f, "Item not created"),
        }
    }
}

/// The item the backend created.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemCreatedOutput {
    /// Backend-assigned identifier of the new item; for a queued write, the
    /// link id it was staged under, which no command addresses before a
    /// sync applies it.
    pub id: String,
    /// Whether a sync engine applies the write later rather than now.
    pub queued: bool,
}

impl fmt::Display for ItemCreatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = sanitize::text(&self.id);

        match self.queued {
            true => writeln!(f, "Item `{id}` queued for the next sync"),
            false => writeln!(f, "Item `{id}` successfully created"),
        }
    }
}
