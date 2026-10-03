//! # Pimdir intents
//!
//! The `pimdir reply` and `pimdir cancel` commands, queueing what a
//! store cannot do by itself: a scheduling message sent by the source
//! the account syncs (pimdir STORAGE Annex B.2).
//!
//! Neither touches the item: the performer sends the message, and the
//! change it makes on the server (the account's new `PARTSTAT`, the
//! removal of a cancelled event) comes back with the next sync.

use std::fmt;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    pimdir::backend::{PimdirBackend, PimdirQueued},
    shared::note::Noted,
};

/// The answer to an invitation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PimdirPartstat {
    /// Attend (`ACCEPTED`).
    Accept,
    /// Maybe attend (`TENTATIVE`).
    Tentative,
    /// Do not attend (`DECLINED`).
    Decline,
}

impl PimdirPartstat {
    /// The `PARTSTAT` the intent carries.
    pub fn as_partstat(self) -> &'static str {
        match self {
            Self::Accept => "ACCEPTED",
            Self::Tentative => "TENTATIVE",
            Self::Decline => "DECLINED",
        }
    }
}

/// Reply to an invitation, sent by the next sync.
///
/// Queues a `calendar-reply` intent for the event: the source the
/// account syncs sends the reply to the organizer, with the comment, and
/// the account's new participation status comes back with the sync after.
/// The event itself is left untouched until then.
///
/// When several sources of the account can send it, pass `--source`.
///
/// JSON output: `{"queued", "calendarId", "eventId", "partstat",
/// "comment", "source"}`, `queued` being the queue row.
#[derive(Debug, Parser)]
pub struct PimdirReplyCommand {
    /// Calendar holding the event; falls back to `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,

    /// Identifier of the event, as `event list` shows it.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// The answer.
    #[arg(value_name = "ANSWER", value_enum)]
    pub answer: PimdirPartstat,

    /// A note for the organizer, sent with the reply.
    #[arg(long, short = 'm', value_name = "TEXT")]
    pub comment: Option<String>,

    /// The source to send it, among several able to.
    #[arg(long, value_name = "SOURCE")]
    pub source: Option<String>,
}

impl PimdirReplyCommand {
    pub fn execute(self, printer: &mut impl Printer, mut backend: PimdirBackend) -> Result<()> {
        let calendar_id = backend.calendar_id(self.calendar)?;
        let partstat = self.answer.as_partstat();

        let queued = backend.reply(
            &calendar_id,
            &self.event_id,
            partstat,
            self.comment.as_deref(),
            self.source.as_deref(),
        )?;

        printer.out(Noted {
            output: PimdirIntentOutput::new(
                queued,
                calendar_id,
                self.event_id,
                Some(partstat),
                self.comment,
            ),
            notes: backend.take_notes(),
        })
    }
}

/// Cancel an event you organise, notifying its attendees on the next sync.
///
/// Queues a `calendar-cancel` intent for the event: the source the
/// account syncs cancels it and notifies the attendees, with the comment,
/// and the removal comes back with the sync after. An event inviting
/// nobody has nobody to notify, so `event delete` removes it instead.
///
/// When several sources of the account can send it, pass `--source`.
///
/// JSON output: `{"queued", "calendarId", "eventId", "partstat",
/// "comment", "source"}`, `partstat` being `null`.
#[derive(Debug, Parser)]
pub struct PimdirCancelCommand {
    /// Calendar holding the event; falls back to `calendar.default`.
    #[arg(short = 'k', long = "calendar", value_name = "CALENDAR-ID")]
    pub calendar: Option<String>,

    /// Identifier of the event, as `event list` shows it.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,

    /// A note for the attendees, sent with the cancellation.
    #[arg(long, short = 'm', value_name = "TEXT")]
    pub comment: Option<String>,

    /// The source to send it, among several able to.
    #[arg(long, value_name = "SOURCE")]
    pub source: Option<String>,
}

impl PimdirCancelCommand {
    pub fn execute(self, printer: &mut impl Printer, mut backend: PimdirBackend) -> Result<()> {
        let calendar_id = backend.calendar_id(self.calendar)?;

        let queued = backend.cancel(
            &calendar_id,
            &self.event_id,
            self.comment.as_deref(),
            self.source.as_deref(),
        )?;

        printer.out(Noted {
            output: PimdirIntentOutput::new(queued, calendar_id, self.event_id, None, self.comment),
            notes: backend.take_notes(),
        })
    }
}

/// An intent queued, as `pimdir reply` and `pimdir cancel` report it.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirIntentOutput {
    /// The queue row the intent was appended as.
    pub queued: i64,
    /// The calendar holding the event.
    pub calendar_id: String,
    /// The event the intent addresses.
    pub event_id: String,
    /// The `PARTSTAT` a reply carries, `null` for a cancellation.
    pub partstat: Option<String>,
    /// The comment sent with it.
    pub comment: Option<String>,
    /// The source named to perform it, `null` in a store whose sources
    /// declare nothing, where the sync engine picks.
    pub source: Option<String>,
}

impl PimdirIntentOutput {
    fn new(
        queued: PimdirQueued,
        calendar_id: String,
        event_id: String,
        partstat: Option<&str>,
        comment: Option<String>,
    ) -> Self {
        Self {
            queued: queued.row,
            calendar_id,
            event_id,
            partstat: partstat.map(ToOwned::to_owned),
            comment,
            source: queued.source,
        }
    }
}

impl fmt::Display for PimdirIntentOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.partstat {
            Some(partstat) => write!(f, "Reply {partstat} to event `{}`", self.event_id)?,
            None => write!(f, "Cancellation of event `{}`", self.event_id)?,
        }
        writeln!(f, " queued; the next sync sends it")
    }
}
