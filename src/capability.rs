//! # Capabilities
//!
//! What each backend serves of the unified API.
//!
//! The names follow pimdir STORAGE Annex B without its domain word, the
//! collection read as the calendar: calendula's `item.add` is pimdir's
//! `calendar.item.add`, its `calendar.create` pimdir's
//! `collection.create`. A backend declares a capability only when it
//! gives the result every other declarer gives; a different result stays
//! protocol-specific.
//!
//! A verb or a flag the backend does not serve fails before connecting,
//! with the `capability-missing` code. The tables are plain data keyed by
//! backend name and compiled into every build, compiled-out backends
//! included, so the `--help` they annotate reads the same in every build.

use std::fmt;

use anyhow::Result;
use pimalaya_cli::error::CodedError;

use crate::error::ErrorCode;

use Capability as C;
use Support::{Full, None as Not, Partial, Server};

/// A unified capability, named by [`Capability::name`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Listing the calendars.
    CalendarList,
    /// Creating a calendar.
    CalendarCreate,
    /// Renaming a calendar.
    CalendarRename,
    /// Deleting a calendar and the items it holds.
    CalendarDelete,
    /// Setting a calendar's description.
    CalendarDescription,
    /// Setting a calendar's colour.
    CalendarColor,
    /// Listing a calendar's items.
    ItemList,
    /// Reading an item.
    ItemRead,
    /// Adding an item to a calendar.
    ItemAdd,
    /// Replacing an item.
    ItemUpdate,
    /// Removing an item.
    ItemRemove,
    /// Replacing an item only while it is at a given version.
    ItemUpdateIfMatch,
    /// Removing an item only while it is at a given version.
    ItemRemoveIfMatch,
    /// Asking for an online meeting with an event.
    OnlineMeeting,
    /// Choosing whom a write notifies (`--send-updates`).
    SchedulingSendUpdates,
}

impl Capability {
    /// Every capability, in the order the listings print them.
    pub const ALL: [Self; 15] = [
        C::CalendarList,
        C::CalendarCreate,
        C::CalendarRename,
        C::CalendarDelete,
        C::CalendarDescription,
        C::CalendarColor,
        C::ItemList,
        C::ItemRead,
        C::ItemAdd,
        C::ItemUpdate,
        C::ItemRemove,
        C::ItemUpdateIfMatch,
        C::ItemRemoveIfMatch,
        C::OnlineMeeting,
        C::SchedulingSendUpdates,
    ];

    /// The capability's name, pimdir's grammar without the domain word.
    pub fn name(self) -> &'static str {
        match self {
            Self::CalendarList => "calendar.list",
            Self::CalendarCreate => "calendar.create",
            Self::CalendarRename => "calendar.rename",
            Self::CalendarDelete => "calendar.delete",
            Self::CalendarDescription => "calendar.description",
            Self::CalendarColor => "calendar.color",
            Self::ItemList => "item.list",
            Self::ItemRead => "item.read",
            Self::ItemAdd => "item.add",
            Self::ItemUpdate => "item.update",
            Self::ItemRemove => "item.remove",
            Self::ItemUpdateIfMatch => "item.update.if-match",
            Self::ItemRemoveIfMatch => "item.remove.if-match",
            Self::OnlineMeeting => "online-meeting",
            Self::SchedulingSendUpdates => "scheduling.send-updates",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How well a backend serves a capability, pimdir's levels plus the one
/// the server decides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Support {
    /// Served with the result every declarer gives.
    Full,
    /// Served, the detail saying what differs or is missing.
    Partial(&'static str),
    /// Served when the server allows it, the detail saying what it may
    /// refuse; refused by the server otherwise.
    Server(&'static str),
    /// Not served, the detail saying why and what serves it instead.
    None(&'static str),
}

impl Support {
    /// The level's name, as the help and `account capability` print it.
    pub fn level(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Partial(_) => "partial",
            Self::Server(_) => "server",
            Self::None(_) => "none",
        }
    }

    /// What the level leaves out, `None` for a full one.
    pub fn detail(self) -> Option<&'static str> {
        match self {
            Self::Full => None,
            Self::Partial(detail) | Self::Server(detail) | Self::None(detail) => Some(detail),
        }
    }
}

/// One declared capability of a backend.
type Row = (Capability, Support);

/// The backends declaring capabilities, by the name `--backend` takes, in
/// the order `auto` tries them.
pub const BACKENDS: [&str; 5] = ["vdir", "pimdir", "caldav", "gcal", "msgraph"];

/// What `backend` declares, nothing for a name serving no unified verb.
pub fn declared(backend: &str) -> &'static [Row] {
    match backend {
        "vdir" => VDIR,
        "pimdir" => PIMDIR,
        "caldav" => CALDAV,
        "gcal" => GCAL,
        "msgraph" => MSGRAPH,
        _ => &[],
    }
}

/// How `backend` serves `capability`, `none` for a row it does not
/// declare.
pub fn support(backend: &str, capability: Capability) -> Support {
    declared(backend)
        .iter()
        .find(|(declared, _)| *declared == capability)
        .map_or(Not("not declared"), |(_, support)| *support)
}

/// Fails with `capability-missing` on the first of `capabilities` that
/// `backend` does not serve, naming it, why, and the first of the
/// account's `configured` backends serving it, before anything connects.
pub fn require(backend: &str, configured: &[&str], capabilities: &[Capability]) -> Result<()> {
    for capability in capabilities {
        let Not(detail) = support(backend, *capability) else {
            continue;
        };

        let mut message = format!("The {backend} backend does not serve `{capability}`: {detail}");

        let serving = configured
            .iter()
            .find(|other| **other != backend && !matches!(support(other, *capability), Not(_)));

        if let Some(other) = serving {
            message.push_str(&format!(
                "; this account's {other} backend serves it (`--backend {other}`)"
            ));
        }

        return Err(CodedError::new(ErrorCode::CapabilityMissing, message).into());
    }

    Ok(())
}

/// The capabilities a unified leaf reaches, flag-dependent ones included.
const LEAVES: &[(&str, &str, &[Capability])] = &[
    ("calendar", "list", &[C::CalendarList]),
    (
        "calendar",
        "create",
        &[C::CalendarCreate, C::CalendarDescription, C::CalendarColor],
    ),
    (
        "calendar",
        "update",
        &[C::CalendarRename, C::CalendarDescription, C::CalendarColor],
    ),
    ("calendar", "delete", &[C::CalendarDelete]),
    ("event", "agenda", &[C::ItemList]),
    ("event", "list", &[C::ItemList]),
    ("event", "read", &[C::ItemRead]),
    ("event", "find", &[C::ItemList]),
    (
        "event",
        "create",
        &[C::ItemAdd, C::OnlineMeeting, C::SchedulingSendUpdates],
    ),
    (
        "event",
        "update",
        &[
            C::ItemUpdate,
            C::ItemUpdateIfMatch,
            C::OnlineMeeting,
            C::SchedulingSendUpdates,
        ],
    ),
    (
        "event",
        "delete",
        &[
            C::ItemRemove,
            C::ItemRemoveIfMatch,
            C::SchedulingSendUpdates,
        ],
    ),
    ("item", "list", &[C::ItemList]),
    ("item", "read", &[C::ItemRead]),
    ("item", "create", &[C::ItemAdd]),
    ("item", "update", &[C::ItemUpdate, C::ItemUpdateIfMatch]),
    ("item", "delete", &[C::ItemRemove, C::ItemRemoveIfMatch]),
];

/// The `--help` footer of `group leaf`: for each capability it reaches,
/// the backends declaring it by level, then what each detail says.
///
/// Generated from the tables, so it never drifts from what the commands
/// enforce. Empty for a leaf reaching no backend.
pub fn help(group: &str, leaf: &str) -> String {
    let Some((_, _, capabilities)) = LEAVES
        .iter()
        .find(|(known_group, known_leaf, _)| *known_group == group && *known_leaf == leaf)
    else {
        return String::new();
    };

    let mut help = String::from("Backends:");

    for capability in *capabilities {
        let mut levels: Vec<(&str, Vec<&str>)> = Vec::new();
        let mut details: Vec<(&str, Vec<&str>)> = Vec::new();

        for backend in BACKENDS {
            let support = support(backend, *capability);
            group_by(&mut levels, support.level(), backend);

            if let Some(detail) = support.detail() {
                group_by(&mut details, detail, backend);
            }
        }

        let levels: Vec<String> = levels
            .into_iter()
            .map(|(level, backends)| format!("{level} on {}", backends.join(", ")))
            .collect();

        help.push_str(&format!("\n  {capability}: {}", levels.join("; ")));

        for (detail, backends) in details {
            help.push_str(&format!("\n    {}: {detail}", backends.join(", ")));
        }
    }

    help
}

/// Pushes `item` under `key`, opening the group on first sight so the
/// groups keep the order they were met in.
fn group_by<'a>(groups: &mut Vec<(&'a str, Vec<&'a str>)>, key: &'a str, item: &'a str) {
    match groups.iter_mut().find(|(known, _)| *known == key) {
        Some((_, items)) => items.push(item),
        None => groups.push((key, vec![item])),
    }
}

/// Why a backend without versions takes no `--if-match`.
const NO_VERSION: &str = "the backend exposes no version to compare, so `--if-match` is refused";

/// Why a backend other than pimdir takes no `--online-meeting`.
const NO_MEETING: &str =
    "`--online-meeting` needs the pimdir backend, whose sync engine creates the meeting";

/// Why a pimdir store takes no calendar write.
const PIMDIR_CALENDAR: &str = "the store is an offline cache whose calendars come from the sync \
     engine filling it: run the change against the account it syncs, then sync again";

/// What a queued pimdir write leaves to the store.
const PIMDIR_QUEUED: &str = "queued for the next sync, and refused when a source of the store \
     does not support it";

/// What a server minting calendar ids does with the one given.
const MINTED_ID: &str = "the server mints the calendar's id, reported in place of the one given";

/// What a backend modelling events alone does with another component.
const EVENT_ONLY: &str = "an event only: a to-do or a journal entry is refused";

/// What a version compared in a request of its own leaves open.
const CHANGE_KEY: &str = "compared with the event's `changeKey` read just before the write, \
     so a write landing in between is not caught";

/// vdir, a directory per calendar.
const VDIR: &[Row] = &[
    (C::CalendarList, Full),
    (C::CalendarCreate, Full),
    (C::CalendarRename, Full),
    (C::CalendarDelete, Full),
    (C::CalendarDescription, Full),
    (C::CalendarColor, Full),
    (C::ItemList, Full),
    (C::ItemRead, Full),
    (C::ItemAdd, Full),
    (C::ItemUpdate, Full),
    (C::ItemRemove, Full),
    (C::ItemUpdateIfMatch, Not(NO_VERSION)),
    (C::ItemRemoveIfMatch, Not(NO_VERSION)),
    (C::OnlineMeeting, Not(NO_MEETING)),
    (
        C::SchedulingSendUpdates,
        Not("a vdir store notifies nobody; `--send-updates` needs the gcal backend"),
    ),
];

/// A pimdir store a sync engine fills.
const PIMDIR: &[Row] = &[
    (C::CalendarList, Full),
    (C::CalendarCreate, Not(PIMDIR_CALENDAR)),
    (C::CalendarRename, Not(PIMDIR_CALENDAR)),
    (C::CalendarDelete, Not(PIMDIR_CALENDAR)),
    (C::CalendarDescription, Not(PIMDIR_CALENDAR)),
    (C::CalendarColor, Not(PIMDIR_CALENDAR)),
    (C::ItemList, Full),
    (C::ItemRead, Full),
    (C::ItemAdd, Partial(PIMDIR_QUEUED)),
    (C::ItemUpdate, Partial(PIMDIR_QUEUED)),
    (C::ItemRemove, Partial(PIMDIR_QUEUED)),
    (C::ItemUpdateIfMatch, Full),
    (C::ItemRemoveIfMatch, Full),
    (
        C::OnlineMeeting,
        Partial(
            "refused before it is queued unless a source pushing the event declares \
             `calendar.online-meeting`",
        ),
    ),
    (
        C::SchedulingSendUpdates,
        Not(
            "the sync engine notifies through a source declaring `calendar.scheduling`; \
             `--send-updates` needs the gcal backend",
        ),
    ),
];

/// CalDAV (RFC 4791).
const CALDAV: &[Row] = &[
    (C::CalendarList, Full),
    (
        C::CalendarCreate,
        Server("the server may refuse to create a calendar (`MKCALENDAR`, RFC 4791 section 5.3.1)"),
    ),
    (C::CalendarRename, Full),
    (
        C::CalendarDelete,
        Server("the server may refuse to delete a calendar"),
    ),
    (C::CalendarDescription, Full),
    (C::CalendarColor, Full),
    (C::ItemList, Full),
    (C::ItemRead, Full),
    (C::ItemAdd, Full),
    (C::ItemUpdate, Full),
    (C::ItemRemove, Full),
    (C::ItemUpdateIfMatch, Full),
    (C::ItemRemoveIfMatch, Full),
    (C::OnlineMeeting, Not(NO_MEETING)),
    (
        C::SchedulingSendUpdates,
        Not(
            "an RFC 6638 server notifies the attendees as it decides; `--send-updates` needs \
             the gcal backend",
        ),
    ),
];

/// Google Calendar API v3.
const GCAL: &[Row] = &[
    (C::CalendarList, Full),
    (C::CalendarCreate, Partial(MINTED_ID)),
    (C::CalendarRename, Full),
    (
        C::CalendarDelete,
        Partial("the primary calendar cannot be deleted"),
    ),
    (C::CalendarDescription, Full),
    (
        C::CalendarColor,
        Partial("a Google calendar always carries a colour, so clearing it is refused"),
    ),
    (C::ItemList, Full),
    (C::ItemRead, Full),
    (C::ItemAdd, Partial(EVENT_ONLY)),
    (C::ItemUpdate, Partial(EVENT_ONLY)),
    (C::ItemRemove, Full),
    (C::ItemUpdateIfMatch, Full),
    (C::ItemRemoveIfMatch, Full),
    (C::OnlineMeeting, Not(NO_MEETING)),
    (
        C::SchedulingSendUpdates,
        Partial(
            "an event organized by another address is imported, which mails nobody: `all` and \
             `external-only` are refused on it",
        ),
    ),
];

/// Microsoft Graph calendars.
const MSGRAPH: &[Row] = &[
    (C::CalendarList, Full),
    (C::CalendarCreate, Partial(MINTED_ID)),
    (C::CalendarRename, Full),
    (
        C::CalendarDelete,
        Partial("the default calendar cannot be deleted"),
    ),
    (
        C::CalendarDescription,
        Not("a Graph calendar carries no description"),
    ),
    (
        C::CalendarColor,
        Not("a Graph calendar names its colour from a palette and takes no RGB one"),
    ),
    (C::ItemList, Full),
    (C::ItemRead, Full),
    (C::ItemAdd, Partial(EVENT_ONLY)),
    (
        C::ItemUpdate,
        Partial(
            "an event only, and the series master alone is written: an override edited in the \
             item does not reach the server",
        ),
    ),
    (C::ItemRemove, Full),
    (C::ItemUpdateIfMatch, Partial(CHANGE_KEY)),
    (C::ItemRemoveIfMatch, Partial(CHANGE_KEY)),
    (C::OnlineMeeting, Not(NO_MEETING)),
    (
        C::SchedulingSendUpdates,
        Not("Graph notifies the attendees as it decides; `--send-updates` needs the gcal backend"),
    ),
];

#[cfg(test)]
mod tests {
    use pimalaya_cli::error::code_of;

    use crate::{
        capability::{BACKENDS, Capability, declared, require},
        error::ErrorCode,
    };

    /// Every backend declares every capability once, so `account
    /// capability` and the help never fall back to "not declared".
    #[test]
    fn every_backend_declares_every_capability() {
        for backend in BACKENDS {
            for capability in Capability::ALL {
                let declared = declared(backend).iter().filter(|(c, _)| *c == capability);
                assert_eq!(declared.count(), 1, "{backend} {capability}");
            }
        }
    }

    #[test]
    fn an_unserved_flag_is_refused_naming_the_backend_serving_it() {
        let capabilities = [Capability::ItemRemove, Capability::ItemRemoveIfMatch];

        let err = require("vdir", &["vdir", "pimdir", "caldav"], &capabilities).unwrap_err();
        assert_eq!(code_of(&err), Some(ErrorCode::CapabilityMissing));
        let message = err.to_string();
        assert!(message.starts_with("The vdir backend"), "{message}");
        assert!(message.contains("`item.remove.if-match`"), "{message}");
        let named = "pimdir backend serves it (`--backend pimdir`)";
        assert!(message.ends_with(named), "{message}");

        let alone = require("vdir", &["vdir"], &capabilities).unwrap_err();
        assert!(!alone.to_string().contains("--backend"), "{alone}");

        assert!(require("caldav", &["vdir", "caldav"], &capabilities).is_ok());
    }
}
