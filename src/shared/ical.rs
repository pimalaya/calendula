use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use clap::Parser;

/// Positional iCalendar source shared by the `event`/`item` create and
/// update commands.
#[derive(Debug, Parser)]
pub struct IcalArg {
    /// A path to an iCalendar file, raw iCalendar contents, or `-` for
    /// stdin.
    #[arg(value_name = "ICAL")]
    pub ical: String,
}

impl IcalArg {
    /// Resolves the source into raw iCalendar bytes: `-` reads stdin, an
    /// existing file is read, otherwise the value is treated as literal
    /// iCalendar contents.
    pub fn read(self) -> Result<Vec<u8>> {
        if self.ical == "-" {
            let mut buf = Vec::new();
            stdin()
                .read_to_end(&mut buf)
                .context("Read iCalendar from stdin error")?;
            return Ok(buf);
        }

        let path = PathBuf::from(&self.ical);

        if path.is_file() {
            return fs::read(&path)
                .with_context(|| format!("Read iCalendar from `{}` error", path.display()));
        }

        let trimmed = self.ical.trim_start();

        if trimmed.starts_with("BEGIN:VCALENDAR") || trimmed.starts_with("BEGIN:VEVENT") {
            return Ok(self.ical.into_bytes());
        }

        bail!(
            "Source `{}` is neither a readable file nor iCalendar contents",
            self.ical
        )
    }
}
