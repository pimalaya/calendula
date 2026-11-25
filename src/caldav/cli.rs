//! # CalDAV CLI
//!
//! The `caldav` command family: one command per WebDAV and CalDAV method,
//! each handed a connected [`CaldavClient`].

use anyhow::Result;
use clap::Subcommand;
use pimalaya_cli::printer::Printer;

use crate::caldav::{
    client::{CaldavClient, check_calendar_id, check_item_id, check_new_calendar_id},
    delete::CaldavDeleteCommand,
    discover::CaldavDiscoverCommand,
    get::CaldavGetCommand,
    mkcalendar::CaldavMkcalendarCommand,
    propfind::CaldavPropfindCommand,
    proppatch::CaldavProppatchCommand,
    put::CaldavPutCommand,
    report::cli::CaldavReportCommand,
    rmcol::CaldavRmcolCommand,
};

/// CalDAV-specific API.
///
/// A flat list of WebDAV and CalDAV methods (RFC 4918, 4791, 6578) run
/// directly against the active account's endpoint, plus the discovery
/// chain. Each command is named after its protocol counterpart and
/// surfaces the native ids, ETags, CTags and sync tokens the unified
/// `calendar` and `event` API hides, and reaches any component kind.
#[derive(Debug, Subcommand)]
#[command(rename_all = "kebab-case")]
pub enum CaldavCommand {
    Discover(CaldavDiscoverCommand),
    #[command(aliases = ["list", "ls"])]
    Propfind(CaldavPropfindCommand),
    Proppatch(CaldavProppatchCommand),
    #[command(alias = "create")]
    Mkcalendar(CaldavMkcalendarCommand),
    Rmcol(CaldavRmcolCommand),
    #[command(subcommand)]
    Report(CaldavReportCommand),
    Get(CaldavGetCommand),
    Put(CaldavPutCommand),
    Delete(CaldavDeleteCommand),
}

impl CaldavCommand {
    /// Refuses, before anything connects, a calendar or item id no CalDAV
    /// resource can have, coded as the unified commands code it, and the
    /// id of a calendar to create that none can have.
    pub fn check(&self) -> Result<()> {
        match self {
            Self::Discover(_) => Ok(()),
            Self::Propfind(cmd) => cmd.calendar_id.as_deref().map_or(Ok(()), check_calendar_id),
            Self::Proppatch(cmd) => check_calendar_id(&cmd.id),
            Self::Mkcalendar(cmd) => check_new_calendar_id(&cmd.id),
            Self::Rmcol(cmd) => check_calendar_id(&cmd.calendar_id),
            Self::Report(cmd) => cmd.check(),
            Self::Get(cmd) => {
                check_calendar_id(&cmd.calendar_id)?;
                check_item_id(&cmd.item_id)
            }
            Self::Put(cmd) => {
                check_calendar_id(&cmd.calendar_id)?;
                check_item_id(&cmd.item_id)
            }
            Self::Delete(cmd) => {
                check_calendar_id(&cmd.calendar_id)?;
                check_item_id(&cmd.item_id)
            }
        }
    }

    pub fn execute(self, printer: &mut impl Printer, client: CaldavClient) -> Result<()> {
        let result = match self {
            Self::Discover(cmd) => cmd.execute(printer, client),
            Self::Propfind(cmd) => cmd.execute(printer, client),
            Self::Proppatch(cmd) => cmd.execute(printer, client),
            Self::Mkcalendar(cmd) => cmd.execute(printer, client),
            Self::Rmcol(cmd) => cmd.execute(printer, client),
            Self::Report(cmd) => cmd.execute(printer, client),
            Self::Get(cmd) => cmd.execute(printer, client),
            Self::Put(cmd) => cmd.execute(printer, client),
            Self::Delete(cmd) => cmd.execute(printer, client),
        };

        crate::shared::refusal::print(printer, &[crate::caldav::client::refusal], result)
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::{
        caldav::cli::CaldavCommand,
        cli::{Cli, Command},
    };

    /// `caldav delete` deletes one item, its id required, so an empty
    /// shell variable never turns it into a collection delete; `rmcol`
    /// deletes the collection.
    #[test]
    fn an_item_delete_requires_its_id_and_rmcol_deletes_the_collection() {
        assert!(Cli::try_parse_from(["calendula", "caldav", "delete", "cal"]).is_err());
        assert!(Cli::try_parse_from(["calendula", "caldav", "delete", "cal", ""]).is_ok());

        let cli = Cli::try_parse_from(["calendula", "caldav", "rmcol", "cal"]).unwrap();
        let Some(Command::Caldav(CaldavCommand::Rmcol(cmd))) = cli.cmd else {
            panic!("not a rmcol");
        };
        assert_eq!(cmd.calendar_id, "cal");
    }
}
