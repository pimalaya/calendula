//! # MCP command
//!
//! The `mcp` subcommand, a Model Context Protocol server over stdin and
//! stdout exposing the unified API as tools, one per leaf command.
//!
//! pimalaya-cli's [`mcp`] module answers the protocol and checks each call
//! against its tool's input schema; this one lists the tools and runs the
//! calls. A tool's input is its clap command's [`Grammar`], and a call
//! turns back into an argv parsed by that same command, so the CLI, the
//! [`crate::repl`] and MCP share one grammar. Commands run over the
//! [`Session`] the server keeps and print into a [`CapturePrinter`], whose
//! one JSON value becomes the call's structured result.
//!
//! [`mcp`]: pimalaya_cli::mcp

use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Result, anyhow};
use clap::{CommandFactory, Parser};
use pimalaya_cli::{
    mcp::{ServerInfo, ToolCall, ToolResult, grammar::Grammar, serve, tool_output_schema},
    printer::CapturePrinter,
};
use serde_json::{Value, json};

use crate::{
    capability,
    cli::{Cli, Command, resolve_account},
    error::ErrorCode,
    json_schema,
    session::Session,
    shared::client::CalendarClient,
};

/// What the server tells the agent at initialization.
const INFO: ServerInfo = ServerInfo {
    name: "calendula",
    version: env!("CARGO_PKG_VERSION"),
    instructions: Some(INSTRUCTIONS),
};

/// How to use the tools, for the client to hand its model.
const INSTRUCTIONS: &str = "Reads, and when the server allows it changes, \
the calendars of one account. Find a calendar with calendar_list and pass \
its id as `calendar`, the account's default one when left out. List its \
events with event_list, a page at a time (give `nextPage` back as \
`page-token`), or between two days with `from` and `to` (YYYY-MM-DD), \
which also lists each occurrence of a recurring event. An event's `id` \
names it, in that calendar, for event_read, event_update and \
event_delete; event_find looks one up by the iCalendar UID an invitation \
carries. item_list and item_read give the raw iCalendar of any \
component. event_create and event_update take the event as field \
arguments (`summary`, `start`, `end`, `location`) or as iCalendar text \
in `ical`, never as a path. account_capability says what this account's \
backend serves; a tool it cannot serve is not listed. The tools that \
change the calendars are listed only when the server runs with \
--allow-write. Inviting people is sending: the `attendee`, `organizer` \
and `send-updates` arguments are listed only with --allow-send as well, \
and without it a write or a delete of an item naming an attendee \
(ATTENDEE), in the iCalendar given or in the one stored, is refused, \
since the server or a sync may notify them. A failed call answers \
isError with a calendula-error object, its `code` naming the failure \
when it has one.";

/// The unified API leaves exposed as tools, with what each may do.
///
/// A leaf missing here is not exposed, so a new command is listed only
/// once it is classified. `event build` and `item build` (local formatters
/// writing files) and `event agenda` (a grid drawn for people) are left
/// out, and so are the protocol commands.
const TOOLS: &[McpTool] = &[
    McpTool::read("account", "capability"),
    McpTool::read("calendar", "list"),
    McpTool::write("calendar", "create", false),
    McpTool::write("calendar", "update", true),
    McpTool::write("calendar", "delete", true),
    McpTool::read("event", "list"),
    McpTool::read("event", "read"),
    McpTool::read("event", "find"),
    McpTool::schedule("event", "create", false),
    McpTool::schedule("event", "update", true),
    McpTool::schedule("event", "delete", true),
    McpTool::read("item", "list"),
    McpTool::read("item", "read"),
    McpTool::schedule("item", "create", false),
    McpTool::schedule("item", "update", true),
    McpTool::schedule("item", "delete", true),
];

/// The arguments inviting or notifying people, listed only under
/// `--allow-send`.
const SEND_ARGS: &[&str] = &[
    "attendee",
    "add-attendee",
    "remove-attendee",
    "organizer",
    "send-updates",
];

/// The arguments no tool takes: the composer spawns an editor, which a
/// server has no terminal for.
const COMPOSER_ARGS: &[&str] = &["interactive", "composer"];

/// Whether local file paths given as command input are read as such.
static PATHS_CLOSED: AtomicBool = AtomicBool::new(false);

/// Whether a write may touch an item naming attendees.
static SENDING_CLOSED: AtomicBool = AtomicBool::new(false);

/// Start an MCP server over stdio for one account.
///
/// Each tool is a unified API command (`event_list`, `event_read`) run
/// over a client connected once for the server's lifetime; the account
/// and `--backend` are the server's. Without flags only the tools that
/// read are listed, and a tool the account's backend does not serve is
/// never listed.
#[derive(Debug, Parser)]
pub struct McpCommand {
    /// List the tools that change the calendars and the items too.
    ///
    /// Adds the calendar create, update and delete tools and the event
    /// and item create, update and delete tools, without the arguments
    /// inviting people.
    #[arg(long)]
    pub allow_write: bool,
    /// Let the writes reach the people an item invites.
    ///
    /// Adds the `attendee`, `add-attendee`, `remove-attendee`,
    /// `organizer` and `send-updates` arguments, and lets a write or a
    /// delete touch an item naming an attendee (`ATTENDEE`), whom the
    /// server or a sync may notify. An agent reading an invitation can be
    /// told by it to cancel or move a meeting, so it is never on by
    /// default.
    #[arg(long, requires = "allow_write")]
    pub allow_send: bool,
}

impl McpCommand {
    /// Serves JSON-RPC requests off stdin until EOF, answering each on
    /// stdout.
    pub fn execute(self, session: &mut Session) -> Result<()> {
        let account = session.account;

        // NOTE: resolved once up front, so a missing configuration or
        // account fails the server rather than its first call, and the
        // tools listed are the ones the account's backend serves.
        let (config, name, account_config) = resolve_account(account.config_paths, account.name)?;
        let client = CalendarClient::new(config, name, account_config, account.backend)?;
        let backend = client.backend_name();
        session.unified = Some(client);

        PATHS_CLOSED.store(true, Ordering::Relaxed);
        SENDING_CLOSED.store(!self.allow_send, Ordering::Relaxed);

        let tools = McpToolSpec::all(self.allow_write, self.allow_send, Some(backend));
        let definitions: Vec<Value> = tools.iter().map(|tool| tool.definition.clone()).collect();

        let mut server = McpServer { session, tools };

        serve(&INFO, &definitions, |call| server.call(call))
    }
}

/// Whether a local file path given as command input must be taken as the
/// input itself, as it is inside an MCP server.
pub fn paths_closed() -> bool {
    PATHS_CLOSED.load(Ordering::Relaxed)
}

/// Whether a write must leave alone an item naming attendees, as it does
/// inside an MCP server without `--allow-send`.
pub fn sending_closed() -> bool {
    SENDING_CLOSED.load(Ordering::Relaxed)
}

/// A leaf as a tool, and what it may do.
struct McpTool {
    group: &'static str,
    leaf: &'static str,
    kind: McpToolKind,
    destructive: bool,
}

/// What a tool may do, deciding which flag lists it and its hints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum McpToolKind {
    /// Reads, listed always.
    Read,
    /// Changes a calendar, listed under `--allow-write`.
    Write,
    /// Changes an item, whose attendees it may notify: listed under
    /// `--allow-write`, reaching them under `--allow-send` alone.
    Schedule,
}

impl McpTool {
    const fn read(group: &'static str, leaf: &'static str) -> Self {
        Self::new(group, leaf, McpToolKind::Read, false)
    }

    const fn write(group: &'static str, leaf: &'static str, destructive: bool) -> Self {
        Self::new(group, leaf, McpToolKind::Write, destructive)
    }

    const fn schedule(group: &'static str, leaf: &'static str, destructive: bool) -> Self {
        Self::new(group, leaf, McpToolKind::Schedule, destructive)
    }

    const fn new(
        group: &'static str,
        leaf: &'static str,
        kind: McpToolKind,
        destructive: bool,
    ) -> Self {
        Self {
            group,
            leaf,
            kind,
            destructive,
        }
    }
}

/// A listed tool: its definition and the grammar a call maps onto.
struct McpToolSpec {
    grammar: Grammar,
    definition: Value,
}

impl McpToolSpec {
    /// The tools the gates allow, from the commands this build carries.
    ///
    /// With a backend, a tool it does not serve at all is left out, as
    /// the capability tables know without connecting.
    fn all(allow_write: bool, allow_send: bool, backend: Option<&str>) -> Vec<Self> {
        let mut cli = Cli::command();
        cli.build();

        let schemas = json_schema::registry().schemas;
        let mut tools = Vec::new();

        let gated = |name: &str| {
            COMPOSER_ARGS.contains(&name) || (SEND_ARGS.contains(&name) && !allow_send)
        };

        for tool in TOOLS {
            if tool.kind != McpToolKind::Read && !allow_write {
                continue;
            }

            let served =
                backend.is_none_or(|backend| capability::serves(backend, tool.group, tool.leaf));

            if !served {
                continue;
            }

            let Some(cmd) = cli
                .find_subcommand(tool.group)
                .and_then(|group| group.find_subcommand(tool.leaf))
            else {
                continue;
            };

            let grammar = Grammar::new(&[tool.group, tool.leaf], cmd).retain(|name| !gated(name));

            let description = cmd
                .get_long_about()
                .or(cmd.get_about())
                .map(ToString::to_string)
                .unwrap_or_default();

            let mut definition = json!({
                "name": format!("{}_{}", tool.group, tool.leaf),
                "description": description,
                "inputSchema": grammar.input_schema(),
                "annotations": {
                    "readOnlyHint": tool.kind == McpToolKind::Read,
                    "destructiveHint": tool.destructive,
                    "openWorldHint": tool.kind == McpToolKind::Schedule && allow_send,
                },
            });

            let schema = schemas.get(&format!("calendula-{}-{}", tool.group, tool.leaf));

            if let Some(schema) = schema.and_then(tool_output_schema) {
                definition["outputSchema"] = schema.clone();
            }

            tools.push(Self {
                grammar,
                definition,
            });
        }

        tools
    }
}

/// The server: the tools it lists and the session they run over.
struct McpServer<'a, 'b> {
    session: &'a mut Session<'b>,
    tools: Vec<McpToolSpec>,
}

impl McpServer<'_, '_> {
    /// Runs a tool call, listed and holding only the arguments its tool
    /// declares, a failure being a result with `isError` rather than a
    /// protocol error, so the caller reads it.
    fn call(&mut self, call: &ToolCall) -> ToolResult {
        let Some(tool) = self
            .tools
            .iter()
            .find(|tool| tool.definition["name"] == *call.name)
        else {
            return ToolResult::error::<ErrorCode>(anyhow!("Unknown tool: {}", call.name));
        };

        let mut printer = CapturePrinter::default();

        let result = tool
            .grammar
            .argv(&call.arguments)
            .and_then(|argv| Ok(McpLine::try_parse_from(argv)?))
            .and_then(|line| line.cmd.execute(&mut printer, self.session));

        ToolResult::printed::<ErrorCode>(printer, result)
    }
}

/// One tool call parsed back, the CLI grammar minus the binary name and the
/// global flags.
#[derive(Debug, Parser)]
#[command(no_binary_name = true)]
struct McpLine {
    #[command(subcommand)]
    cmd: Command,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::{cli::Cli, mcp::*};

    /// The unified API leaves no tool exposes.
    const EXCLUDED: &[(&str, &str)] = &[("event", "agenda"), ("event", "build"), ("item", "build")];

    #[test]
    fn every_unified_leaf_is_classified() {
        let mut cli = Cli::command();
        cli.build();

        for group in ["calendar", "event", "item"] {
            let group = cli.find_subcommand(group).unwrap();

            for leaf in group
                .get_subcommands()
                .filter(|leaf| leaf.get_name() != "help")
            {
                let path = (group.get_name(), leaf.get_name());
                let classified = TOOLS.iter().any(|tool| (tool.group, tool.leaf) == path)
                    || EXCLUDED.contains(&path);

                assert!(classified, "{path:?} is neither a tool nor excluded");
            }
        }
    }

    #[test]
    fn the_instructions_name_existing_tools() {
        let words = INSTRUCTIONS
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .filter(|word| word.contains('_'));

        for word in words {
            let known = TOOLS
                .iter()
                .any(|tool| word == format!("{}_{}", tool.group, tool.leaf));
            assert!(known, "{word}");
        }
    }
}
