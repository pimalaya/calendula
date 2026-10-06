//! # Item build
//!
//! The pipeline the `build` verb of every component family runs: an
//! iCalendar source, set by the family's field flags, refined in the
//! composer and printed, rather than written to a backend.

use core::fmt;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use pimalaya_cli::{
    clap::parsers::path_parser,
    printer::{Message, Printer},
};
use pimalaya_config::command::CommandConfig;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    account::context::Account,
    cli::resolve_account,
    shared::{
        arg::IcalComposerArgs,
        composer::IcalComposer,
        event::build::EventBuildOutput,
        ical::{IcalFamily, blank_item, ensure_valid, read_source},
    },
};

/// What a family sets on the item between its source and the composer.
///
/// Only `event` has field flags; the other families pass [`NoFields`].
pub trait IcalFields {
    /// Whether nothing is set, the source then passing through.
    fn is_empty(&self) -> bool;

    /// Writes the fields onto `item`, returning its new bytes.
    fn apply(&self, item: &[u8]) -> Result<Vec<u8>>;
}

/// The fields of a family taking no flag.
pub struct NoFields;

impl IcalFields for NoFields {
    fn is_empty(&self) -> bool {
        true
    }

    fn apply(&self, item: &[u8]) -> Result<Vec<u8>> {
        Ok(item.to_vec())
    }
}

/// What a `build` takes, whichever family it belongs to.
#[derive(Debug, Parser)]
pub struct IcalBuildArgs {
    /// The composer the item is refined in before it is printed.
    #[command(flatten)]
    pub composer: IcalComposerArgs,
    /// Write the item here instead of printing it.
    ///
    /// The composer inherits stdout, so `-i` cannot be redirected: this
    /// is how an item refined in an editor is captured.
    #[arg(short, long, value_name = "PATH", value_parser = path_parser)]
    pub output: Option<PathBuf>,
    /// iCalendar to build on: a path to a file, raw iCalendar contents,
    /// or `-` for stdin.
    ///
    /// Omit to mint one, which only a family naming a component kind can
    /// do: `item` mints nothing.
    #[arg(value_name = "ICAL")]
    pub ical: Option<String>,
}

impl IcalBuildArgs {
    /// Builds the item and hands it over, reaching no backend.
    pub fn execute(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        family: IcalFamily,
    ) -> Result<()> {
        self.execute_with(printer, config_paths, account_name, family, &NoFields)
    }

    /// Builds the item as [`execute`](Self::execute) does, `fields` set on
    /// it between the source and the composer.
    pub fn execute_with(
        self,
        printer: &mut impl Printer,
        config_paths: &[PathBuf],
        account_name: Option<&str>,
        family: IcalFamily,
        fields: &impl IcalFields,
    ) -> Result<()> {
        let Self {
            composer,
            output,
            ical,
        } = self;

        if ical.is_none() && fields.is_empty() && !composer.interactive {
            if family == IcalFamily::Event {
                bail!("Nothing to build; give an iCalendar, a field flag, or -i to compose one");
            }

            bail!("Nothing to build; give an iCalendar, or -i to compose one");
        }

        let base = match &ical {
            Some(source) => read_source(source)?,
            None => blank_item(family)?,
        };

        let seed = fields.apply(&base)?;

        if !composer.interactive {
            // NOTE: what a create checks, a build checks too. A built item
            // reaches `create` as a source, which goes to the backend as
            // it was written, so a laxer check here would be a way around
            // that one.
            if ical.is_none() {
                ensure_valid(&seed)?;
            }

            return emit(printer, output.as_deref(), family, &seed);
        }

        let composer = IcalComposer {
            command: item_composer(printer, config_paths, account_name, composer.composer)?,
        };

        let Some(draft) = composer.edit(printer, &seed)? else {
            // NOTE: an abandoned edit prints nothing rather than a line
            // saying so: this output is an iCalendar someone pipes
            // onwards, and a message in that stream is not one.
            return Ok(());
        };

        let emitted = emit(printer, output.as_deref(), family, &draft.contents);

        draft.finish(emitted)
    }
}

/// The iCalendar the command built.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IcalBuildOutput {
    /// The raw iCalendar, as text.
    pub contents: String,
}

impl fmt::Display for IcalBuildOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)
    }
}

/// Resolves the composer a `-i` build edits in.
///
/// A `--composer` line names it on its own, and only its absence sends
/// the command looking for a configuration: building an item is otherwise
/// something a machine holding none can do.
fn item_composer(
    printer: &mut impl Printer,
    config_paths: &[PathBuf],
    account_name: Option<&str>,
    flag: Option<String>,
) -> Result<CommandConfig> {
    if let Some(line) = flag {
        return Ok(CommandConfig::Shell(line));
    }

    let (config, account_config) = resolve_account(printer, config_paths, account_name)?;
    let account = Account::from(config).merge(Account::from(account_config));

    account.item_composer(None)
}

/// Hands the item over: to the file `-o` names, or to the printer.
fn emit(
    printer: &mut impl Printer,
    output: Option<&Path>,
    family: IcalFamily,
    item: &[u8],
) -> Result<()> {
    let Some(path) = output else {
        let contents = String::from_utf8(item.to_vec())?;
        if family == IcalFamily::Event {
            return printer.out(EventBuildOutput::of(contents));
        }
        return printer.out(IcalBuildOutput { contents });
    };

    fs::write(path, item).with_context(|| format!("Cannot write iCalendar {path:?}"))?;

    printer.out(Message::new(format!(
        "{} successfully written to {}",
        family.label(),
        path.display()
    )))
}
