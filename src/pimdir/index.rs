//! # Pimdir index
//!
//! The read-only query client of the store's search index (pimdir
//! SEARCH), behind `pimdir search` and `pimdir count`.

use std::fmt;

use anyhow::Result;
use ariadne::{Config, Label, Report, ReportKind, Source};
use io_pimdir::{
    client::{
        reader::PimdirItem,
        search::{PimdirIndex, PimdirSearchClock, PimdirSearchMatched, PimdirSearchOptions},
    },
    search::{
        coverage::{PimdirSearchCoverage, PimdirSearchTemporal},
        query::{
            PimdirQuery, PimdirQueryError, PimdirQueryErrorCode, PimdirQueryKind,
            PimdirQueryPredicate, PimdirQueryTerm,
        },
    },
    summary::{PimdirSummary, calendar::PimdirTime},
};
use jiff::tz::TimeZone;
use log::warn;
use pimalaya_cli::{error::CodedError, sanitize};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    error::ErrorCode,
    pimdir::client::{PimdirClient, store_error},
    shared::{
        event::{Event, EventPerson},
        item::CalendarItem,
    },
};

impl PimdirClient {
    /// One page of the events `query` matches in the account.
    pub fn search_items(
        &self,
        query: PimdirQuery,
        calendar: Option<&str>,
        options: &PimdirSearchOptions,
    ) -> Result<PimdirCalendarPage> {
        let index = self.index()?;
        let answer = index
            .search(&self.scope(query, calendar), options)
            .map_err(|err| store_error(err, "Search the pimdir index"))?;

        let mut hits = Vec::new();

        for hit in answer.hits {
            let placement = hit
                .placements
                .iter()
                .find(|placement| Some(placement.collection.as_str()) == calendar)
                .or(hit.placements.first());
            let Some(calendar_id) = placement.map(|placement| placement.collection.clone()) else {
                continue;
            };

            let stored = index
                .reader()
                .get_item(&calendar_id, hit.seq)
                .map_err(|err| store_error(err, format_args!("Read item `{}`", hit.seq)))?;
            let object = stored.as_ref().and_then(|stored| stored.object.as_ref());
            let contents = match object {
                Some(hash) => self.blobs.get(hash)?.unwrap_or_default(),
                None => Vec::new(),
            };
            let item = CalendarItem {
                id: hit.seq.to_string(),
                calendar_id: calendar_id.clone(),
                etag: object.map(|hash| hash.0.clone()),
                contents,
            };
            let matched = hit.matched.map(PimdirCalendarMatched::from);

            // NOTE: an item dropped since the refresh has no row, only a title.
            hits.push(PimdirCalendarHit {
                event: event_hit(&item, stored.as_ref(), hit.title),
                calendar_id,
                matched,
                snippet: hit.snippet,
            });
        }

        Ok(PimdirCalendarPage {
            hits,
            more: answer.more,
            coverage: PimdirCalendarCoverage::from(&answer.coverage),
            zones: PimdirCalendarZones::of(&index, &options.clock)?,
        })
    }

    /// How many events `query` matches in the account, its dates read
    /// against `clock`.
    pub fn count_items(
        &self,
        query: PimdirQuery,
        calendar: Option<&str>,
        clock: &PimdirSearchClock,
    ) -> Result<PimdirCalendarCount> {
        let index = self.index()?;
        let answer = index
            .count(&self.scope(query, calendar), clock)
            .map_err(|err| store_error(err, "Count in the pimdir index"))?;

        Ok(PimdirCalendarCount {
            count: answer.count,
            coverage: PimdirCalendarCoverage::from(&answer.coverage),
            zones: PimdirCalendarZones::of(&index, clock)?,
        })
    }

    /// Opens the store's search index read-only.
    fn index(&self) -> Result<PimdirIndex> {
        PimdirIndex::open_query(&self.root).map_err(|err| {
            store_error(
                err,
                format_args!("Open the pimdir search index of `{}`", self.root.display()),
            )
        })
    }

    /// `query` scoped by top-level `kind:event`, `account:` and
    /// `collection:` conjuncts.
    // NOTE: io-pimdir reads the coverage's scope from top-level terms alone.
    fn scope(&self, query: PimdirQuery, calendar: Option<&str>) -> PimdirQuery {
        let mut conjuncts = match query {
            PimdirQuery::And(conjuncts) => conjuncts,
            query => vec![query],
        };
        let term = |predicate| {
            PimdirQuery::Term(PimdirQueryTerm {
                predicate,
                start: 0,
                end: 0,
            })
        };

        conjuncts.push(term(PimdirQueryPredicate::Kind(PimdirQueryKind::Event)));
        if let Some(account) = &self.account {
            conjuncts.push(term(PimdirQueryPredicate::Account(account.clone())));
        }
        if let Some(calendar) = calendar {
            conjuncts.push(term(PimdirQueryPredicate::Collection(calendar.into())));
        }

        PimdirQuery::And(conjuncts)
    }
}

/// The clock of a search: now, in the zone the tables read times in
/// (`TZ`, then the system's), UTC when that zone has no IANA name.
pub fn search_clock() -> PimdirSearchClock {
    let zone = TimeZone::try_system()
        .ok()
        .and_then(|zone| zone.iana_name().map(ToOwned::to_owned));

    if zone.is_none() {
        warn!("cannot name the local time zone, reading search days in UTC; set TZ");
    }

    PimdirSearchClock { now: None, zone }
}

/// Parses the words of a query, joined by spaces (pimdir SEARCH §8.3),
/// a refusal coded `query-refused`.
pub fn parse_query(words: &[String], json: bool) -> Result<PimdirQuery> {
    let query = words.join(" ");

    PimdirQuery::parse(&query).map_err(|err| {
        let message = match json {
            true => refusal_line(&err),
            false => refusal_report(&query, &err),
        };
        CodedError::new(ErrorCode::QueryRefused, message).into()
    })
}

/// A refusal as one line, for JSON.
fn refusal_line(err: &PimdirQueryError) -> String {
    let mut line = format!(
        "Query refused, {} at {}..{}: {}",
        err.code.as_str(),
        err.start,
        err.end,
        err.message
    );

    if !err.expected.is_empty() {
        line.push_str(&format!("; expected {}", err.expected.join(", ")));
    }

    line
}

/// A refusal drawn over the query with ariadne, without colour.
fn refusal_report(query: &str, err: &PimdirQueryError) -> String {
    let name = "query";
    let config = Config::default().with_color(false);
    // NOTE: an empty span past the end is drawn on the last character.
    let end = err.end.max(err.start + 1);

    let mut report = Report::build(ReportKind::Error, (name, err.start..end))
        .with_config(config)
        .with_code(err.code.as_str())
        .with_message("Query refused")
        .with_label(Label::new((name, err.start..end)).with_message(&err.message));
    if !err.expected.is_empty() {
        report = report.with_note(format!("expected {}", err.expected.join(", ")));
    }
    if err.code == PimdirQueryErrorCode::UnknownField {
        report = report.with_help(format!("fields: {}", err.fields.join(", ")));
    }

    let mut buf = Vec::new();
    let _ = report.finish().write((name, Source::from(query)), &mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}

/// One page of hits.
pub struct PimdirCalendarPage {
    /// The hits, in the order asked.
    pub hits: Vec<PimdirCalendarHit>,
    /// Whether hits follow this page.
    pub more: bool,
    /// What the answer could not see.
    pub coverage: PimdirCalendarCoverage,
    /// The zones, when the index reads floating times in another.
    pub zones: Option<PimdirCalendarZones>,
}

/// A count of hits.
pub struct PimdirCalendarCount {
    /// How many items the query matches.
    pub count: u64,
    /// What the count could not see.
    pub coverage: PimdirCalendarCoverage,
    /// The zones, when the index reads floating times in another.
    pub zones: Option<PimdirCalendarZones>,
}

/// The zones of an index reading floating times and dates in another zone
/// than the search reads days in (pimdir SEARCH §7, §8.1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PimdirCalendarZones {
    /// The zone the index reads floating times and dates in.
    pub index: String,
    /// The zone the search reads days in.
    pub search: String,
}

impl PimdirCalendarZones {
    /// The zones of `index` and `clock`, `None` when they are one.
    fn of(index: &PimdirIndex, clock: &PimdirSearchClock) -> Result<Option<Self>> {
        let index = index
            .floating_zone()
            .map_err(|err| store_error(err, "Read the zone of the pimdir index"))?;
        let search = clock.zone.clone().unwrap_or_else(|| String::from("UTC"));

        Ok((index != search).then_some(Self { index, search }))
    }
}

/// The line under a table.
impl fmt::Display for PimdirCalendarZones {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "The index reads floating times and dates in {}, this search reads days in {}",
            sanitize::text(&self.index),
            sanitize::text(&self.search)
        )
    }
}

/// An event the index matched.
#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCalendarHit {
    /// The event, projected as `event read` projects it.
    #[serde(flatten)]
    pub event: Event,
    /// The calendar holding it.
    pub calendar_id: String,
    /// What a text term matched, `null` when only field terms did.
    pub matched: Option<PimdirCalendarMatched>,
    /// A passage naming a query word, `null` without an indexed body.
    pub snippet: Option<String>,
}

/// What a text term matched (pimdir SEARCH §8.2).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PimdirCalendarMatched {
    /// The local body.
    Body,
    /// The summary of an item whose body is not local.
    Summary,
}

impl From<PimdirSearchMatched> for PimdirCalendarMatched {
    fn from(matched: PimdirSearchMatched) -> Self {
        match matched {
            PimdirSearchMatched::Body => Self::Body,
            PimdirSearchMatched::Summary => Self::Summary,
        }
    }
}

impl fmt::Display for PimdirCalendarMatched {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Body => "body",
            Self::Summary => "summary",
        })
    }
}

/// What an answer could not see (pimdir SEARCH §8.2).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PimdirCalendarCoverage {
    /// Whether every other field is at rest.
    pub complete: bool,
    /// Items in scope whose body is not local.
    pub bodiless: u64,
    /// Items in scope whose last change the index has not folded in.
    pub unindexed: u64,
    /// Changes the store drew since the index was refreshed, store-wide.
    pub behind: u64,
    /// The instant below which the store may lack items of the range.
    pub since: Option<String>,
    /// The instant from which it may lack them.
    pub until: Option<String>,
    /// Whether a first sync is still listing a calendar in scope.
    pub filling: bool,
    /// Whether the date range leaves the index's horizon.
    pub horizon_left: bool,
    /// Series in scope whose occurrences stop before the range's end.
    pub cut: u64,
}

impl From<&PimdirSearchCoverage> for PimdirCalendarCoverage {
    fn from(coverage: &PimdirSearchCoverage) -> Self {
        let (since, until, filling) = match &coverage.temporal {
            PimdirSearchTemporal::Full => (None, None, false),
            PimdirSearchTemporal::Bounded {
                since,
                until,
                filling,
            } => (since.clone(), until.clone(), *filling),
        };

        Self {
            complete: coverage.is_complete() && coverage.local.behind == 0,
            bodiless: coverage.local.bodiless.values().sum(),
            unindexed: coverage.local.unindexed,
            behind: coverage.local.behind,
            since,
            until,
            filling,
            horizon_left: coverage.horizon_left,
            cut: coverage.cut,
        }
    }
}

/// The line under a table, empty when complete.
impl fmt::Display for PimdirCalendarCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let day = |instant: &str| sanitize::text(instant.get(..10).unwrap_or(instant)).into_owned();
        let mut parts = Vec::new();

        if self.bodiless > 0 {
            parts.push(format!(
                "{} item(s) have no local body and matched on their summary alone",
                self.bodiless
            ));
        }
        if let Some(since) = &self.since {
            parts.push(format!("items before {} are not synced", day(since)));
        }
        if let Some(until) = &self.until {
            parts.push(format!("items from {} on are not synced", day(until)));
        }
        if self.filling {
            parts.push(String::from("a first sync is still listing a calendar"));
        }
        if self.horizon_left {
            parts.push(String::from(
                "the dates reach outside the occurrences the index holds",
            ));
        }
        if self.cut > 0 {
            parts.push(format!(
                "{} series stop their occurrences before the range ends",
                self.cut
            ));
        }
        if self.unindexed > 0 {
            parts.push(format!(
                "{} item(s) changed since the index was refreshed",
                self.unindexed
            ));
        }
        if self.behind > 0 {
            parts.push(format!(
                "the index is {} change(s) behind the store, which the sync engine refreshes",
                self.behind
            ));
        }

        if !parts.is_empty() {
            writeln!(f, "Incomplete: {}", parts.join("; "))?;
        }

        Ok(())
    }
}

/// The event a hit renders as: from its body, its summary or its title.
fn event_hit(item: &CalendarItem, stored: Option<&PimdirItem>, title: String) -> Event {
    let events = Event::project(item);
    let master = events
        .iter()
        .find(|event| event.recurrence_id.is_none())
        .or(events.first());
    if let Some(event) = master {
        return event.clone();
    }

    let Some(PimdirSummary::Event(summary)) = stored.and_then(|stored| stored.summary.as_ref())
    else {
        return Event {
            id: item.id.clone(),
            summary: title,
            ..Default::default()
        };
    };

    // NOTE: projected as a body, so the times resolve as a listing's do.
    let mut contents = String::from(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Pimalaya//calendula//EN\r\n\
         BEGIN:VEVENT\r\n",
    );
    if let Some(start) = &summary.dtstart {
        contents.push_str(&time_line("DTSTART", start, &start.value));
        if let Some(end) = &summary.dtend {
            contents.push_str(&time_line("DTEND", start, end));
        }
    }
    contents.push_str("END:VEVENT\r\nEND:VCALENDAR\r\n");

    let times = CalendarItem {
        contents: contents.into_bytes(),
        ..item.clone()
    };
    let event = Event::project(&times)
        .into_iter()
        .next()
        .unwrap_or_default();

    Event {
        id: item.id.clone(),
        etag: None,
        uid: summary.uid.clone().unwrap_or_default(),
        summary: summary.summary.clone(),
        location: summary.location.clone().unwrap_or_default(),
        recurring: summary.recurring.unwrap_or_default(),
        organizer: summary.organizer.as_ref().map(|organizer| EventPerson {
            email: organizer.address.clone(),
            name: organizer.name.clone(),
        }),
        ..event
    }
}

/// A time property `name` holding `value`, in the form of `start`.
fn time_line(name: &str, start: &PimdirTime, value: &str) -> String {
    if start.date {
        return format!("{name};VALUE=DATE:{value}\r\n");
    }

    match &start.tzid {
        Some(tzid) if !value.ends_with(['Z', 'z']) => {
            format!("{name};TZID=\"{}\":{value}\r\n", tzid.replace('"', ""))
        }
        _ => format!("{name}:{value}\r\n"),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use io_pimdir::{
        change::PimdirWriteOp,
        client::{
            PimdirStore,
            search::{
                PimdirIndex, PimdirIndexRefreshOptions, PimdirSearchClock, PimdirSearchOptions,
            },
        },
        collection::PimdirCollectionId,
        placement::{
            PimdirBase, PimdirFlags, PimdirHandle, PimdirLevel, PimdirLinkId, PimdirPlacement,
            PimdirStatus,
        },
        search::{coverage::PimdirSearchCoverage, query::PimdirQuery},
        summary::{
            PimdirSummary,
            calendar::{PimdirEventSummary, PimdirTime},
        },
    };
    use pimalaya_cli::error::code_of;

    use crate::{
        config::PimdirConfig,
        error::ErrorCode,
        pimdir::{
            backend::PimdirBackend,
            client::PimdirClient,
            index::{
                PimdirCalendarCoverage, PimdirCalendarMatched, PimdirCalendarPage,
                PimdirCalendarZones, parse_query,
            },
        },
    };

    /// 2026-11-06T00:00:00Z, the instant the test index is refreshed at.
    const NOW: i64 = 1_793_923_200;

    /// One component of `kind` carrying `lines`, alone in a calendar.
    fn body(kind: &str, uid: &str, lines: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//t//t//EN\r\nBEGIN:{kind}\r\n\
             UID:{uid}@example.org\r\nDTSTAMP:20261001T080000Z\r\n{lines}END:{kind}\r\n\
             END:VCALENDAR\r\n"
        )
    }

    /// A weekly meeting begun on Monday 2019-01-07.
    fn standup() -> String {
        body(
            "VEVENT",
            "standup",
            "DTSTART:20190107T090000Z\r\nDTEND:20190107T091500Z\r\n\
             RRULE:FREQ=WEEKLY\r\nSUMMARY:Stand-up\r\n",
        )
    }

    /// An event at 00:30 UTC on 2026-11-07, naming a place and a guest.
    fn offsite() -> String {
        body(
            "VEVENT",
            "offsite",
            "DTSTART:20261107T003000Z\r\nDTEND:20261107T013000Z\r\nSUMMARY:Offsite\r\n\
             DESCRIPTION:Budget review for the quarter\r\nLOCATION:Lisbon\r\n\
             ORGANIZER:mailto:alice@example.org\r\n\
             ATTENDEE;PARTSTAT=ACCEPTED:mailto:carol@example.org\r\n",
        )
    }

    /// An indexed store of two calendars, `cal` and `work`, one item of
    /// the latter bodiless.
    fn indexed_store() -> (tempfile::TempDir, PimdirBackend) {
        let dir = tempfile::tempdir().unwrap();
        let store = PimdirStore::open(dir.path()).unwrap();
        store.ensure_collection("cal", "text/calendar").unwrap();
        store.ensure_collection("work", "text/calendar").unwrap();
        drop(store);

        let mut backend = backend(dir.path(), None);
        let items = [
            ("cal", standup()),
            ("cal", offsite()),
            (
                "cal",
                body(
                    "VTODO",
                    "report",
                    "DUE:20261110T170000Z\r\nSUMMARY:Write the budget report\r\n\
                     STATUS:IN-PROCESS\r\nPRIORITY:1\r\nPERCENT-COMPLETE:40\r\n",
                ),
            ),
            (
                "cal",
                body(
                    "VTODO",
                    "plants",
                    "SUMMARY:Water the plants\r\nSTATUS:NEEDS-ACTION\r\nPRIORITY:9\r\n",
                ),
            ),
            (
                "cal",
                body(
                    "VJOURNAL",
                    "retro",
                    "DTSTART;VALUE=DATE:20261105\r\nSUMMARY:Retro notes\r\n\
                     DESCRIPTION:The budget slipped\r\n",
                ),
            ),
            (
                "work",
                body(
                    "VEVENT",
                    "planning",
                    "DTSTART:20261112T100000Z\r\nSUMMARY:Planning\r\n\
                     DESCRIPTION:Budget planning\r\n",
                ),
            ),
        ];
        for (calendar, item) in items {
            backend.create_item(calendar, item.into_bytes()).unwrap();
        }

        let mut owner = PimdirStore::open(dir.path()).unwrap().for_source("caldav");
        assert_eq!(owner.drain().unwrap().applied, 6);
        owner
            .write(vec![PimdirWriteOp::UpsertPlacement(bodiless("work"))])
            .unwrap();
        drop(owner);
        refresh(dir.path());

        (dir, backend)
    }

    /// An event of `calendar` listed with its summary alone.
    fn bodiless(calendar: &str) -> PimdirPlacement {
        PimdirPlacement {
            collection: PimdirCollectionId(calendar.into()),
            handle: PimdirHandle("invoice.ics".into()),
            link_id: Some(PimdirLinkId("invoice@example.org".into())),
            object: None,
            level: PimdirLevel::Meta,
            summary: Some(PimdirSummary::Event(PimdirEventSummary {
                uid: Some("invoice@example.org".into()),
                summary: "Invoice review".into(),
                dtstart: Some(PimdirTime {
                    value: "20261120T140000Z".into(),
                    ..Default::default()
                }),
                dtend: Some("20261120T150000Z".into()),
                ..Default::default()
            })),
            sort_key: "2026-11-20T14:00:00Z".into(),
            flags: PimdirFlags::default(),
            status: PimdirStatus::Clean,
            conflict_revision: None,
            conflict_object: None,
            base: Some(PimdirBase {
                flags: PimdirFlags::default(),
                revision: None,
                object: None,
            }),
            origin: None,
        }
    }

    fn backend(dir: &Path, account: Option<&str>) -> PimdirBackend {
        PimdirBackend::new(PimdirConfig {
            root: dir.to_path_buf(),
            account: account.map(Into::into),
        })
        .unwrap()
    }

    /// Refreshes the index at [`NOW`], as the store's owner does.
    fn refresh(dir: &Path) {
        PimdirIndex::open(dir)
            .unwrap()
            .refresh(&PimdirIndexRefreshOptions {
                now: Some(NOW),
                ..Default::default()
            })
            .unwrap();
    }

    fn query(text: &str) -> PimdirQuery {
        parse_query(&[text.to_string()], true).unwrap()
    }

    fn page(client: &PimdirClient, text: &str, calendar: Option<&str>) -> PimdirCalendarPage {
        let options = PimdirSearchOptions::default();
        client
            .search_items(query(text), calendar, &options)
            .unwrap()
    }

    /// The summaries of the hits of `text`, sorted.
    fn summaries(client: &PimdirClient, text: &str, calendar: Option<&str>) -> Vec<String> {
        let hits = page(client, text, calendar).hits;
        let mut summaries: Vec<String> = hits.into_iter().map(|h| h.event.summary).collect();
        summaries.sort();
        summaries
    }

    fn events(client: &PimdirClient, text: &str) -> Vec<String> {
        summaries(client, text, None)
    }

    /// A clock at [`NOW`] reading days in `zone`.
    fn clock(zone: &str) -> PimdirSearchClock {
        PimdirSearchClock {
            now: Some(NOW),
            zone: Some(zone.into()),
        }
    }

    /// The summaries of the events `text` matches, its days read in `zone`.
    fn events_in(client: &PimdirClient, text: &str, zone: &str) -> Vec<String> {
        let options = PimdirSearchOptions {
            clock: clock(zone),
            ..Default::default()
        };
        let page = client.search_items(query(text), None, &options).unwrap();

        let mut summaries: Vec<String> =
            page.hits.into_iter().map(|hit| hit.event.summary).collect();
        summaries.sort();
        summaries
    }

    /// A store of `items` in `cal`, indexed at [`NOW`] reading floating
    /// times and dates in `zone`.
    fn zoned_store(zone: &str, items: &[String]) -> (tempfile::TempDir, PimdirBackend) {
        let dir = tempfile::tempdir().unwrap();
        PimdirStore::open(dir.path())
            .unwrap()
            .ensure_collection("cal", "text/calendar")
            .unwrap();

        let mut backend = backend(dir.path(), None);
        for item in items {
            backend
                .create_item("cal", item.clone().into_bytes())
                .unwrap();
        }

        let mut owner = PimdirStore::open(dir.path()).unwrap().for_source("caldav");
        owner.drain().unwrap();
        drop(owner);
        PimdirIndex::open(dir.path())
            .unwrap()
            .refresh(&PimdirIndexRefreshOptions {
                now: Some(NOW),
                floating_zone: zone.into(),
                ..Default::default()
            })
            .unwrap();

        (dir, backend)
    }

    #[test]
    fn a_word_of_a_description_is_found_with_its_snippet() {
        let (_dir, backend) = indexed_store();

        let hits = page(&backend.client, "quarter", None).hits;
        assert_eq!(hits.len(), 1);
        let hit = &hits[0];
        assert_eq!(hit.event.summary, "Offsite");
        assert_eq!(hit.event.location, "Lisbon");
        assert_eq!(hit.calendar_id, "cal");
        assert_eq!(hit.matched, Some(PimdirCalendarMatched::Body));
        assert!(
            hit.snippet.as_deref().unwrap().contains("quarter"),
            "{hit:?}"
        );
        assert_eq!(
            hit.event.starts_at.as_deref(),
            Some("2026-11-07T00:30:00+00:00")
        );
        let read = backend
            .client
            .reader
            .get_item("cal", hit.event.id.parse().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(read.link_id.0, "offsite@example.org");

        assert_eq!(events(&backend.client, "location:lisbon"), ["Offsite"]);
        assert_eq!(
            events(&backend.client, "attendee:carol@example.org"),
            ["Offsite"]
        );
        assert_eq!(
            events(&backend.client, "organizer:alice@example.org"),
            ["Offsite"]
        );
    }

    #[test]
    fn a_weekly_series_begun_years_ago_is_found_on_a_later_week() {
        let (_dir, backend) = indexed_store();

        assert_eq!(events(&backend.client, "date:2026-11-09"), ["Stand-up"]);
        assert_eq!(events(&backend.client, "date:2026-11-16"), ["Stand-up"]);
        assert!(events(&backend.client, "date:2026-11-10").is_empty());
    }

    #[test]
    fn a_day_is_the_local_day() {
        let late = body(
            "VEVENT",
            "late",
            "DTSTART;TZID=Europe/Paris:20261107T003000\r\n\
             DTEND;TZID=Europe/Paris:20261107T013000\r\nSUMMARY:Late call\r\n",
        );
        let (_dir, backend) = zoned_store("Europe/Paris", &[late]);
        let client = &backend.client;

        assert_eq!(
            events_in(client, "date:2026-11-07", "Europe/Paris"),
            ["Late call"]
        );
        assert!(events_in(client, "date:2026-11-06", "Europe/Paris").is_empty());

        let count = client
            .count_items(query("date:2026-11-07"), None, &clock("Europe/Paris"))
            .unwrap();
        assert_eq!(count.count, 1);
        assert_eq!(count.zones, None);
    }

    #[test]
    fn an_all_day_event_stays_on_its_day() {
        let holiday = body(
            "VEVENT",
            "holiday",
            "DTSTART;VALUE=DATE:20261107\r\nDTEND;VALUE=DATE:20261108\r\n\
             SUMMARY:Holiday\r\n",
        );

        for zone in ["Europe/Paris", "America/New_York"] {
            let (_dir, backend) = zoned_store(zone, std::slice::from_ref(&holiday));
            let client = &backend.client;

            assert_eq!(events_in(client, "date:2026-11-07", zone), ["Holiday"]);
            assert!(
                events_in(client, "date:2026-11-06", zone).is_empty(),
                "{zone}"
            );
            assert!(
                events_in(client, "date:2026-11-08", zone).is_empty(),
                "{zone}"
            );
        }
    }

    #[test]
    fn an_index_read_in_another_zone_is_named() {
        let (_dir, backend) = indexed_store();
        let client = &backend.client;
        let zones = |clock: PimdirSearchClock| {
            let options = PimdirSearchOptions {
                clock: clock.clone(),
                ..Default::default()
            };
            let page = client
                .search_items(query("budget"), None, &options)
                .unwrap();
            let count = client.count_items(query("budget"), None, &clock).unwrap();
            assert_eq!(page.zones, count.zones);
            page.zones
        };

        let paris = zones(clock("Europe/Paris")).unwrap();
        assert_eq!(
            paris,
            PimdirCalendarZones {
                index: "UTC".into(),
                search: "Europe/Paris".into(),
            }
        );
        assert_eq!(
            paris.to_string(),
            "The index reads floating times and dates in UTC, this search reads days in \
             Europe/Paris\n"
        );

        assert_eq!(zones(clock("UTC")), None);
        assert_eq!(zones(PimdirSearchClock::default()), None);
    }

    /// The store holds a to-do and a journal entry naming the budget too.
    #[test]
    fn a_search_reads_events_alone() {
        let (_dir, backend) = indexed_store();

        assert_eq!(events(&backend.client, "budget"), ["Offsite", "Planning"]);
        assert!(events(&backend.client, "status:in-process").is_empty());

        let count = backend
            .client
            .count_items(query("budget"), None, &PimdirSearchClock::default())
            .unwrap();
        assert_eq!(count.count, 2);
    }

    #[test]
    fn a_bodiless_event_matches_on_its_summary() {
        let (_dir, backend) = indexed_store();

        let invoice = page(&backend.client, "invoice", None);
        let hits = &invoice.hits;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].matched, Some(PimdirCalendarMatched::Summary));
        assert_eq!(hits[0].calendar_id, "work");
        assert_eq!(hits[0].event.summary, "Invoice review");
        assert_eq!(hits[0].event.uid, "invoice@example.org");
        assert_eq!(
            hits[0].event.starts_at.as_deref(),
            Some("2026-11-20T14:00:00+00:00")
        );
        assert_eq!(
            hits[0].event.ends_at.as_deref(),
            Some("2026-11-20T15:00:00+00:00")
        );
        assert_eq!(invoice.coverage.bodiless, 1);
        assert!(!invoice.coverage.complete);
        assert!(
            invoice
                .coverage
                .to_string()
                .contains("1 item(s) have no local body"),
            "{}",
            invoice.coverage
        );
    }

    #[test]
    fn a_calendar_narrows_the_hits_and_the_coverage() {
        let (_dir, backend) = indexed_store();
        let client = &backend.client;

        assert_eq!(summaries(client, "budget", Some("work")), ["Planning"]);
        assert_eq!(summaries(client, "budget", Some("cal")), ["Offsite"]);

        let all = page(client, "budget", None).coverage;
        let cal = page(client, "budget", Some("cal")).coverage;
        assert_eq!(all.bodiless, 1);
        assert_eq!(cal.bodiless, 0);
        assert!(cal.complete, "{cal:?}");
        assert_eq!(cal.to_string(), "");
    }

    #[test]
    fn a_range_outside_the_horizon_is_reported() {
        let (_dir, backend) = indexed_store();

        let past = page(&backend.client, "date:2030", None).coverage;
        assert!(past.horizon_left);
        assert!(!past.complete);
        assert!(
            past.to_string().contains("outside the occurrences"),
            "{past}"
        );

        let within = page(&backend.client, "date:2026-11-09", Some("cal")).coverage;
        assert!(!within.horizon_left);
    }

    #[test]
    fn a_queued_event_is_not_found_until_drained_and_refreshed() {
        let (dir, mut backend) = indexed_store();
        let zeppelin = body(
            "VEVENT",
            "zeppelin",
            "DTSTART:20261201T100000Z\r\nSUMMARY:Zeppelin ride\r\n",
        );
        backend.create_item("cal", zeppelin.into_bytes()).unwrap();

        let queued = page(&backend.client, "zeppelin", None);
        assert!(
            events(&backend.client, "zeppelin").is_empty(),
            "a queued add is listed"
        );
        assert_eq!(queued.coverage.behind, 0);

        let mut owner = PimdirStore::open(dir.path()).unwrap().for_source("caldav");
        assert_eq!(owner.drain().unwrap().applied, 1);
        drop(owner);
        let drained = page(&backend.client, "zeppelin", None);
        assert!(
            events(&backend.client, "zeppelin").is_empty(),
            "an unindexed add is listed"
        );
        assert!(drained.coverage.behind > 0);
        assert!(!drained.coverage.complete);
        assert!(
            drained.coverage.to_string().contains("behind the store"),
            "{}",
            drained.coverage
        );

        refresh(dir.path());
        assert_eq!(events(&backend.client, "zeppelin"), ["Zeppelin ride"]);
    }

    #[test]
    fn a_search_reads_the_configured_account_alone() {
        let dir = tempfile::tempdir().unwrap();
        for account in ["work", "home"] {
            PimdirStore::open(dir.path())
                .unwrap()
                .for_account(account)
                .ensure_collection(format!("{account}/cal"), "text/calendar")
                .unwrap();
        }
        let (mut work, mut home) = (
            backend(dir.path(), Some("work")),
            backend(dir.path(), Some("home")),
        );
        let lunch = |uid| {
            body(
                "VEVENT",
                uid,
                "DTSTART:20261110T120000Z\r\nSUMMARY:Lunch\r\n",
            )
        };
        work.create_item("work/cal", lunch("w").into_bytes())
            .unwrap();
        home.create_item("home/cal", lunch("h").into_bytes())
            .unwrap();
        let mut owner = PimdirStore::open(dir.path()).unwrap().for_source("caldav");
        assert_eq!(owner.drain().unwrap().applied, 2);
        drop(owner);
        refresh(dir.path());

        for (backend, calendar) in [(&work, "work/cal"), (&home, "home/cal")] {
            let hits = page(&backend.client, "lunch", None).hits;
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].calendar_id, calendar);
        }
    }

    #[test]
    fn a_refused_query_names_its_code_in_both_renderings() {
        let long = "a".repeat(1025);
        let cases = [
            ("()", "empty-query"),
            ("frm:alice", "unknown-field"),
            ("status:done", "invalid-value"),
            ("date:2026-13-45", "invalid-date"),
            ("\"hello", "unterminated-phrase"),
            ("(hello", "unbalanced-parenthesis"),
            ("hello and", "dangling-operator"),
            ("he*llo", "invalid-wildcard"),
            (long.as_str(), "limit-exceeded"),
        ];

        for (text, code) in cases {
            for json in [true, false] {
                let err = parse_query(&[text.to_string()], json).err().unwrap();
                assert_eq!(code_of(&err), Some(ErrorCode::QueryRefused), "{text}");
                let shown = err.to_string();
                assert!(shown.contains(code), "{text}: {shown}");
                assert_eq!(shown.lines().count() == 1, json, "{shown}");
            }
        }

        let err = parse_query(&[String::from("frm:alice")], true)
            .err()
            .unwrap();
        assert!(
            err.to_string()
                .starts_with("Query refused, unknown-field at 0..4"),
            "{err}"
        );
        let drawn = parse_query(&[String::from("frm:alice")], false)
            .err()
            .unwrap()
            .to_string();
        // NOTE: a terminal stdout (a Nix builder's pty) colours the drawing.
        let drawn = strip_ansi(&drawn);
        assert!(drawn.contains("frm:alice"), "{drawn}");
        assert!(drawn.contains("fields: "), "{drawn}");
    }

    /// `text` without its ANSI escape sequences.
    fn strip_ansi(text: &str) -> String {
        let mut plain = String::with_capacity(text.len());
        let mut chars = text.chars();
        while let Some(c) = chars.next() {
            if c != '\x1b' {
                plain.push(c);
                continue;
            }
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        }
        plain
    }

    #[test]
    fn a_missing_or_outdated_index_says_to_run_the_sync_engine() {
        let (dir, backend) = indexed_store();
        let assert_coded = |code| {
            let options = PimdirSearchOptions::default();
            let err = backend
                .client
                .search_items(query("budget"), None, &options)
                .err()
                .unwrap();
            assert_eq!(code_of(&err), Some(code), "{err:#}");
            let count = backend
                .client
                .count_items(query("budget"), None, &PimdirSearchClock::default())
                .err()
                .unwrap();
            assert_eq!(code_of(&count), Some(code), "{count:#}");
            let err = format!("{err:#}");
            assert!(err.contains("sync engine"), "{err}");
            assert!(!err.contains("delete"), "{err}");
            err
        };
        let index = dir.path().join("index.db");

        // NOTE: another store's index reads as outdated.
        let (other, _) = indexed_store();
        std::fs::copy(other.path().join("index.db"), &index).unwrap();
        assert_coded(ErrorCode::IndexOutdated);

        // NOTE: the version is spelled once, overwritten in place.
        refresh(dir.path());
        let ours = io_pimdir::sql::search::FORMAT_VERSION.as_bytes();
        let mut db = std::fs::read(&index).unwrap();
        let at: Vec<usize> = db
            .windows(ours.len())
            .enumerate()
            .filter(|(_, window)| *window == ours)
            .map(|(at, _)| at)
            .collect();
        assert_eq!(at.len(), 1, "the version is spelled once");
        db[at[0]..at[0] + ours.len()].copy_from_slice(b"search-draft-08");
        std::fs::write(&index, db).unwrap();
        let err = assert_coded(ErrorCode::IndexVersion);
        assert!(err.contains("search-draft-08"), "{err}");

        for name in ["index.db", "index.db-wal", "index.db-shm"] {
            let _ = std::fs::remove_file(dir.path().join(name));
        }
        assert_coded(ErrorCode::IndexMissing);
        assert!(!index.exists(), "the index was built");
    }

    #[test]
    fn a_complete_answer_prints_no_coverage_line() {
        let coverage = PimdirCalendarCoverage::from(&PimdirSearchCoverage::default());

        assert!(coverage.complete);
        assert_eq!(coverage.to_string(), "");
    }

    #[test]
    fn the_lines_under_a_table_defuse_what_the_store_holds() {
        let zones = PimdirCalendarZones {
            index: "UTC\x1b[2J".into(),
            search: "Europe/Paris".into(),
        };
        let line = zones.to_string();
        assert!(line.contains("in UTC\u{fffd}[2J, this"), "{line:?}");

        let coverage = PimdirCalendarCoverage {
            since: Some("\x1b]0;x\x07".into()),
            until: Some("2026\u{202e}".into()),
            ..Default::default()
        };
        let line = coverage.to_string();
        assert!(line.contains("before \u{fffd}]0;x\u{fffd} are"), "{line:?}");
        assert!(line.contains("from 2026\u{fffd} on"), "{line:?}");
    }

    #[test]
    fn coverage_renders_as_camel_case_json_and_one_line() {
        let coverage = PimdirCalendarCoverage {
            complete: false,
            bodiless: 2,
            unindexed: 1,
            behind: 3,
            since: Some("2026-05-15T00:00:00Z".into()),
            until: None,
            filling: true,
            horizon_left: true,
            cut: 4,
        };

        assert_eq!(
            serde_json::to_value(&coverage).unwrap(),
            serde_json::json!({
                "complete": false,
                "bodiless": 2,
                "unindexed": 1,
                "behind": 3,
                "since": "2026-05-15T00:00:00Z",
                "until": null,
                "filling": true,
                "horizonLeft": true,
                "cut": 4,
            })
        );
        let line = coverage.to_string();
        assert_eq!(line.lines().count(), 1, "{line}");
        for part in [
            "2 item(s) have no local body",
            "items before 2026-05-15 are not synced",
            "a first sync is still listing a calendar",
            "the dates reach outside the occurrences the index holds",
            "4 series stop their occurrences before the range ends",
            "1 item(s) changed since the index was refreshed",
            "the index is 3 change(s) behind the store",
        ] {
            assert!(line.contains(part), "{line}");
        }
    }
}
