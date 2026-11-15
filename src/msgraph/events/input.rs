//! # Event JSON input
//!
//! The raw Graph event body shared by the create and update commands.

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Parser;
use io_msgraph::v1::rest::users::events::MsgraphEvent;

/// Positional raw Graph-event JSON source for `create` / `update`.
#[derive(Debug, Parser)]
pub struct EventJsonArg {
    /// A path to a JSON file, raw Graph event JSON, or `-` for stdin.
    #[arg(value_name = "JSON")]
    pub body: String,
}

impl EventJsonArg {
    /// Reads the source and parses it into a Graph event.
    ///
    /// A `-` reads stdin, an existing path is read as a file, and anything
    /// else is the JSON itself.
    pub fn read(self) -> Result<MsgraphEvent> {
        let raw = if self.body == "-" {
            pimalaya_cli::session::check_stdin(crate::error::ErrorCode::StdinTaken)?;
            let mut buf = String::new();
            stdin()
                .read_to_string(&mut buf)
                .context("Read event JSON from stdin error")?;
            buf
        } else {
            let path = PathBuf::from(&self.body);
            if path.is_file() {
                fs::read_to_string(&path)
                    .with_context(|| format!("Read event JSON from `{}` error", path.display()))?
            } else {
                self.body
            }
        };

        serde_json::from_str(&raw).context("Parse Graph event JSON error")
    }
}
