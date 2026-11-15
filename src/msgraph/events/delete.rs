//! # Event delete command
//!
//! Deletes an event by id.

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;

use crate::{
    msgraph::client::{MsgraphClient, event_refusal},
    shared::receipt::IdReceipt,
};

/// Delete an event by id, the whole series for a series master.
///
/// JSON output: `{"id"}`.
#[derive(Debug, Parser)]
pub struct MsgraphEventDeleteCommand {
    /// Event id.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
}

impl MsgraphEventDeleteCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: MsgraphClient) -> Result<()> {
        client.event_delete(&self.event_id).map_err(event_refusal)?;

        printer.out(IdReceipt {
            id: self.event_id,
            done: "Event deleted",
        })
    }
}
