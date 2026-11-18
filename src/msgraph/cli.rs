//! # Microsoft Graph commands
//!
//! Dispatches the Graph-specific command tree, one subtree per Graph
//! resource.

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::msgraph::{
    calendars::cli::MsgraphCalendarsCommand, client::MsgraphClient,
    events::cli::MsgraphEventsCommand, profile::cli::MsgraphProfileCommand,
    request::MsgraphRequestCommand,
};

/// Microsoft Graph-specific API.
///
/// Nested by Graph resource (calendars, events, the signed-in user), each
/// command named after its Graph operation. Works with the raw Graph event
/// model: `create` / `update` take a Graph JSON body and `--json` prints
/// the raw Graph payload.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum MsgraphCommand {
    #[command(subcommand, alias = "calendar")]
    Calendars(MsgraphCalendarsCommand),
    #[command(subcommand, alias = "event")]
    Events(MsgraphEventsCommand),
    #[command(subcommand)]
    Profile(MsgraphProfileCommand),
    Request(MsgraphRequestCommand),
}

impl MsgraphCommand {
    pub fn execute(self, printer: &mut impl Printer, client: MsgraphClient) -> Result<()> {
        let result = match self {
            Self::Calendars(cmd) => cmd.execute(printer, client),
            Self::Events(cmd) => cmd.execute(printer, client),
            Self::Profile(cmd) => cmd.execute(printer, client),
            Self::Request(cmd) => cmd.execute(printer, client),
        };

        crate::shared::refusal::print(printer, &[crate::msgraph::backend::refusal], result)
    }
}

#[cfg(test)]
mod tests {
    use pimalaya_cli::{error::PartialOutcome, printer::CapturePrinter};

    use crate::msgraph::{
        backend::scripted::{answering, refusal},
        cli::MsgraphCommand,
        client::MsgraphClient,
        request::MsgraphRequestCommand,
    };

    /// COH-5: a request Graph refuses is the command's result, under
    /// `refusal`, coded as a failure would be, and exits 3.
    #[test]
    fn a_refused_request_is_the_result() {
        let backend = answering(&[(403, &refusal("ErrorAccessDenied"))]);
        let client = MsgraphClient::new(backend.into_client());
        let mut printer = CapturePrinter::default();

        let command = MsgraphCommand::Request(MsgraphRequestCommand {
            method: "get".into(),
            path: "me/calendars/nope".into(),
            body: None,
        });
        let err = command.execute(&mut printer, client).unwrap_err();

        assert!(err.downcast_ref::<PartialOutcome>().is_some(), "{err}");
        let output = printer.into_single().unwrap();
        assert_eq!(output["refusal"]["status"], 403, "{output}");
        assert_eq!(output["refusal"]["reason"], "ErrorAccessDenied", "{output}");
        assert_eq!(output["refusal"]["message"], "Refused", "{output}");
        assert_eq!(output["refusal"]["code"], "permission-denied", "{output}");
    }
}
