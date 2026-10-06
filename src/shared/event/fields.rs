//! # Event fields
//!
//! The VEVENT properties a build, a create or an update can set from the
//! command line, and how they are written onto an event.
//!
//! They are a convenience over the common fields rather than the whole of
//! iCalendar: the composer is the complete surface, which is why a
//! recurrence (`RRULE`, `RDATE`, `EXDATE`, overrides), an alarm, a
//! conference link (`CONFERENCE`) and a hand-written `VTIMEZONE` are
//! deliberately not a flag. `--online-meeting` asks the sync engine for a
//! meeting it creates itself, which is not writing one.

use std::borrow::Cow;

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use ical::{
    component::{vevent::VEVENT, vtimezone::VTIMEZONE},
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    tree::{
        codec::mode::Escaper,
        cst::{IcalCst, IcalItem},
        line::IcalLine,
    },
    tzdb,
    value::{
        IcalValue,
        cal_address::IcalCalAddress,
        datetime::{IcalDate, IcalDateTime},
        duration::IcalDuration,
        integer::IcalInteger,
        text::{IcalText, IcalTextList},
        uri::IcalUri,
    },
};
use jiff::{
    Timestamp, ToSpan,
    civil::{Date, DateTime},
    tz::TimeZone,
};

use crate::shared::{event::Event, item::CalendarItem};

/// How long a content line may run before it is folded (RFC 5545 3.1).
const FOLD_OCTETS: usize = 75;

/// The VEVENT fields `event build`, `event create` and `event update` set
/// from flags.
///
/// Each flag names one property, and writing it replaces every instance
/// of that property the event already carried. An empty value given to
/// a text or a person flag removes the property instead.
#[derive(Debug, Parser)]
pub struct EventFieldsArgs {
    /// Identity of the event (`UID`).
    ///
    /// An event minted from nothing gets a fresh UUID when this names
    /// none.
    #[arg(long, value_name = "TEXT")]
    pub uid: Option<String>,
    /// Title of the event (`SUMMARY`); empty removes it.
    #[arg(long, value_name = "TEXT")]
    pub summary: Option<String>,
    /// Free-form description (`DESCRIPTION`); empty removes it.
    #[arg(long, value_name = "TEXT")]
    pub description: Option<String>,
    /// Where the event takes place (`LOCATION`); empty removes it.
    #[arg(long, value_name = "TEXT")]
    pub location: Option<String>,
    /// Web page of the event (`URL`); empty removes it.
    #[arg(long, value_name = "URL")]
    pub url: Option<String>,
    /// Start of the event (`DTSTART`).
    ///
    /// `YYYY-MM-DD` for a whole day, `YYYY-MM-DDTHH:MM[:SS]` for a local
    /// time, the same with a trailing `Z` for UTC, or a time prefixed by
    /// an IANA zone: `Europe/Paris:2026-10-19T09:00`. A zone arrives with
    /// its `VTIMEZONE`.
    ///
    /// On `event update`, given without `--end` or `--duration`, the end
    /// moves with it and the event keeps its length.
    #[arg(long, value_name = "TIME")]
    pub start: Option<String>,
    /// End of the event (`DTEND`), spelled as `--start` is.
    ///
    /// Exclusive, a whole day's included: a one-day event on the 19th
    /// ends on the 20th. Replaces any `DURATION`.
    #[arg(long, value_name = "TIME", conflicts_with = "duration")]
    pub end: Option<String>,
    /// Length of the event (`DURATION`), instead of `--end`.
    ///
    /// An RFC 5545 duration: `PT1H30M`, `P1D`, `P2W`. Replaces any
    /// `DTEND`.
    #[arg(long, value_name = "DURATION")]
    pub duration: Option<String>,
    /// IANA zone of the `--start` and `--end` that name none, such as
    /// `Europe/Paris`.
    ///
    /// Without it such a time is floating: the same wall-clock time
    /// wherever it is read.
    #[arg(long, value_name = "ZONE")]
    pub time_zone: Option<String>,
    /// Status of the event (`STATUS`).
    #[arg(long, value_name = "STATUS")]
    pub status: Option<EventStatusArg>,
    /// Whether the event blocks time (`TRANSP`).
    #[arg(long, value_name = "TRANSPARENCY")]
    pub transparency: Option<EventTransparencyArg>,
    /// Organizer of the event (`ORGANIZER`).
    ///
    /// An email address, written as a `mailto:` URI, optionally prefixed
    /// by a name: `Jane Doe:jane@example.org`. Empty removes it.
    #[arg(long, value_name = "[NAME:]ADDRESS")]
    pub organizer: Option<String>,
    /// Attendees (`ATTENDEE`), the flag repeating, each spelled as
    /// `--organizer` is.
    ///
    /// Each is invited plainly: `PARTSTAT=NEEDS-ACTION` and `RSVP=TRUE`,
    /// no role, which the composer sets. `--attendee ""` alone removes
    /// every attendee.
    #[arg(long, value_name = "[NAME:]ADDRESS")]
    pub attendee: Vec<String>,
    /// Categories (`CATEGORIES`), the flag repeating; `--categories ""`
    /// alone removes them.
    #[arg(long, value_name = "TEXT")]
    pub categories: Vec<String>,
    /// Revision of the event (`SEQUENCE`), which an organizer bumps on a
    /// significant change.
    #[arg(long, value_name = "NUMBER")]
    pub sequence: Option<u32>,
    /// Ask for an online meeting (`X-PIMDIR-ONLINE-MEETING:TRUE`, pimdir
    /// STORAGE Annex B.1), which the sync engine creates with the
    /// provider's own service (Google Meet, Microsoft Teams) and brings
    /// back as a `CONFERENCE` link.
    ///
    /// Served by the pimdir backend alone, and only when a source of the
    /// store declares `calendar.online-meeting`; every other backend
    /// refuses it.
    #[arg(long)]
    pub online_meeting: bool,
}

/// The status an event can be set to.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum EventStatusArg {
    /// Not settled yet (`TENTATIVE`).
    Tentative,
    /// Settled (`CONFIRMED`).
    Confirmed,
    /// Called off (`CANCELLED`).
    Cancelled,
}

impl EventStatusArg {
    /// The `STATUS` value.
    pub fn as_status(self) -> &'static str {
        match self {
            Self::Tentative => "TENTATIVE",
            Self::Confirmed => "CONFIRMED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

/// Whether an event blocks time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum EventTransparencyArg {
    /// Busy (`OPAQUE`).
    Opaque,
    /// Free (`TRANSPARENT`).
    Transparent,
}

impl EventTransparencyArg {
    /// The `TRANSP` value.
    pub fn as_transp(self) -> &'static str {
        match self {
            Self::Opaque => "OPAQUE",
            Self::Transparent => "TRANSPARENT",
        }
    }
}

/// The scheduling method a built calendar can carry (RFC 5546 1.4).
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum EventMethodArg {
    /// Published, no reply expected (`PUBLISH`).
    Publish,
    /// An invitation, or an update to one (`REQUEST`).
    Request,
    /// An attendee's answer (`REPLY`).
    Reply,
    /// New instances for an existing event (`ADD`).
    Add,
    /// A cancellation (`CANCEL`).
    Cancel,
    /// An attendee asking for the latest version (`REFRESH`).
    Refresh,
    /// An attendee's counter-proposal (`COUNTER`).
    Counter,
    /// The organizer declining a counter-proposal (`DECLINECOUNTER`).
    DeclineCounter,
}

impl EventMethodArg {
    /// The `METHOD` value.
    pub fn as_method(self) -> &'static str {
        match self {
            Self::Publish => "PUBLISH",
            Self::Request => "REQUEST",
            Self::Reply => "REPLY",
            Self::Add => "ADD",
            Self::Cancel => "CANCEL",
            Self::Refresh => "REFRESH",
            Self::Counter => "COUNTER",
            Self::DeclineCounter => "DECLINECOUNTER",
        }
    }
}

impl EventFieldsArgs {
    /// Whether no field flag was given, the event then passing through.
    pub fn is_empty(&self) -> bool {
        self.uid.is_none()
            && self.summary.is_none()
            && self.description.is_none()
            && self.location.is_none()
            && self.url.is_none()
            && self.start.is_none()
            && self.end.is_none()
            && self.duration.is_none()
            && self.time_zone.is_none()
            && self.status.is_none()
            && self.transparency.is_none()
            && self.organizer.is_none()
            && self.attendee.is_empty()
            && self.categories.is_empty()
            && self.sequence.is_none()
            && !self.online_meeting
    }

    /// Refuses `--online-meeting` on a backend that cannot serve it,
    /// `backend` naming the one serving the account.
    ///
    /// Only a pimdir store hands the ask to a sync engine able to create
    /// the meeting; anywhere else the property would be written and never
    /// acted upon. Whether the store's sources can is the pimdir
    /// backend's own check, made before it queues the write.
    pub fn check_backend(&self, backend: &str) -> Result<()> {
        if self.online_meeting && backend != "pimdir" {
            bail!(
                "--online-meeting needs the pimdir backend, whose sync engine creates the \
                 meeting; the {backend} backend serves this account"
            );
        }

        Ok(())
    }

    /// Writes the flags onto the VEVENT of `ical`, returning its new
    /// bytes.
    ///
    /// Every instance of a property a flag names is dropped and the flag's
    /// own is written in place of the first, so a flag sets that property
    /// rather than adding to it; an empty value writes nothing in its
    /// place, which removes it. Every other line keeps the event's own
    /// bytes, the properties no flag covers included.
    pub fn apply(&self, ical: &[u8]) -> Result<Vec<u8>> {
        self.write(ical, false)
    }

    /// Writes the flags as [`apply`](Self::apply) does, a `--start` given
    /// alone moving the end with it.
    ///
    /// This is what an update means by a new start: the event moves, and
    /// keeps its length. A `DTEND` is rewritten that far from the new
    /// start, a `DURATION` is kept as it is, and an event carrying
    /// neither has no end to move.
    pub fn apply_keeping_length(&self, ical: &[u8]) -> Result<Vec<u8>> {
        self.write(ical, true)
    }

    fn write(&self, ical: &[u8], keep_length: bool) -> Result<Vec<u8>> {
        if self.is_empty() {
            return Ok(ical.to_vec());
        }

        // NOTE: a flag rewrites one event, and the parser reads the first
        // calendar alone, so a source holding several events would come
        // back with all but one dropped or left inconsistent with it.
        // Refusing is the one outcome worth having.
        let events: usize = IcalCst::parse_many(ical)
            .filter_map(|cst| cst.ok())
            .map(|cst| cst.components::<VEVENT>().count())
            .sum();

        if events > 1 {
            bail!("Cannot apply a field flag to a source holding several VEVENTs");
        }

        let mut cst = IcalCst::parse(ical).context("Parse iCalendar error")?;

        if events == 0 || cst.component::<VEVENT>().is_none() {
            bail!("Cannot apply a field flag to a source holding no VEVENT");
        }

        let escaper = Escaper::for_version(cst.version());
        let times = self.times(ical, keep_length)?;

        let written = self.props(&times)?;
        let mut dropped: Vec<IcalPropName<'static>> =
            written.iter().map(|(name, _)| name.clone()).collect();

        // NOTE: an end and a length are one fact spelled twice (RFC 5545
        // 3.6.1 allows either, never both), so setting one clears the
        // other.
        if self.end.is_some() || times.moved_end.is_some() {
            dropped.push(IcalPropName::Kind(IcalPropKind::Duration));
        }

        if self.duration.is_some() {
            dropped.push(IcalPropName::Kind(IcalPropKind::DtEnd));
        }

        let lines: Lines = written
            .into_iter()
            .map(|(name, props)| {
                let items = props
                    .iter()
                    .map(|prop| folded(prop.encode(escaper)))
                    .collect::<Result<Vec<_>>>()?;
                Ok((name, items))
            })
            .collect::<Result<_>>()?;

        let vevent = cst.component_mut::<VEVENT>().context("Find VEVENT error")?;

        replace(vevent, &dropped, lines);
        define_zones(&mut cst, &times.zones);

        Ok(cst.to_bytes())
    }

    /// The times the flags name, resolved against the zone flag, and the
    /// end an update moves.
    fn times(&self, ical: &[u8], keep_length: bool) -> Result<Times> {
        let zone = match &self.time_zone {
            Some(name) => Some(Zone::named(name)?),
            None => None,
        };

        if zone.is_some() && self.start.is_none() && self.end.is_none() {
            bail!("--time-zone names the zone of --start and --end; give one of them");
        }

        let start = match &self.start {
            Some(value) => Some(Moment::parse(value, zone.as_ref())?),
            None => None,
        };

        let end = match &self.end {
            Some(value) => Some(Moment::parse(value, zone.as_ref())?),
            None => None,
        };

        if let (Some(start), Some(end)) = (&start, &end) {
            if start.is_date() != end.is_date() {
                bail!("--start and --end must both be dates or both be times");
            }

            if let (Some(from), Some(to)) = (start.comparable(end), end.comparable(start))
                && to < from
            {
                bail!("--end comes before --start");
            }
        }

        let duration = match &self.duration {
            Some(value) => Some(duration(value)?),
            None => None,
        };

        let moved_end = match &start {
            Some(start) if keep_length && end.is_none() && duration.is_none() => {
                moved_end(ical, start)?
            }
            _ => None,
        };

        let mut zones: Vec<(String, i64)> = Vec::new();

        for moment in [&start, &end, &moved_end].into_iter().flatten() {
            if let Moment::Zoned(zone, _) = moment
                && !zones.iter().any(|(tzid, _)| *tzid == zone.tzid)
            {
                zones.push((zone.tzid.clone(), moment.anchor()));
            }
        }

        Ok(Times {
            start,
            end,
            moved_end,
            duration,
            zones,
        })
    }

    /// The properties the flags name, grouped by name, in a stable order.
    ///
    /// A name holding no property is one an empty flag removes: it is
    /// dropped and nothing is written in its place.
    fn props(&self, times: &Times) -> Result<Vec<(IcalPropName<'static>, Vec<IcalProp<'static>>)>> {
        let mut props = Vec::new();
        let mut push = |kind: IcalPropKind, values: Vec<IcalProp<'static>>| {
            props.push((IcalPropName::Kind(kind), values));
        };

        if let Some(uid) = &self.uid {
            push(IcalPropKind::Uid, vec![text(IcalPropKind::Uid, uid)]);
        }

        for (kind, value) in [
            (IcalPropKind::Summary, &self.summary),
            (IcalPropKind::Description, &self.description),
            (IcalPropKind::Location, &self.location),
        ] {
            if let Some(value) = value {
                push(kind, unless_empty(value, |value| text(kind, value)));
            }
        }

        if let Some(url) = &self.url {
            let url = unless_empty(url, |url| {
                let value = IcalValue::Uri(IcalUri(Cow::Owned(url.to_owned())));
                prop(IcalPropKind::Url, vec![], value)
            });
            push(IcalPropKind::Url, url);
        }

        if let Some(start) = &times.start {
            push(
                IcalPropKind::DtStart,
                vec![start.prop(IcalPropKind::DtStart)],
            );
        }

        if let Some(end) = times.end.as_ref().or(times.moved_end.as_ref()) {
            push(IcalPropKind::DtEnd, vec![end.prop(IcalPropKind::DtEnd)]);
        }

        if let Some(duration) = &times.duration {
            let value = IcalValue::Duration(IcalDuration(Cow::Owned(duration.clone())));
            let prop = prop(IcalPropKind::Duration, vec![], value);
            push(IcalPropKind::Duration, vec![prop]);
        }

        if let Some(status) = self.status {
            let prop = text(IcalPropKind::Status, status.as_status());
            push(IcalPropKind::Status, vec![prop]);
        }

        if let Some(transparency) = self.transparency {
            let prop = text(IcalPropKind::Transp, transparency.as_transp());
            push(IcalPropKind::Transp, vec![prop]);
        }

        if let Some(organizer) = &self.organizer {
            let mut organizers = Vec::new();

            if !organizer.is_empty() {
                let (name, address) = person(organizer)?;
                let params = name.map(IcalParam::Cn).into_iter().collect();
                organizers.push(prop(IcalPropKind::Organizer, params, address));
            }

            push(IcalPropKind::Organizer, organizers);
        }

        if !self.attendee.is_empty() {
            let mut attendees = Vec::new();

            // NOTE: an empty value names nobody, so `--attendee ""` alone
            // leaves the event inviting no one.
            for attendee in self.attendee.iter().filter(|value| !value.is_empty()) {
                let (name, address) = person(attendee)?;
                let mut params: Vec<IcalParam<'static>> =
                    name.map(IcalParam::Cn).into_iter().collect();
                params.push(IcalParam::PartStat(Cow::Borrowed("NEEDS-ACTION")));
                params.push(IcalParam::Rsvp(Cow::Borrowed("TRUE")));
                attendees.push(prop(IcalPropKind::Attendee, params, address));
            }

            push(IcalPropKind::Attendee, attendees);
        }

        if !self.categories.is_empty() {
            let values: Vec<Cow<'static, str>> = self
                .categories
                .iter()
                .filter(|value| !value.is_empty())
                .map(|value| Cow::Owned(value.clone()))
                .collect();

            let categories = match values.is_empty() {
                true => Vec::new(),
                false => {
                    let value = IcalValue::TextList(IcalTextList(values));
                    vec![prop(IcalPropKind::Categories, vec![], value)]
                }
            };

            push(IcalPropKind::Categories, categories);
        }

        if let Some(sequence) = self.sequence {
            let value = IcalValue::Integer(IcalInteger(Cow::Owned(sequence.to_string())));
            let prop = prop(IcalPropKind::Sequence, vec![], value);
            push(IcalPropKind::Sequence, vec![prop]);
        }

        if self.online_meeting {
            let name = IcalPropName::Unknown(Cow::Borrowed(ONLINE_MEETING));
            let ask = IcalProp {
                name: name.clone(),
                params: Vec::new(),
                value: IcalValue::Text(IcalText(Cow::Borrowed("TRUE"))),
            };
            props.push((name, vec![ask]));
        }

        Ok(props)
    }
}

/// The property asking the sync engine for an online meeting (pimdir
/// STORAGE Annex B.1).
pub const ONLINE_MEETING: &str = "X-PIMDIR-ONLINE-MEETING";

/// Encoded lines, grouped by the property they spell.
type Lines = Vec<(IcalPropName<'static>, Vec<IcalItem<'static>>)>;

/// The one property a value stands for, none when it is empty: what an
/// empty flag writes is its property's removal.
fn unless_empty(
    value: &str,
    prop: impl FnOnce(&str) -> IcalProp<'static>,
) -> Vec<IcalProp<'static>> {
    match value.is_empty() {
        true => Vec::new(),
        false => vec![prop(value)],
    }
}

/// Marks every VEVENT of `ical` as revised at `now`, returning its new
/// bytes (RFC 5545 3.8.7.2, 3.8.7.3).
///
/// `DTSTAMP` is set to `now` in UTC, where it stood, or after the
/// event's last property when it carried none; `LAST-MODIFIED` is set
/// to the same instant only where the event carries one. Every other
/// line keeps its bytes, and a source holding no VEVENT comes back as it
/// was, as does one the parser cannot structure.
pub fn revise(ical: &[u8], now: Timestamp) -> Result<Vec<u8>> {
    let mut recovery = IcalCst::parse_recovering(ical);

    if !recovery.is_clean() {
        return Ok(ical.to_vec());
    }

    let stamp = IcalValue::DateTime(IcalDateTime(Cow::Owned(
        now.strftime("%Y%m%dT%H%M%SZ").to_string(),
    )));

    for cst in &mut recovery.calendars {
        let escaper = Escaper::for_version(cst.version());

        for item in &mut cst.items {
            if !is_vevent(item) {
                continue;
            }

            let IcalItem::Component(vevent) = item else {
                continue;
            };

            let mut kinds = vec![IcalPropKind::DtStamp];

            if vevent
                .items
                .iter()
                .any(|item| names(item, &IcalPropKind::LastModified))
            {
                kinds.push(IcalPropKind::LastModified);
            }

            let mut dropped = Vec::new();
            let mut lines: Lines = Vec::new();

            for kind in kinds {
                let line = folded(prop(kind, Vec::new(), stamp.clone()).encode(escaper))?;
                dropped.push(IcalPropName::Kind(kind));
                lines.push((IcalPropName::Kind(kind), vec![line]));
            }

            replace(vevent, &dropped, lines);
        }
    }

    Ok(recovery.to_bytes())
}

/// Sets the calendar's `METHOD` (RFC 5546), returning its new bytes.
///
/// The one flag that writes on the calendar rather than on its event, so
/// it holds for a source of several events too. Every `METHOD` the
/// calendar carried is dropped, and every other line keeps its bytes.
pub fn set_method(ical: &[u8], method: EventMethodArg) -> Result<Vec<u8>> {
    let mut cst = IcalCst::parse(ical).context("Parse iCalendar error")?;
    let escaper = Escaper::for_version(cst.version());
    let value = IcalValue::Text(IcalText(Cow::Borrowed(method.as_method())));
    let line = folded(prop(IcalPropKind::Method, vec![], value).encode(escaper))?;

    replace(
        &mut cst,
        &[IcalPropName::Kind(IcalPropKind::Method)],
        vec![(IcalPropName::Kind(IcalPropKind::Method), vec![line])],
    );

    Ok(cst.to_bytes())
}

/// What the time flags resolve to.
#[derive(Debug, Default)]
struct Times {
    /// The `--start`.
    start: Option<Moment>,
    /// The `--end`.
    end: Option<Moment>,
    /// The end an update moves with its start.
    moved_end: Option<Moment>,
    /// The `--duration`, validated.
    duration: Option<String>,
    /// Every zone a time names, and the instant its `VTIMEZONE` is
    /// described around.
    zones: Vec<(String, i64)>,
}

/// An IANA zone a flag names.
#[derive(Clone, Debug)]
struct Zone {
    /// The name, as the database spells it.
    tzid: String,
    /// The zone itself.
    tz: TimeZone,
}

impl Zone {
    /// Resolves an IANA name, refusing one the database does not know.
    fn named(name: &str) -> Result<Self> {
        let Ok(tz) = TimeZone::get(name) else {
            bail!("Unknown time zone `{name}`; give an IANA name such as `Europe/Paris`");
        };

        let tzid = tz.iana_name().unwrap_or(name).to_owned();

        Ok(Self { tzid, tz })
    }
}

/// A time a flag names.
#[derive(Clone, Debug)]
enum Moment {
    /// A whole day.
    Date(Date),
    /// A local time, read wherever it is read.
    Floating(DateTime),
    /// A time in UTC.
    Utc(DateTime),
    /// A time in a named zone.
    Zoned(Zone, DateTime),
}

impl Moment {
    /// Parses a time flag, a time naming no zone taking `zone`'s.
    fn parse(value: &str, zone: Option<&Zone>) -> Result<Self> {
        let value = value.trim();

        // NOTE: a time starts with its year, and a zone name never starts
        // with a digit, so a leading letter announces a zone prefix.
        let (prefix, time) = match value.starts_with(|c: char| c.is_ascii_alphabetic()) {
            true => match value.split_once(':') {
                Some((prefix, time)) => (Some(Zone::named(prefix)?), time),
                None => bail!("Cannot read time `{value}`; {}", Self::SHAPES),
            },
            false => (None, value),
        };

        if let Ok(date) = Date::strptime("%Y-%m-%d", time) {
            if prefix.is_some() {
                bail!("Cannot read time `{value}`: a whole day names no zone");
            }

            return Ok(Self::Date(date));
        }

        let (time, utc) = match time.strip_suffix(['Z', 'z']) {
            Some(time) => (time, true),
            None => (time, false),
        };

        let Some(civil) = ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"]
            .iter()
            .find_map(|format| DateTime::strptime(format, time).ok())
        else {
            bail!("Cannot read time `{value}`; {}", Self::SHAPES);
        };

        match (prefix, utc) {
            (Some(_), true) => bail!("Cannot read time `{value}`: it names a zone and UTC both"),
            (Some(zone), false) => Ok(Self::Zoned(zone, civil)),
            (None, true) => Ok(Self::Utc(civil)),
            (None, false) => match zone {
                Some(zone) => Ok(Self::Zoned(zone.clone(), civil)),
                None => Ok(Self::Floating(civil)),
            },
        }
    }

    /// The shapes a time flag takes, for an error naming them.
    const SHAPES: &str = "give YYYY-MM-DD, YYYY-MM-DDTHH:MM[:SS], the same with a trailing Z, \
                          or prefixed by an IANA zone (Europe/Paris:2026-10-19T09:00)";

    /// Whether the time is a whole day.
    fn is_date(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    /// The instant the time stands for, a whole day and a local time read
    /// as UTC.
    fn anchor(&self) -> i64 {
        let instant = match self {
            Self::Date(date) => date.to_zoned(TimeZone::UTC).map(|at| at.timestamp()),
            Self::Floating(civil) | Self::Utc(civil) => {
                civil.to_zoned(TimeZone::UTC).map(|at| at.timestamp())
            }
            Self::Zoned(zone, civil) => zone
                .tz
                .to_ambiguous_zoned(*civil)
                .compatible()
                .map(|at| at.timestamp()),
        };

        instant.map(Timestamp::as_second).unwrap_or_default()
    }

    /// The time as a number `other` compares to, `None` when the two are
    /// not on one timeline (a local time against a zoned one).
    fn comparable(&self, other: &Self) -> Option<i64> {
        let absolute = |moment: &Self| matches!(moment, Self::Utc(_) | Self::Zoned(..));

        match (self, other) {
            (Self::Date(_), Self::Date(_)) | (Self::Floating(_), Self::Floating(_)) => {
                Some(self.anchor())
            }
            (this, that) if absolute(this) && absolute(that) => Some(self.anchor()),
            _ => None,
        }
    }

    /// The time `seconds` later, spelled the same way.
    ///
    /// A whole day moves by whole days, rounded up so an event keeps at
    /// least the day it covered. A local time moves on the wall clock, a
    /// zoned one on the timeline.
    fn plus(&self, seconds: i64) -> Result<Self> {
        let moved = match self {
            Self::Date(date) => {
                let days = (seconds + 86_399).div_euclid(86_400).max(1);
                Self::Date(date.checked_add(days.days())?)
            }
            Self::Floating(civil) => Self::Floating(civil.checked_add(seconds.seconds())?),
            Self::Utc(civil) => Self::Utc(civil.checked_add(seconds.seconds())?),
            Self::Zoned(zone, civil) => {
                let at = zone.tz.to_ambiguous_zoned(*civil).compatible()?;
                let at = at.timestamp().checked_add(seconds.seconds())?;
                Self::Zoned(zone.clone(), at.to_zoned(zone.tz.clone()).datetime())
            }
        };

        Ok(moved)
    }

    /// The property spelling the time: `VALUE=DATE` for a whole day, a
    /// trailing `Z` for UTC, a `TZID` for a zone.
    fn prop(&self, kind: IcalPropKind) -> IcalProp<'static> {
        let stamp = |civil: &DateTime| civil.strftime("%Y%m%dT%H%M%S").to_string();

        let (params, value) = match self {
            Self::Date(date) => (
                vec![IcalParam::Value(Cow::Borrowed("DATE"))],
                IcalValue::Date(IcalDate(Cow::Owned(date.strftime("%Y%m%d").to_string()))),
            ),
            Self::Floating(civil) => (
                Vec::new(),
                IcalValue::DateTime(IcalDateTime(Cow::Owned(stamp(civil)))),
            ),
            Self::Utc(civil) => (
                Vec::new(),
                IcalValue::DateTime(IcalDateTime(Cow::Owned(format!("{}Z", stamp(civil))))),
            ),
            Self::Zoned(zone, civil) => (
                vec![IcalParam::TzId(Cow::Owned(zone.tzid.clone()))],
                IcalValue::DateTime(IcalDateTime(Cow::Owned(stamp(civil)))),
            ),
        };

        prop(kind, params, value)
    }
}

/// The end an update moves with its new start, `None` when the event
/// carries no `DTEND` to move.
///
/// The length is read through the event projection, so a zone the
/// calendar defines itself counts as it does when the event is listed.
fn moved_end(ical: &[u8], start: &Moment) -> Result<Option<Moment>> {
    let cst = IcalCst::parse(ical).context("Parse iCalendar error")?;
    let Some(vevent) = cst.component::<VEVENT>() else {
        return Ok(None);
    };

    let carries = |kind: IcalPropKind| vevent.items.iter().any(|item| names(item, &kind));

    if !carries(IcalPropKind::DtEnd) || carries(IcalPropKind::Duration) {
        return Ok(None);
    }

    let item = CalendarItem {
        contents: ical.to_vec(),
        ..Default::default()
    };

    let event = Event::project(&item).into_iter().next();
    let length = event.and_then(|event| Some(event.end_secs? - event.start_secs?));

    let Some(length) = length else {
        bail!("Cannot read the length of the event to move its end; give --end or --duration");
    };

    Ok(Some(start.plus(length.max(0))?))
}

/// Validates an RFC 5545 duration (3.3.6), uppercased.
///
/// Strict where ical-rs's reading is liberal: what a flag writes has to
/// be the grammar, not something a reader happens to tolerate. A negative
/// length is no length for an event.
fn duration(value: &str) -> Result<String> {
    let duration = value.trim().to_ascii_uppercase();
    let span = duration.strip_prefix('+').unwrap_or(&duration);

    let valid = match span.strip_prefix('P') {
        Some(rest) => match rest.split_once('T') {
            Some((date, time)) => units(date, "D") && !time.is_empty() && units(time, "HMS"),
            None => (units(rest, "D") || units(rest, "W")) && !rest.is_empty(),
        },
        None => false,
    };

    if !valid {
        bail!(
            "Cannot read duration `{value}`; give an RFC 5545 duration such as PT1H30M, P1D or P2W"
        );
    }

    Ok(span.to_owned())
}

/// Whether `value` is numbers each followed by one of `units`, in that
/// order, none twice; empty is allowed.
fn units(value: &str, units: &str) -> bool {
    let mut allowed = units.chars();
    let mut digits = false;

    for character in value.chars() {
        if character.is_ascii_digit() {
            digits = true;
            continue;
        }

        if !digits || !allowed.any(|unit| unit == character) {
            return false;
        }

        digits = false;
    }

    !digits
}

/// Splits `[NAME:]ADDRESS` into the `CN` and the `mailto:` URI.
fn person(value: &str) -> Result<(Option<Cow<'static, str>>, IcalValue<'static>)> {
    let bare = strip_mailto(value.trim());

    let (name, address) = match bare.split_once(':') {
        Some((name, address)) if !value.trim().to_ascii_lowercase().starts_with("mailto:") => {
            (Some(name.trim()), strip_mailto(address.trim()))
        }
        _ => (None, bare),
    };

    let address = address.trim();

    if !address.contains('@') || address.contains([':', ' ']) {
        bail!("Cannot read `{value}`: give an email address, optionally named as NAME:ADDRESS");
    }

    let name = name
        .filter(|name| !name.is_empty())
        .map(|name| Cow::Owned(name.to_owned()));
    let address = IcalValue::CalAddress(IcalCalAddress(Cow::Owned(format!("mailto:{address}"))));

    Ok((name, address))
}

/// Drops a leading `mailto:`, in any case.
fn strip_mailto(value: &str) -> &str {
    match value.get(..7) {
        Some(scheme) if scheme.eq_ignore_ascii_case("mailto:") => &value[7..],
        _ => value,
    }
}

/// Drops every line of `dropped` from a component and writes `lines` in
/// their place.
///
/// The lines of a kind land where its first instance stood, so a flag
/// leaves the layout as it found it; a kind the component did not carry
/// lands after its last property, ahead of any nested component, as RFC
/// 5545 orders a component's body.
fn replace(component: &mut IcalCst<'_>, dropped: &[IcalPropName<'static>], mut lines: Lines) {
    let items = std::mem::take(&mut component.items);
    let mut kept = Vec::with_capacity(items.len());

    for item in items {
        let Some(name) = dropped.iter().find(|name| names(&item, name)) else {
            kept.push(item);
            continue;
        };

        if let Some(index) = lines
            .iter()
            .position(|(written, _)| written.eq_ignore_ascii_case(name))
        {
            kept.extend(lines.remove(index).1);
        }
    }

    let at = kept
        .iter()
        .position(|item| matches!(item, IcalItem::Component(_)))
        .unwrap_or(kept.len());

    kept.splice(at..at, lines.into_iter().flat_map(|(_, items)| items));
    component.items = kept;
}

/// Gives every zone a flag named its `VTIMEZONE`, ahead of the VEVENT.
///
/// One the calendar already defines is left as it is, and one minted is
/// minted once, described around the first time naming it.
fn define_zones(cst: &mut IcalCst<'_>, zones: &[(String, i64)]) {
    for (tzid, anchor) in zones {
        let defined = cst.components::<VTIMEZONE>().any(|vtimezone| {
            vtimezone
                .items
                .iter()
                .filter_map(|item| match item {
                    IcalItem::Prop(line) if line.name.get().eq_ignore_ascii_case("TZID") => {
                        Some(line.raw_value_str())
                    }
                    _ => None,
                })
                .any(|defined| defined.eq_ignore_ascii_case(tzid))
        });

        if defined {
            continue;
        }

        // NOTE: a zone the database knows always rebuilds; one it cannot
        // describe goes back to standing on its TZID alone, which every
        // reader resolves through the database anyway.
        let Some(vtimezone) = tzdb::vtimezone(tzid, *anchor) else {
            continue;
        };

        // NOTE: ahead of the VEVENT rather than at the end of the
        // envelope, so a reader taking the document a line at a time
        // meets a definition before the property leaning on it.
        let at = cst
            .items
            .iter()
            .position(is_vevent)
            .unwrap_or(cst.items.len());

        cst.items
            .insert(at, IcalItem::Component(Box::new(vtimezone)));
    }
}

/// Whether an item is a VEVENT.
fn is_vevent(item: &IcalItem<'_>) -> bool {
    match item {
        IcalItem::Component(child) => child
            .begin
            .as_ref()
            .is_some_and(|begin| begin.raw_value_str().eq_ignore_ascii_case("VEVENT")),
        _ => false,
    }
}

/// Whether an item is a line carrying the given property.
fn names(item: &IcalItem<'_>, name: &str) -> bool {
    match item {
        IcalItem::Prop(line) => line.name.get().eq_ignore_ascii_case(name),
        _ => false,
    }
}

/// An encoded line folded at 75 octets (RFC 5545 3.1).
///
/// ical-rs writes an encoded property out on one line, and a description
/// easily runs past what RFC 5545 asks a line to stay within. The fold is
/// read back through the parser, which records it on the line's wire
/// shape, so it serializes folded.
fn folded(line: IcalLine<'static>) -> Result<IcalItem<'static>> {
    let logical = line.to_string();
    let logical = logical.trim_end_matches(['\r', '\n']);

    let mut out = String::with_capacity(logical.len() + logical.len() / FOLD_OCTETS * 3);
    let mut width = 0;

    for character in logical.chars() {
        // NOTE: a fold never splits a character: the octet budget is
        // checked before a whole one is written.
        if width + character.len_utf8() > FOLD_OCTETS {
            out.push_str("\r\n ");
            width = 1;
        }

        out.push(character);
        width += character.len_utf8();
    }

    let mut scratch = IcalCst::empty("VEVENT");
    scratch
        .push_raw(&out)
        .with_context(|| format!("Encode iCalendar line `{logical}` error"))?;

    scratch.items.pop().context("Encode iCalendar line error")
}

/// A text property, its line breaks normalized to the `\n` RFC 5545
/// escapes.
fn text(kind: IcalPropKind, value: &str) -> IcalProp<'static> {
    let value = value.replace("\r\n", "\n").replace('\r', "\n");
    prop(
        kind,
        Vec::new(),
        IcalValue::Text(IcalText(Cow::Owned(value))),
    )
}

/// A property under a canonical name.
fn prop(
    kind: IcalPropKind,
    params: Vec<IcalParam<'static>>,
    value: IcalValue<'static>,
) -> IcalProp<'static> {
    IcalProp {
        name: IcalPropName::Kind(kind),
        params,
        value,
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use jiff::Timestamp;

    use super::{EventFieldsArgs, EventMethodArg, revise, set_method};

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        fields: EventFieldsArgs,
    }

    fn fields(args: &[&str]) -> EventFieldsArgs {
        Wrap::try_parse_from([&["calendula"], args].concat())
            .unwrap()
            .fields
    }

    fn apply(ical: &str, args: &[&str]) -> String {
        let out = fields(args).apply(ical.as_bytes()).unwrap();
        String::from_utf8(out).unwrap()
    }

    fn update(ical: &str, args: &[&str]) -> String {
        let out = fields(args).apply_keeping_length(ical.as_bytes()).unwrap();
        String::from_utf8(out).unwrap()
    }

    fn refusal(ical: &str, args: &[&str]) -> String {
        fields(args).apply(ical.as_bytes()).unwrap_err().to_string()
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
        "ATTENDEE;CN=Jane:mailto:jane@example.org\r\n",
        "ATTENDEE;CN=John:mailto:john@example.org\r\n",
        "X-ODD;FOO=bar:kept as is\r\n",
        "BEGIN:VALARM\r\n",
        "ACTION:DISPLAY\r\n",
        "TRIGGER:-PT10M\r\n",
        "END:VALARM\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    const PARIS: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//x//y//EN\r\n",
        "BEGIN:VTIMEZONE\r\n",
        "TZID:Europe/Paris\r\n",
        "BEGIN:STANDARD\r\n",
        "DTSTART:19701025T030000\r\n",
        "TZOFFSETFROM:+0200\r\n",
        "TZOFFSETTO:+0100\r\n",
        "END:STANDARD\r\n",
        "END:VTIMEZONE\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART;TZID=Europe/Paris:20261019T090000\r\n",
        "DTEND;TZID=Europe/Paris:20261019T100000\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    #[test]
    fn no_flag_leaves_the_event_byte_for_byte() {
        assert_eq!(apply(EVENT, &[]), EVENT);

        // NOTE: a source the parser would not even read passes through
        // too, since nothing asked for it to be read.
        assert_eq!(apply("not an ical", &[]), "not an ical");
    }

    #[test]
    fn a_flag_replaces_every_instance_of_its_property() {
        let out = apply(EVENT, &["--attendee", "new@example.org"]);

        assert!(!out.contains("jane@example.org"), "{out}");
        assert!(!out.contains("john@example.org"), "{out}");
        assert_eq!(out.matches("ATTENDEE").count(), 1, "{out}");
        assert!(
            out.contains("ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:new@example.org\r\n"),
            "{out}"
        );
    }

    #[test]
    fn a_repeated_flag_writes_one_instance_per_value() {
        let out = apply(
            EVENT,
            &[
                "--attendee",
                "Ann Lee:ann@example.org",
                "--attendee",
                "mailto:bob@example.org",
            ],
        );

        assert!(
            out.contains(
                "ATTENDEE;CN=Ann Lee;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:ann@example.org\r\n"
            ),
            "{out}"
        );
        assert!(
            out.contains("ATTENDEE;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:bob@example.org\r\n"),
            "{out}"
        );

        // NOTE: the two land where the first instance stood, so the body
        // keeps its order.
        let ann = out.find("ann@").unwrap();
        let bob = out.find("bob@").unwrap();
        let summary = out.find("SUMMARY").unwrap();
        assert!(summary < ann && ann < bob, "{out}");
    }

    #[test]
    fn an_untouched_property_keeps_its_own_bytes() {
        let out = apply(EVENT, &["--summary", "Retro"]);

        assert_eq!(out, EVENT.replace("SUMMARY:Stand-up", "SUMMARY:Retro"));
    }

    #[test]
    fn a_property_the_event_lacked_lands_ahead_of_its_alarm() {
        let out = apply(EVENT, &["--location", "Room 1"]);

        let location = out.find("LOCATION:Room 1\r\n").unwrap();
        let alarm = out.find("BEGIN:VALARM").unwrap();
        assert!(location < alarm, "{out}");
        assert!(out.contains("X-ODD;FOO=bar:kept as is\r\n"), "{out}");
    }

    #[test]
    fn every_text_flag_writes_its_property_escaped() {
        let out = apply(
            EVENT,
            &[
                "--uid",
                "2@example.org",
                "--description",
                "one, two; three\nfour",
                "--location",
                "Room 1",
                "--url",
                "https://example.org/a?b=1;c=2",
                "--status",
                "tentative",
                "--transparency",
                "transparent",
                "--organizer",
                "Jane Doe:jane@example.org",
                "--categories",
                "work",
                "--categories",
                "a, b",
                "--sequence",
                "3",
            ],
        );

        assert!(out.contains("UID:2@example.org\r\n"), "{out}");
        assert!(
            out.contains("DESCRIPTION:one\\, two\\; three\\nfour\r\n"),
            "{out}"
        );
        assert!(out.contains("LOCATION:Room 1\r\n"), "{out}");
        assert!(
            out.contains("URL:https://example.org/a?b=1;c=2\r\n"),
            "{out}"
        );
        assert!(out.contains("STATUS:TENTATIVE\r\n"), "{out}");
        assert!(out.contains("TRANSP:TRANSPARENT\r\n"), "{out}");
        assert!(
            out.contains("ORGANIZER;CN=Jane Doe:mailto:jane@example.org\r\n"),
            "{out}"
        );
        assert!(out.contains("CATEGORIES:work,a\\, b\r\n"), "{out}");
        assert!(out.contains("SEQUENCE:3\r\n"), "{out}");
        assert!(!out.contains("UID:1@example.org"), "{out}");
    }

    #[test]
    fn a_name_carrying_a_delimiter_is_quoted() {
        let out = apply(EVENT, &["--organizer", "Doe, Jane:jane@example.org"]);

        assert!(
            out.contains("ORGANIZER;CN=\"Doe, Jane\":mailto:jane@example.org\r\n"),
            "{out}"
        );
    }

    #[test]
    fn an_address_that_is_not_one_is_refused() {
        let err = refusal(EVENT, &["--attendee", "Jane"]);

        assert!(err.contains("email address"), "{err}");
    }

    #[test]
    fn a_long_line_is_folded() {
        let long = "word ".repeat(40);
        let out = apply(EVENT, &["--description", &long]);

        assert!(out.contains("DESCRIPTION:word"), "{out}");
        assert!(out.contains("\r\n "), "{out}");
        assert!(out.lines().all(|line| line.len() <= 75), "{out}");
    }

    #[test]
    fn a_time_takes_a_date_a_local_time_and_a_utc_one() {
        let out = apply(EVENT, &["--start", "2026-10-19", "--end", "2026-10-20"]);
        assert!(out.contains("DTSTART;VALUE=DATE:20261019\r\n"), "{out}");
        assert!(out.contains("DTEND;VALUE=DATE:20261020\r\n"), "{out}");

        let out = apply(EVENT, &["--start", "2026-10-19T09:00"]);
        assert!(out.contains("DTSTART:20261019T090000\r\n"), "{out}");

        let out = apply(EVENT, &["--start", "2026-10-19T09:00:30Z"]);
        assert!(out.contains("DTSTART:20261019T090030Z\r\n"), "{out}");

        // NOTE: no zone named, so none minted.
        assert!(!out.contains("VTIMEZONE"), "{out}");
    }

    #[test]
    fn an_unreadable_time_is_refused_naming_the_shapes() {
        let err = refusal(EVENT, &["--start", "19/10/2026"]);

        assert!(err.contains("YYYY-MM-DD"), "{err}");
    }

    #[test]
    fn a_zone_flag_names_the_zone_and_mints_it_once() {
        let out = apply(
            EVENT,
            &[
                "--time-zone",
                "Europe/Paris",
                "--start",
                "2026-10-19T09:00",
                "--end",
                "2026-10-19T10:00",
            ],
        );

        assert!(
            out.contains("DTSTART;TZID=Europe/Paris:20261019T090000\r\n"),
            "{out}"
        );
        assert!(
            out.contains("DTEND;TZID=Europe/Paris:20261019T100000\r\n"),
            "{out}"
        );
        assert_eq!(out.matches("BEGIN:VTIMEZONE").count(), 1, "{out}");
        assert!(out.contains("TZID:Europe/Paris\r\n"), "{out}");

        // NOTE: the definition comes ahead of the event leaning on it.
        assert!(
            out.find("BEGIN:VTIMEZONE").unwrap() < out.find("BEGIN:VEVENT").unwrap(),
            "{out}"
        );
    }

    #[test]
    fn a_zone_the_calendar_defines_is_not_minted_again() {
        let out = apply(PARIS, &["--start", "Europe/Paris:2026-10-20T09:00"]);

        assert_eq!(out.matches("BEGIN:VTIMEZONE").count(), 1, "{out}");
        assert!(out.contains("TZOFFSETTO:+0100\r\n"), "{out}");
        assert!(
            out.contains("DTSTART;TZID=Europe/Paris:20261020T090000\r\n"),
            "{out}"
        );
    }

    #[test]
    fn an_inline_zone_wins_over_the_zone_flag() {
        let out = apply(
            EVENT,
            &[
                "--time-zone",
                "Europe/Paris",
                "--start",
                "America/New_York:2026-10-19T09:00",
                "--end",
                "2026-10-19T16:00",
            ],
        );

        assert!(
            out.contains("DTSTART;TZID=America/New_York:20261019T090000\r\n"),
            "{out}"
        );
        assert!(
            out.contains("DTEND;TZID=Europe/Paris:20261019T160000\r\n"),
            "{out}"
        );
        assert_eq!(out.matches("BEGIN:VTIMEZONE").count(), 2, "{out}");
    }

    #[test]
    fn a_utc_time_ignores_the_zone_flag() {
        let out = apply(
            EVENT,
            &[
                "--time-zone",
                "Europe/Paris",
                "--start",
                "2026-10-19T07:00Z",
            ],
        );

        assert!(out.contains("DTSTART:20261019T070000Z\r\n"), "{out}");
        assert!(!out.contains("VTIMEZONE"), "{out}");
    }

    #[test]
    fn an_unknown_zone_is_refused_by_name() {
        let err = refusal(
            EVENT,
            &["--time-zone", "Mars/Olympus", "--start", "2026-10-19T09:00"],
        );
        assert!(err.contains("Mars/Olympus"), "{err}");

        let err = refusal(EVENT, &["--start", "Mars/Olympus:2026-10-19T09:00"]);
        assert!(err.contains("Mars/Olympus"), "{err}");
    }

    #[test]
    fn a_zone_flag_with_no_time_is_refused() {
        let err = refusal(EVENT, &["--time-zone", "Europe/Paris"]);

        assert!(err.contains("--start"), "{err}");
    }

    #[test]
    fn a_date_and_a_time_do_not_mix() {
        let err = refusal(
            EVENT,
            &["--start", "2026-10-19", "--end", "2026-10-19T10:00"],
        );
        assert!(err.contains("both"), "{err}");

        let err = refusal(
            EVENT,
            &["--start", "2026-10-19T10:00Z", "--end", "2026-10-19T09:00Z"],
        );
        assert!(err.contains("before"), "{err}");
    }

    #[test]
    fn a_duration_replaces_the_end_and_an_end_the_duration() {
        let out = apply(EVENT, &["--duration", "pt1h30m"]);
        assert!(out.contains("DURATION:PT1H30M\r\n"), "{out}");
        assert!(!out.contains("DTEND"), "{out}");

        let out = apply(&out, &["--end", "2026-10-19T11:00Z"]);
        assert!(out.contains("DTEND:20261019T110000Z\r\n"), "{out}");
        assert!(!out.contains("DURATION"), "{out}");
    }

    #[test]
    fn a_duration_outside_the_grammar_is_refused() {
        for duration in ["1h", "P1H", "PT", "P1DT", "-PT1H", "P1W2D"] {
            let err = refusal(EVENT, &[&format!("--duration={duration}")]);
            assert!(err.contains("RFC 5545 duration"), "{duration}: {err}");
        }

        for duration in ["P1D", "P2W", "PT15M", "P1DT2H", "PT1H0M5S"] {
            apply(EVENT, &["--duration", duration]);
        }
    }

    #[test]
    fn an_end_and_a_duration_cannot_both_be_given() {
        let parsed = Wrap::try_parse_from([
            "calendula",
            "--end",
            "2026-10-19T10:00Z",
            "--duration",
            "PT1H",
        ]);

        assert!(parsed.is_err());
    }

    #[test]
    fn an_update_moving_the_start_keeps_the_length() {
        let out = update(EVENT, &["--start", "2026-10-20T14:00Z"]);

        assert!(out.contains("DTSTART:20261020T140000Z\r\n"), "{out}");
        assert!(out.contains("DTEND:20261020T150000Z\r\n"), "{out}");
        assert_eq!(out.matches("DTEND").count(), 1, "{out}");
    }

    #[test]
    fn an_update_moving_a_zoned_start_spells_the_end_in_its_zone() {
        let out = update(PARIS, &["--start", "Europe/Paris:2026-10-26T09:00"]);

        assert!(
            out.contains("DTEND;TZID=Europe/Paris:20261026T100000\r\n"),
            "{out}"
        );
    }

    #[test]
    fn an_update_moving_a_whole_day_keeps_its_days() {
        let event = EVENT
            .replace("DTSTART:20261019T090000Z", "DTSTART;VALUE=DATE:20261019")
            .replace("DTEND:20261019T100000Z", "DTEND;VALUE=DATE:20261021");
        let out = update(&event, &["--start", "2026-11-02"]);

        assert!(out.contains("DTEND;VALUE=DATE:20261104\r\n"), "{out}");
    }

    #[test]
    fn an_update_keeps_a_duration_and_lets_an_end_win() {
        let event = EVENT.replace("DTEND:20261019T100000Z", "DURATION:PT2H");
        let out = update(&event, &["--start", "2026-10-20T14:00Z"]);
        assert!(out.contains("DURATION:PT2H\r\n"), "{out}");
        assert!(!out.contains("DTEND"), "{out}");

        let out = update(
            EVENT,
            &["--start", "2026-10-20T14:00Z", "--end", "2026-10-20T18:00Z"],
        );
        assert!(out.contains("DTEND:20261020T180000Z\r\n"), "{out}");

        let out = update(
            EVENT,
            &["--start", "2026-10-20T14:00Z", "--duration", "PT30M"],
        );
        assert!(out.contains("DURATION:PT30M\r\n"), "{out}");
        assert!(!out.contains("DTEND"), "{out}");
    }

    #[test]
    fn a_build_moving_the_start_leaves_the_end() {
        let out = apply(EVENT, &["--start", "2026-10-19T08:00Z"]);

        assert!(out.contains("DTEND:20261019T100000Z\r\n"), "{out}");
    }

    #[test]
    fn a_source_holding_several_vevents_is_refused() {
        let two = EVENT.replace(
            "END:VCALENDAR",
            "BEGIN:VEVENT\r\nUID:2@example.org\r\nDTSTAMP:20260101T000000Z\r\nEND:VEVENT\r\nEND:VCALENDAR",
        );

        let err = refusal(&two, &["--summary", "x"]);
        assert!(err.contains("several VEVENTs"), "{err}");

        // NOTE: with no flag it passes through as written.
        assert_eq!(apply(&two, &[]), two);

        let calendars = format!("{EVENT}{EVENT}");
        let err = refusal(&calendars, &["--summary", "x"]);
        assert!(err.contains("several VEVENTs"), "{err}");
    }

    #[test]
    fn a_source_holding_no_vevent_is_refused() {
        let todo = EVENT.replace("VEVENT", "VTODO");
        let err = refusal(&todo, &["--summary", "x"]);

        assert!(err.contains("no VEVENT"), "{err}");
    }

    #[test]
    fn the_method_is_set_on_the_calendar() {
        let out = set_method(EVENT.as_bytes(), EventMethodArg::Request).unwrap();
        let out = String::from_utf8(out).unwrap();

        assert!(
            out.contains("PRODID:-//x//y//EN\r\nMETHOD:REQUEST\r\nBEGIN:VEVENT"),
            "{out}"
        );

        let out = set_method(out.as_bytes(), EventMethodArg::DeclineCounter).unwrap();
        let out = String::from_utf8(out).unwrap();

        assert_eq!(out.matches("METHOD").count(), 1, "{out}");
        assert!(out.contains("METHOD:DECLINECOUNTER\r\n"), "{out}");
    }

    /// An event carrying every property an empty flag removes.
    const FULL: &str = concat!(
        "BEGIN:VCALENDAR\r\n",
        "VERSION:2.0\r\n",
        "PRODID:-//x//y//EN\r\n",
        "BEGIN:VEVENT\r\n",
        "UID:1@example.org\r\n",
        "DTSTAMP:20260101T000000Z\r\n",
        "DTSTART:20261019T090000Z\r\n",
        "DTEND:20261019T100000Z\r\n",
        "SUMMARY:Stand-up\r\n",
        "DESCRIPTION:Daily\r\n",
        "LOCATION:Room 1\r\n",
        "URL:https://example.org/standup\r\n",
        "CATEGORIES:work,team\r\n",
        "ORGANIZER;CN=Jane:mailto:jane@example.org\r\n",
        "ATTENDEE;CN=Jane:mailto:jane@example.org\r\n",
        "ATTENDEE;CN=John:mailto:john@example.org\r\n",
        "X-ODD;FOO=bar:kept as is\r\n",
        "END:VEVENT\r\n",
        "END:VCALENDAR\r\n",
    );

    #[test]
    fn an_empty_flag_removes_its_property_and_nothing_else() {
        let cases: [(&str, &[&str]); 7] = [
            ("--summary", &["SUMMARY:Stand-up\r\n"]),
            ("--description", &["DESCRIPTION:Daily\r\n"]),
            ("--location", &["LOCATION:Room 1\r\n"]),
            ("--url", &["URL:https://example.org/standup\r\n"]),
            ("--categories", &["CATEGORIES:work,team\r\n"]),
            (
                "--organizer",
                &["ORGANIZER;CN=Jane:mailto:jane@example.org\r\n"],
            ),
            (
                "--attendee",
                &[
                    "ATTENDEE;CN=Jane:mailto:jane@example.org\r\n",
                    "ATTENDEE;CN=John:mailto:john@example.org\r\n",
                ],
            ),
        ];

        for (flag, lines) in cases {
            let out = apply(FULL, &[flag, ""]);
            let expected = lines
                .iter()
                .fold(FULL.to_owned(), |event, line| event.replace(line, ""));

            assert_eq!(out, expected, "{flag}");
        }
    }

    #[test]
    fn an_empty_flag_on_an_absent_property_changes_nothing() {
        for flag in [
            "--description",
            "--location",
            "--url",
            "--categories",
            "--organizer",
        ] {
            assert_eq!(apply(EVENT, &[flag, ""]), EVENT, "{flag}");
        }

        let bare = EVENT
            .replace("ATTENDEE;CN=Jane:mailto:jane@example.org\r\n", "")
            .replace("ATTENDEE;CN=John:mailto:john@example.org\r\n", "");
        assert_eq!(apply(&bare, &["--attendee", ""]), bare);
    }

    #[test]
    fn several_empty_flags_remove_several_properties() {
        let out = apply(FULL, &["--location", "", "--url", "", "--summary", "Retro"]);

        assert!(!out.contains("LOCATION"), "{out}");
        assert!(!out.contains("URL"), "{out}");
        assert!(out.contains("SUMMARY:Retro\r\n"), "{out}");
        assert!(out.contains("X-ODD;FOO=bar:kept as is\r\n"), "{out}");
    }

    #[test]
    fn an_empty_value_among_others_is_skipped() {
        let out = apply(
            FULL,
            &[
                "--categories",
                "",
                "--categories",
                "home",
                "--attendee",
                "",
                "--attendee",
                "bob@example.org",
            ],
        );

        assert!(out.contains("CATEGORIES:home\r\n"), "{out}");
        assert_eq!(out.matches("ATTENDEE").count(), 1, "{out}");
        assert!(out.contains("mailto:bob@example.org"), "{out}");
    }

    #[test]
    fn an_empty_time_status_or_number_is_still_refused() {
        for flag in ["--start", "--end", "--duration"] {
            let err = refusal(EVENT, &[flag, ""]);
            assert!(err.contains("Cannot read"), "{flag}: {err}");
        }

        for flag in ["--status", "--transparency", "--sequence"] {
            let parsed = Wrap::try_parse_from(["calendula", flag, ""]);
            assert!(parsed.is_err(), "{flag}");
        }
    }

    #[test]
    fn an_online_meeting_is_asked_for_on_the_event() {
        let out = apply(EVENT, &["--online-meeting"]);

        assert_eq!(
            out,
            EVENT.replace(
                "X-ODD;FOO=bar:kept as is\r\n",
                "X-ODD;FOO=bar:kept as is\r\nX-PIMDIR-ONLINE-MEETING:TRUE\r\n",
            )
        );

        // NOTE: an ask already there, whatever it said, is replaced in
        // place rather than doubled.
        let asked = EVENT.replace("SUMMARY", "x-pimdir-online-meeting:FALSE\r\nSUMMARY");
        let out = apply(&asked, &["--online-meeting"]);

        assert_eq!(out.matches("ONLINE-MEETING").count(), 1, "{out}");
        assert_eq!(
            out,
            EVENT.replace("SUMMARY", "X-PIMDIR-ONLINE-MEETING:TRUE\r\nSUMMARY")
        );
    }

    /// The instant the revision tests are written at: 2026-10-06 12:34:56
    /// UTC.
    fn at() -> Timestamp {
        Timestamp::from_second(1_791_290_096).unwrap()
    }

    fn revised(ical: &str) -> String {
        String::from_utf8(revise(ical.as_bytes(), at()).unwrap()).unwrap()
    }

    #[test]
    fn a_revision_stamps_the_event_and_nothing_else() {
        assert_eq!(at().to_string(), "2026-10-06T12:34:56Z");

        let out = revised(EVENT);

        assert_eq!(
            out,
            EVENT.replace("DTSTAMP:20260101T000000Z", "DTSTAMP:20261006T123456Z")
        );
        assert!(!out.contains("LAST-MODIFIED"), "{out}");
    }

    #[test]
    fn a_revision_moves_a_last_modified_the_event_carries() {
        let event = EVENT.replace(
            "SUMMARY:Stand-up\r\n",
            "SUMMARY:Stand-up\r\nLAST-MODIFIED:20260102T000000Z\r\n",
        );

        let out = revised(&event);

        assert_eq!(
            out,
            event
                .replace("DTSTAMP:20260101T000000Z", "DTSTAMP:20261006T123456Z")
                .replace(
                    "LAST-MODIFIED:20260102T000000Z",
                    "LAST-MODIFIED:20261006T123456Z"
                )
        );
    }

    #[test]
    fn a_revision_stamps_every_vevent_and_one_lacking_a_dtstamp() {
        let series = EVENT.replace(
            "END:VCALENDAR",
            "BEGIN:VEVENT\r\nUID:1@example.org\r\nRECURRENCE-ID:20261026T090000Z\r\n\
             DTSTART:20261026T100000Z\r\nEND:VEVENT\r\nEND:VCALENDAR",
        );

        let out = revised(&series);

        assert_eq!(
            out.matches("DTSTAMP:20261006T123456Z\r\n").count(),
            2,
            "{out}"
        );
        assert!(
            out.contains("DTSTART:20261026T100000Z\r\nDTSTAMP:20261006T123456Z\r\nEND:VEVENT"),
            "{out}"
        );
    }

    #[test]
    fn a_revision_leaves_what_holds_no_vevent_as_it_is() {
        let todo = EVENT.replace("VEVENT", "VTODO");

        assert_eq!(revised(&todo), todo);
        assert_eq!(revised("not an ical"), "not an ical");
    }
}
