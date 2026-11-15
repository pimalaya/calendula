//! # Discover command
//!
//! The `calendula caldav discover` command, reporting the CalDAV endpoints
//! the client resolves from the configured server.

use std::fmt;

use anyhow::Result;
use clap::Parser;
use pimalaya_cli::{fields::Fields, printer::Printer, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::caldav::client::CaldavClient;

/// Report the resolved CalDAV endpoints for the active account.
///
/// The client resolves `current-user-principal` (RFC 5397) and
/// `calendar-home-set` (RFC 4791) from its server. A `home`-configured
/// account skips the principal walk, so an endpoint the server does not
/// answer shows as unresolved rather than failing the command.
///
/// JSON output: `{"server", "principal", "calendarHomeSet"}`, with
/// unresolved endpoints as `null`.
#[derive(Debug, Parser)]
pub struct CaldavDiscoverCommand;

impl CaldavDiscoverCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CaldavClient) -> Result<()> {
        let server = client.base_url.to_string();
        let principal = client
            .current_user_principal()
            .ok()
            .map(|url| url.to_string());
        let calendar_home_set = client.calendar_home_set().ok().map(|url| url.to_string());

        printer.out(CaldavDiscoverOutput {
            server,
            principal,
            calendar_home_set,
        })
    }
}

/// The endpoints a CalDAV session runs against, `None` when unresolved.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CaldavDiscoverOutput {
    /// The DAV context root the session runs against.
    pub server: String,
    /// The principal the server reports for these credentials, `None`
    /// when unresolved.
    pub principal: Option<String>,
    /// The collection the principal files its calendars under, `None`
    /// when unresolved.
    pub calendar_home_set: Option<String>,
}

impl fmt::Display for CaldavDiscoverOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unresolved = "(unresolved)";
        let principal = self.principal.as_deref().unwrap_or(unresolved);
        let home = self.calendar_home_set.as_deref().unwrap_or(unresolved);

        let mut fields = Fields::default();
        fields.push("Server", sanitize::text(&self.server));
        fields.push("Principal", sanitize::text(principal));
        fields.push("Calendar home-set", sanitize::text(home));
        write!(f, "{fields}")
    }
}
