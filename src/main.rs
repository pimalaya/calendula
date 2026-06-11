mod account;
mod backend;
#[cfg(feature = "caldav")]
mod caldav;
mod cli;
mod config;
mod shared;
#[cfg(feature = "vdir")]
mod vdir;
mod wizard;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use crate::cli::CalendulaCli;

fn main() {
    let cli = CalendulaCli::parse();
    let mut printer = StdoutPrinter::new(&cli.json);
    let result = execute(cli, &mut printer);
    ErrorReport::eval(&mut printer, result);
}

fn execute(cli: CalendulaCli, printer: &mut StdoutPrinter) -> Result<()> {
    Logger::try_init(&cli.log)?;
    let config = cli.config_paths.as_ref();
    let account = cli.account.name.as_deref();
    let backend = cli.backend;
    cli.command.execute(printer, config, account, backend)
}
