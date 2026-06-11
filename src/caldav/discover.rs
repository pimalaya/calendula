use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::printer::Printer;
use serde::Serialize;

use crate::caldav::client::CaldavClient;

/// Run the CalDAV discovery chain.
///
/// Walks well-known caldav -> current-user-principal -> calendar
/// home-set and prints each resolved URL.
///
/// JSON output: `{"principal", "calendar_home_set"}`.
#[derive(Debug, Parser)]
pub struct CaldavDiscoverCommand;

impl CaldavDiscoverCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let principal = client.current_user_principal()?;
        let home = client.calendar_home_set()?;

        printer.out(DiscoveryReport {
            principal: principal.to_string(),
            calendar_home_set: home.to_string(),
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DiscoveryReport {
    pub principal: String,
    pub calendar_home_set: String,
}

impl fmt::Display for DiscoveryReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Principal: {}", self.principal)?;
        writeln!(f, "Calendar home-set: {}", self.calendar_home_set)?;
        Ok(())
    }
}
