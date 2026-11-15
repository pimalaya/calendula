//! # Event update command
//!
//! Patches an event from a raw Graph event body.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, event_refusal},
    events::{input::EventJsonArg, render::MsgraphEventOutput},
};

/// Update an event from a raw Graph event JSON body (PATCH: only the
/// fields present are changed).
///
/// JSON output: the raw Graph event after the update.
#[derive(Debug, Parser)]
pub struct MsgraphEventUpdateCommand {
    /// Event id.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
    /// The raw Graph event JSON body.
    #[command(flatten)]
    pub json: EventJsonArg,
}

impl MsgraphEventUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let event = self.json.read()?;
        let updated = client
            .event_update(&self.event_id, &event)
            .map_err(event_refusal)?
            .response;

        printer.out(MsgraphEventOutput(updated))
    }
}
