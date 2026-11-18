//! # Event get command
//!
//! Reads a single event by id.

use anyhow::Result;
use clap::Parser;
use io_msgraph::v1::rest::users::events::MsgraphEvent;
use pimalaya_cli::printer::Printer;

use crate::{
    msgraph::{
        client::{MsgraphClient, coded, event_refusal},
        events::render::MsgraphEventOutput,
    },
    shared::raw_json::RawRead,
};

/// GET an event by id.
///
/// JSON output: the Graph event object as Graph sent it.
#[derive(Debug, Parser)]
pub struct MsgraphEventGetCommand {
    /// Event id.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
}

impl MsgraphEventGetCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        let value = client
            .raw_get(&["events", &self.event_id])
            .map_err(|err| coded(err, event_refusal))?;
        let text = serde_json::from_value::<MsgraphEvent>(value.clone())
            .map(|event| MsgraphEventOutput(event).to_string())
            .unwrap_or_default();

        printer.out(RawRead { value, text })
    }
}
