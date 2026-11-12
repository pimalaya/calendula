//! # Session
//!
//! The account the commands of one run share, and the unified client kept
//! across them.
//!
//! A one-shot invocation runs a single command over a fresh [`Session`],
//! so it builds exactly the client that command needs. The [`crate::repl`]
//! keeps its session across lines and the [`crate::mcp`] server across
//! tool calls, so the unified client connects once. A command that fails
//! drops the client it ran on, its connection being possibly dead or left
//! mid-exchange, and the next command builds it again. The protocol
//! commands build their own client each time.

use std::path::PathBuf;

use anyhow::Result;
use pimalaya_cli::printer::Printer;

use crate::{backend::Backend, shared::client::CalendarClient};

/// The clients of one run.
#[derive(Default)]
pub struct Session<'a> {
    /// The account every client of the session is built for.
    pub account: SessionAccount<'a>,
    /// The client of the unified commands, built on first use.
    pub unified: Option<CalendarClient>,
}

/// The account a session runs against, as the global flags name it.
#[derive(Clone, Copy, Debug, Default)]
pub struct SessionAccount<'a> {
    /// The configuration paths, `-c` or the canonical ones.
    pub config_paths: &'a [PathBuf],
    /// The account `-a` names, the default one when `None`.
    pub name: Option<&'a str>,
    /// The backend `--backend` allows for the unified commands.
    pub backend: Backend,
}

impl Session<'_> {
    /// Runs a unified command over the kept client, building it when the
    /// session holds none, and dropping it when the command fails.
    pub fn run_unified<P: Printer>(
        &mut self,
        printer: &mut P,
        command: impl FnOnce(&mut P, &mut CalendarClient) -> Result<()>,
    ) -> Result<()> {
        let client = match self.unified.take() {
            Some(kept) => self.unified.insert(kept),
            None => {
                let account = self.account;
                let client =
                    CalendarClient::resolve(account.config_paths, account.name, account.backend)?;
                self.unified.insert(client)
            }
        };

        let result = command(printer, client);

        if result.is_err() {
            self.unified = None;
        }

        result
    }
}
