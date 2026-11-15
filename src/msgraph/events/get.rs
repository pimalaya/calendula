//! # Event get command
//!
//! Reads a single event by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    client::{MsgraphClient, event_refusal},
    events::render::MsgraphEventOutput,
};

/// GET an event by id.
///
/// JSON output: the raw Graph event object.
#[derive(Debug, Parser)]
pub struct MsgraphEventGetCommand {
    /// Event id.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
}

impl MsgraphEventGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let event = client
            .event_get(&self.event_id, None, None)
            .map_err(event_refusal)?
            .response;
        printer.out(MsgraphEventOutput(event))
    }
}
