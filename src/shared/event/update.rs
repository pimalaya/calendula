//! # Event update
//!
//! The `event update` command, replacing one VEVENT of the selected
//! calendar, from an iCalendar source, from field flags, from the
//! composer, or from any combination of the three.

use core::fmt;

use anyhow::{Result, bail};
use clap::Parser;
use jiff::Timestamp;
use pimalaya_cli::printer::Printer;
use schemars::JsonSchema;
use serde::Serialize;

use crate::shared::{
    arg::{CalendarIdArg, IcalComposerArgs},
    client::CalendarClient,
    composer::IcalComposer,
    event::fields::{EventFieldsArgs, revise},
    ical::read_source,
};

/// Overwrite an existing event from an iCalendar source.
///
/// The source, the field flags and `-i` stack: the source replaces the
/// event's bytes, the flags set the properties they name on it, and `-i`
/// opens the result in the composer. The whole component is replaced, so
/// a source has to carry every property the event keeps: what it omits
/// is dropped.
///
/// With no source the event is read from the backend first, and the
/// version it answered guards the write; `--summary "New title"` alone
/// then changes the title and nothing else. A `--start` given without
/// `--end` or `--duration` moves the event, which keeps its length. An
/// empty text or person flag removes its property: `--location ""`.
///
/// Whatever it writes, the event is marked as revised then: `DTSTAMP`
/// is set to the time of the write, in UTC, and `LAST-MODIFIED` to the
/// same instant when the event carries one.
///
/// The flags cover the common fields; a recurrence, an alarm, a
/// conference link and a hand-written `VTIMEZONE` are left to the
/// composer, the complete surface. `--online-meeting` asks a pimdir
/// store's sync engine to create a meeting, and is refused elsewhere.
///
/// JSON output: `{"id"}`, the event the backend updated.
#[derive(Debug, Parser)]
pub struct EventUpdateCommand {
    #[command(flatten)]
    pub calendar: CalendarIdArg,
    /// Gate the update on this ETag, as returned by a previous read.
    ///
    /// The update only lands if the backend still holds that version
    /// (RFC 9110 `If-Match`), which is how a concurrent write is caught.
    /// With no source the command sends the ETag it read when this is
    /// omitted.
    #[arg(long, value_name = "ETAG")]
    pub if_match: Option<String>,
    /// The composer the event is refined in before it is written.
    #[command(flatten)]
    pub composer: IcalComposerArgs,
    /// The properties the command sets on the event.
    #[command(flatten)]
    pub fields: EventFieldsArgs,
    /// Event to update, as `event list` reports it.
    #[arg(value_name = "EVENT-ID")]
    pub event_id: String,
    /// iCalendar replacing the current contents: a path to a file, raw
    /// iCalendar contents, or `-` for stdin. Omit to start from the
    /// stored event.
    #[arg(value_name = "ICAL")]
    pub ical: Option<String>,
}

impl EventUpdateCommand {
    pub fn execute(self, printer: &mut impl Printer, client: CalendarClient) -> Result<()> {
        self.execute_at(printer, client, Timestamp::now)
    }

    /// Runs the update, `now` telling the instant the event is revised
    /// at when it is written.
    ///
    /// The refresh is this shared pipeline's rather than each backend's,
    /// so every backend stores an event revised alike; a projected one
    /// maps `DTSTAMP` and `LAST-MODIFIED` as its projection says.
    pub fn execute_at(
        self,
        printer: &mut impl Printer,
        mut client: CalendarClient,
        now: impl Fn() -> Timestamp,
    ) -> Result<()> {
        if self.ical.is_none() && self.fields.is_empty() && !self.composer.interactive {
            bail!("Nothing to update; give an iCalendar, a field flag, or -i to edit the event");
        }

        self.fields.check_backend(client.backend_name())?;

        let calendar_id = client.account.calendar_id(self.calendar.id)?;

        // NOTE: reading the stored bytes also gives the version they are
        // being changed from, so an edit that took a minute cannot
        // silently overwrite a write that landed during it.
        let (base, etag) = match &self.ical {
            Some(source) => (read_source(source)?, None),
            None => {
                let item = client.get_item(&calendar_id, &self.event_id)?;
                (item.contents, item.etag)
            }
        };

        let if_match = self.if_match.or(etag);
        let seed = self.fields.apply_keeping_length(&base)?;

        if !self.composer.interactive {
            let contents = revise(&seed, now())?;
            client.update_item(&calendar_id, &self.event_id, contents, if_match.as_deref())?;

            return printer.out(EventUpdateOutput::Applied(EventUpdatedOutput {
                id: self.event_id,
            }));
        }

        let composer = IcalComposer {
            command: client.account.item_composer(self.composer.composer)?,
        };

        // NOTE: reading the event is the one thing that has to happen
        // before the editor, so its connection is dropped here rather
        // than left idle for the minutes an edit takes: a server closes
        // such a connection, and the update below opens a fresh one.
        client.disconnect();

        let Some(draft) = composer.edit(printer, &seed)? else {
            return printer.out(EventUpdateOutput::Abandoned);
        };

        // NOTE: stamped once the edit is over, the instant it is written,
        // whatever the editor left in DTSTAMP.
        let updated = revise(&draft.contents, now()).and_then(|contents| {
            client.update_item(&calendar_id, &self.event_id, contents, if_match.as_deref())
        });

        draft.finish(updated)?;

        printer.out(EventUpdateOutput::Applied(EventUpdatedOutput {
            id: self.event_id,
        }))
    }
}

/// What `event update` prints, which is whether it wrote anything.
///
/// Untagged, so the write serializes exactly as it would on its own. The
/// second shape is reachable through `-i` alone, which `--json` refuses
/// to run.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum EventUpdateOutput {
    /// The event the backend updated.
    Applied(EventUpdatedOutput),
    /// Nothing, the edit having been abandoned.
    Abandoned,
}

impl fmt::Display for EventUpdateOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Applied(out) => out.fmt(f),
            Self::Abandoned => writeln!(f, "Event not updated"),
        }
    }
}

/// The event the backend updated.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventUpdatedOutput {
    /// Backend-specific identifier of the updated event.
    pub id: String,
}

impl fmt::Display for EventUpdatedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Event `{}` successfully updated", self.id)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use anyhow::Result;
    use clap::Parser;
    use jiff::Timestamp;
    use pimalaya_cli::printer::Printer;
    use serde::Serialize;

    use super::EventUpdateCommand;
    use crate::{
        backend::Backend,
        config::{AccountConfig, Config},
        shared::client::CalendarClient,
    };

    /// A printer dropping what it is handed.
    struct Quiet;

    impl Printer for Quiet {
        fn out<T: fmt::Display + Serialize>(&mut self, _data: T) -> Result<()> {
            Ok(())
        }

        fn is_json(&self) -> bool {
            true
        }
    }

    /// The instant the updates are written at: 2026-10-06 12:34:56 UTC.
    fn at() -> Timestamp {
        Timestamp::from_second(1_791_290_096).unwrap()
    }

    const EVENT: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//x//y//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20261019T090000Z\r\n",
        "DTEND:20261019T100000Z\r\n",
        "SUMMARY:Stand-up\r\n",
        "LOCATION:Room 1\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    fn update(client: CalendarClient, args: &[&str]) -> Result<()> {
        let command = EventUpdateCommand::try_parse_from([&["update"], args].concat())?;
        command.execute_at(&mut Quiet, client, at)
    }

    #[cfg(feature = "vdir")]
    mod vdir {
        use super::*;
        use crate::config::VdirConfig;

        /// A vdir home holding one calendar, `cal`, carrying `body`.
        fn home(body: &str) -> (tempfile::TempDir, String) {
            let dir = tempfile::tempdir().unwrap();
            std::fs::create_dir(dir.path().join("cal")).unwrap();
            let id = client(&dir)
                .create_item("cal", body.as_bytes().to_vec())
                .unwrap();
            (dir, id)
        }

        fn client(dir: &tempfile::TempDir) -> CalendarClient {
            let account = AccountConfig {
                vdir: Some(VdirConfig {
                    home_dir: dir.path().to_path_buf(),
                }),
                ..Default::default()
            };
            CalendarClient::new(Config::default(), account, Backend::Auto).unwrap()
        }

        fn stored(dir: &tempfile::TempDir, id: &str) -> String {
            let item = client(dir).get_item("cal", id).unwrap();
            String::from_utf8(item.contents).unwrap()
        }

        #[test]
        fn an_update_from_flags_revises_the_dtstamp() {
            let (dir, id) = home(EVENT);

            update(client(&dir), &["-k", "cal", "--location", "", &id]).unwrap();

            assert_eq!(
                stored(&dir, &id),
                EVENT
                    .replace("DTSTAMP:20260101T000000Z", "DTSTAMP:20261006T123456Z")
                    .replace("LOCATION:Room 1\r\n", "")
            );
        }

        #[test]
        fn an_update_from_a_source_revises_its_last_modified_too() {
            let (dir, id) = home(EVENT);
            let source = EVENT.replace(
                "SUMMARY:Stand-up\r\n",
                "SUMMARY:Retro\r\nLAST-MODIFIED:20260102T000000Z\r\n",
            );

            update(client(&dir), &["-k", "cal", &id, &source]).unwrap();

            assert_eq!(
                stored(&dir, &id),
                source
                    .replace("DTSTAMP:20260101T000000Z", "DTSTAMP:20261006T123456Z")
                    .replace(
                        "LAST-MODIFIED:20260102T000000Z",
                        "LAST-MODIFIED:20261006T123456Z"
                    )
            );
        }

        #[test]
        fn an_online_meeting_is_refused_off_pimdir_by_name() {
            let (dir, id) = home(EVENT);

            let err = update(client(&dir), &["-k", "cal", "--online-meeting", &id])
                .unwrap_err()
                .to_string();

            assert!(err.contains("--online-meeting"), "{err}");
            assert!(err.contains("vdir backend"), "{err}");
            assert_eq!(stored(&dir, &id), EVENT);
        }
    }

    #[cfg(feature = "pimdir")]
    mod pimdir {
        use io_pimdir::capability::{CALENDAR, PimdirCapability, PimdirSupport};

        use super::*;
        use crate::config::PimdirConfig;

        /// A store holding one calendar, `cal`, carrying `EVENT` as an item
        /// the `caldav` source applied, and its public id.
        fn store() -> (tempfile::TempDir, String) {
            let dir = tempfile::tempdir().unwrap();
            let store = io_pimdir::client::PimdirStore::open(dir.path()).unwrap();
            store.ensure_collection("cal", "text/calendar").unwrap();
            drop(store);

            client(&dir)
                .create_item("cal", EVENT.as_bytes().to_vec())
                .unwrap();

            let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_source("caldav");
            owner.drain().unwrap();
            drop(owner);

            let id = client(&dir)
                .list_items("cal", Default::default())
                .unwrap()
                .remove(0)
                .id;
            (dir, id)
        }

        fn client(dir: &tempfile::TempDir) -> CalendarClient {
            let account = AccountConfig {
                pimdir: Some(PimdirConfig {
                    root: dir.path().to_path_buf(),
                    account: None,
                }),
                ..Default::default()
            };
            CalendarClient::new(Config::default(), account, Backend::Auto).unwrap()
        }

        /// Declares every calendar capability for `caldav`, at `support`.
        fn declare(dir: &tempfile::TempDir, support: PimdirSupport) {
            let mut owner = io_pimdir::client::PimdirStore::open(dir.path())
                .unwrap()
                .for_source("caldav");
            let declaration: Vec<_> = CALENDAR
                .iter()
                .map(|name| PimdirCapability {
                    collection: None,
                    name: name.to_string(),
                    support,
                    detail: None,
                })
                .collect();
            owner.declare("caldav", &declaration).unwrap();
        }

        #[test]
        fn an_online_meeting_no_source_declares_is_refused_before_it_is_queued() {
            let (dir, id) = store();

            let err = update(client(&dir), &["-k", "cal", "--online-meeting", &id])
                .unwrap_err()
                .to_string();

            assert!(err.contains("calendar.online-meeting"), "{err}");
            assert!(err.contains("no source of this store declares it"), "{err}");

            let item = client(&dir).get_item("cal", &id).unwrap();
            assert_eq!(item.contents, EVENT.as_bytes());
        }

        #[test]
        fn an_online_meeting_a_source_declares_is_queued_revised() {
            let (dir, id) = store();
            declare(&dir, PimdirSupport::Full);

            update(client(&dir), &["-k", "cal", "--online-meeting", &id]).unwrap();

            let item = client(&dir).get_item("cal", &id).unwrap();
            let contents = String::from_utf8(item.contents).unwrap();
            assert_eq!(
                contents,
                EVENT
                    .replace("DTSTAMP:20260101T000000Z", "DTSTAMP:20261006T123456Z")
                    .replace(
                        "LOCATION:Room 1\r\n",
                        "LOCATION:Room 1\r\nX-PIMDIR-ONLINE-MEETING:TRUE\r\n"
                    )
            );
        }
    }
}
