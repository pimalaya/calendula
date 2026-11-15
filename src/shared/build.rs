//! # Item build
//!
//! The pipeline `event build` and `item build` run: an iCalendar source,
//! set by the family's field flags, refined in the composer and printed,
//! rather than written to a backend.

use core::fmt;

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use pimalaya_cli::{
    clap::parsers::path_parser,
    output::{Bytes, SavedOutput},
    printer::Printer,
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
        item::{ItemContents, ItemEncoding},
    },
};

/// What a family sets on the item between its source and the composer.
///
/// Only `event` has field flags; `item` passes [`NoFields`].
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
            command: item_composer(config_paths, account_name, composer.composer)?,
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
    /// The raw iCalendar, as text or as base64.
    #[serde(flatten)]
    pub contents: ItemContents,
}

impl fmt::Display for IcalBuildOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents.contents)
    }
}

/// Resolves the composer a `-i` build edits in.
///
/// A `--composer` line names it on its own, and only its absence sends
/// the command looking for a configuration: building an item is otherwise
/// something a machine holding none can do.
fn item_composer(
    config_paths: &[PathBuf],
    account_name: Option<&str>,
    flag: Option<String>,
) -> Result<CommandConfig> {
    if let Some(line) = flag {
        return Ok(CommandConfig::Shell(line));
    }

    let (config, _, account_config) = resolve_account(config_paths, account_name)?;
    let account = Account::from(config).merge(Account::from(account_config));

    account.item_composer(None)
}

/// Hands the item over: to the file `-o` names, or to the printer.
///
/// An item that is not UTF-8 goes out as its bytes in text, which a pipe
/// takes on as they are, and in base64 under `--json`, as cardamum's
/// `card build` hands a vCard over.
fn emit(
    printer: &mut impl Printer,
    output: Option<&Path>,
    family: IcalFamily,
    item: &[u8],
) -> Result<()> {
    let Some(path) = output else {
        let contents = ItemContents::of(item);

        if contents.encoding == ItemEncoding::Base64 && !printer.is_json() {
            return printer.out_bytes(Bytes {
                data: item.to_vec(),
                mime: String::from("text/calendar"),
            });
        }

        if family == IcalFamily::Event {
            return printer.out(EventBuildOutput::of(item, contents));
        }

        return printer.out(IcalBuildOutput { contents });
    };

    fs::write(path, item).with_context(|| format!("Cannot write iCalendar {path:?}"))?;

    printer.out(SavedOutput {
        path: path.display().to_string(),
        size: item.len(),
    })
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use anyhow::Result;
    use pimalaya_cli::{output::Bytes, printer::CapturePrinter, printer::Printer};
    use serde::Serialize;
    use serde_json::json;

    use super::emit;
    use crate::shared::ical::IcalFamily;

    /// A printer keeping the text it was handed, and the raw bytes.
    #[derive(Default)]
    struct TextPrinter(Vec<u8>);

    impl Printer for TextPrinter {
        fn out<T: fmt::Display + Serialize>(&mut self, data: T) -> Result<()> {
            self.0.extend(data.to_string().into_bytes());
            Ok(())
        }

        fn event<T: fmt::Display + Serialize>(&mut self, _event: T) -> Result<()> {
            Ok(())
        }

        fn out_bytes(&mut self, bytes: Bytes) -> Result<()> {
            self.0.extend(bytes.data);
            Ok(())
        }
    }

    const TEXT: &[u8] = b"BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n";
    const LATIN1: &[u8] = b"BEGIN:VCALENDAR\r\nX-NOTE:Caf\xe9\r\nEND:VCALENDAR\r\n";

    /// A built item names how `contents` spells its bytes, as cardamum's
    /// `card build` does: its text, or the base64 of bytes that are not
    /// UTF-8, which the text output prints as they are.
    #[test]
    fn a_built_item_names_its_encoding() {
        for family in [IcalFamily::Item, IcalFamily::Event] {
            let mut printer = CapturePrinter::default();
            emit(&mut printer, None, family, TEXT).unwrap();
            emit(&mut printer, None, family, LATIN1).unwrap();

            let [text, latin1] = &printer.values[..] else {
                panic!("{:?}", printer.values);
            };
            assert_eq!(text["contents"], json!(String::from_utf8_lossy(TEXT)));
            assert_eq!(text["encoding"], "utf-8");
            assert_eq!(
                latin1["contents"],
                "QkVHSU46VkNBTEVOREFSDQpYLU5PVEU6Q2Fm6Q0KRU5EOlZDQUxFTkRBUg0K"
            );
            assert_eq!(latin1["encoding"], "base64");

            let mut printer = TextPrinter::default();
            emit(&mut printer, None, family, LATIN1).unwrap();
            assert_eq!(printer.0, LATIN1);
        }
    }
}
