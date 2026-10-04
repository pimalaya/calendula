//! # iCalendar source
//!
//! Where the shared commands get raw iCalendar bytes: a file, the
//! standard input, the argument itself, or nothing at all.
//!
//! It also holds the two halves of the strict reading calendula keeps for
//! the bytes it wrote itself: [`check`], which weighs an item against the
//! RFC 5545 contract, and [`blank_item`], which mints the identity a
//! composer cannot invent.

use std::{
    fs,
    io::{Read, stdin},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use ical::{
    component::{IcalComponent, IcalComponentKind, IcalComponentName},
    ical::Ical,
    prop::{IcalProp, IcalPropKind, IcalPropName},
    tree::cst::IcalCst,
    value::{IcalValue, datetime::IcalDateTime, text::IcalText},
    version::IcalVersion,
};

use crate::shared::uuid::uuid_v4;

/// `PRODID` a minted item is stamped with (RFC 5545 3.7.3).
const PRODID: &str = "-//Pimalaya//calendula//EN";

/// One of the four shared command families, and the component kind it
/// names.
///
/// The families are views over the same items, so a pipeline they share
/// takes this rather than four copies of itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IcalFamily {
    /// The `event` family, over VEVENT.
    Event,
    /// The `todo` family, over VTODO.
    Todo,
    /// The `journal` family, over VJOURNAL.
    Journal,
    /// The `item` family, over any kind, which is why it names none.
    Item,
}

impl IcalFamily {
    /// The component kind the family mints, `None` for `item`.
    pub fn kind(&self) -> Option<IcalComponentKind> {
        match self {
            Self::Event => Some(IcalComponentKind::VEvent),
            Self::Todo => Some(IcalComponentKind::VTodo),
            Self::Journal => Some(IcalComponentKind::VJournal),
            Self::Item => None,
        }
    }

    /// What the family calls what it writes, for a message about one.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Event => "Event",
            Self::Todo => "Todo",
            Self::Journal => "Journal",
            Self::Item => "Item",
        }
    }
}

/// Resolves an iCalendar source into raw bytes.
///
/// `-` reads stdin and an existing path is read from disk, otherwise the
/// value is taken as literal iCalendar contents.
///
/// A source carrying nothing but whitespace is refused rather than passed
/// on: handing a backend an empty body is never what was meant, and an
/// abandoned interactive build prints nothing, which pipes straight into
/// here.
pub fn read_source(source: &str) -> Result<Vec<u8>> {
    if source == "-" {
        let mut buf = Vec::new();
        stdin()
            .read_to_end(&mut buf)
            .context("Read iCalendar from stdin error")?;

        return refuse_blank(buf, "stdin");
    }

    let path = PathBuf::from(source);

    if path.is_file() {
        let contents = fs::read(&path)
            .with_context(|| format!("Read iCalendar from `{}` error", path.display()))?;

        return refuse_blank(contents, &format!("`{}`", path.display()));
    }

    let trimmed = source.trim_start();

    if trimmed.starts_with("BEGIN:VCALENDAR") || trimmed.starts_with("BEGIN:V") {
        return Ok(source.as_bytes().to_vec());
    }

    bail!("Source `{source}` is neither a readable file nor iCalendar contents")
}

/// What is wrong with an item, empty when nothing is.
///
/// Checks it against the RFC 5545 contract through ical-rs, so a
/// VCALENDAR missing its PRODID or a VEVENT missing its DTSTAMP is caught
/// here rather than by the server, or worse by neither. Reading is
/// liberal and this is the strict half: bytes that do not parse at all
/// come back as the one violation they are.
///
/// [`missing_dtstart`] is checked alongside it, being the one requirement
/// a static property list cannot state.
pub fn check(item: &[u8]) -> Vec<String> {
    let cst = match IcalCst::parse(item) {
        Ok(cst) => cst,
        Err(err) => return vec![format!("{err}")],
    };

    let ical = cst.decode();
    let conditional = missing_dtstart(&ical);

    let mut violations = match ical.validate() {
        Ok(_) => Vec::new(),
        Err(errors) => errors.iter().map(|err| format!("{err}")).collect(),
    };

    violations.extend(conditional);
    violations
}

/// Every VEVENT that owes a DTSTART and carries none.
///
/// RFC 5545 3.6.1 makes DTSTART required of a VEVENT unless the calendar
/// specifies a METHOD, which is a condition on the calendar rather than a
/// property list, so ical-rs's per-component contract cannot state it and
/// does not.
///
/// It is worth stating here because it is the one thing a person writing
/// an event by hand leaves out, and because a server does worse than
/// refuse it: SabreDAV denormalizes DTSTART into its index on write and
/// answers HTTP 500 when there is none.
fn missing_dtstart(ical: &Ical<'_>) -> Vec<String> {
    if ical
        .props
        .iter()
        .any(|prop| prop.name.eq_ignore_ascii_case("METHOD"))
    {
        return Vec::new();
    }

    ical.components
        .iter()
        .filter(|component| component.name.eq_ignore_ascii_case("VEVENT"))
        .filter(|component| {
            !component
                .props
                .iter()
                .any(|prop| prop.name.eq_ignore_ascii_case("DTSTART"))
        })
        .map(|_| {
            String::from(
                "Component `VEVENT` is missing required property `DTSTART` (RFC 5545 3.6.1, the calendar specifying no `METHOD`)",
            )
        })
        .collect()
}

/// Whether an item carries a component of `kind`.
///
/// Only the two backends that narrow a listing after parsing it read
/// this; CalDAV pushes the kind down and gcal models one kind only.
///
/// The projections are keyed on a marker type, which a listing narrowing
/// by a kind it only knows at runtime cannot use, so this reads the
/// decoded component names instead. Bytes that do not parse carry
/// nothing, which is what a projection makes of them too.
#[cfg(any(feature = "vdir", feature = "pimdir"))]
pub fn holds_kind(contents: &[u8], kind: IcalComponentKind) -> bool {
    let Ok(cst) = IcalCst::parse(contents) else {
        return false;
    };

    cst.decode()
        .components
        .iter()
        .any(|component| component.name.eq_ignore_ascii_case(&kind))
}

/// Mints an item carrying nothing but its identity and what it owes.
///
/// This is what a create starts from when it is given no source, and it
/// is an item rather than an empty file so that no composer is asked to
/// invent a `UID`: a backend may key the item on it, and an editor
/// handed an empty file mints none.
///
/// A VEVENT also carries a DTSTART of now, which [`missing_dtstart`]
/// would otherwise refuse the moment the composer handed it back. The
/// value is a placeholder like the identity is, and an event with no time
/// is not an event; a VTODO and a VJOURNAL owe none and get none.
///
/// `item` names no component kind, so it mints nothing and says which
/// families do.
pub fn blank_item(family: IcalFamily) -> Result<Vec<u8>> {
    let Some(kind) = family.kind() else {
        bail!(
            "`item` names no component kind to mint; give an iCalendar, \
             or compose one with `event`, `todo` or `journal`"
        )
    };

    let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    let mut props = vec![
        text_prop(IcalPropKind::Uid, uuid_v4()?),
        stamp_prop(IcalPropKind::DtStamp, &now),
    ];

    if kind == IcalComponentKind::VEvent {
        props.push(stamp_prop(IcalPropKind::DtStart, &now));
    }

    let component = IcalComponent {
        name: IcalComponentName::Kind(kind),
        props,
        components: Vec::new(),
    };

    let ical = Ical {
        version: IcalVersion::V2_0,
        props: vec![text_prop(IcalPropKind::ProdId, PRODID)],
        components: vec![component],
    };

    Ok(IcalCst::from(ical).to_bytes())
}

/// Refuses a source that carries no item at all.
fn refuse_blank(contents: Vec<u8>, source: &str) -> Result<Vec<u8>> {
    if contents.iter().all(u8::is_ascii_whitespace) {
        bail!("No iCalendar to read from {source}");
    }

    Ok(contents)
}

/// A text property under a canonical name, carrying no parameter.
fn text_prop(kind: IcalPropKind, value: impl ToString) -> IcalProp<'static> {
    prop(kind, IcalValue::Text(IcalText(value.to_string().into())))
}

/// A date-time property under a canonical name, carrying no parameter.
fn stamp_prop(kind: IcalPropKind, value: impl ToString) -> IcalProp<'static> {
    prop(
        kind,
        IcalValue::DateTime(IcalDateTime(value.to_string().into())),
    )
}

/// A property under a canonical name, carrying no parameter.
fn prop(kind: IcalPropKind, value: IcalValue<'static>) -> IcalProp<'static> {
    IcalProp {
        name: IcalPropName::Kind(kind),
        params: Vec::new(),
        value,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "vdir", feature = "pimdir"))]
    use ical::component::IcalComponentKind;

    #[cfg(any(feature = "vdir", feature = "pimdir"))]
    use super::holds_kind;
    use super::{IcalFamily, blank_item, check, read_source};

    #[test]
    fn a_minted_item_passes_the_check_it_will_be_held_to() {
        for family in [IcalFamily::Event, IcalFamily::Todo, IcalFamily::Journal] {
            let item = blank_item(family).unwrap();
            let text = String::from_utf8(item.clone()).unwrap();

            assert!(text.contains("PRODID:"), "{text}");
            assert!(text.contains("UID:"), "{text}");
            assert!(text.contains("DTSTAMP:"), "{text}");
            assert!(check(&item).is_empty(), "{:?}", check(&item));

            // NOTE: RFC 5545 3.6.1 owes a DTSTART on a VEVENT alone.
            assert_eq!(
                text.contains("DTSTART:"),
                family == IcalFamily::Event,
                "{text}"
            );
        }
    }

    #[test]
    fn the_item_family_mints_nothing_and_names_the_ones_that_do() {
        let err = blank_item(IcalFamily::Item).unwrap_err().to_string();

        assert!(err.contains("event"), "{err}");
        assert!(err.contains("todo"), "{err}");
        assert!(err.contains("journal"), "{err}");
    }

    #[cfg(any(feature = "vdir", feature = "pimdir"))]
    #[test]
    fn an_item_answers_the_kind_it_carries_and_no_other() {
        let item = |kind: &str| {
            format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n\
                 BEGIN:{kind}\r\nUID:a\r\nDTSTAMP:20260101T000000Z\r\n\
                 DTSTART:20260101T000000Z\r\nEND:{kind}\r\nEND:VCALENDAR\r\n"
            )
        };

        assert!(holds_kind(
            item("VTODO").as_bytes(),
            IcalComponentKind::VTodo
        ));
        assert!(!holds_kind(
            item("VTODO").as_bytes(),
            IcalComponentKind::VEvent
        ));

        // NOTE: bytes that carry nothing readable carry no kind either,
        // which is what a projection makes of them too.
        assert!(!holds_kind(b"not an ical", IcalComponentKind::VEvent));
    }

    #[test]
    fn a_source_that_is_neither_a_file_nor_an_item_is_refused() {
        let err = read_source("not an ical").unwrap_err().to_string();

        assert!(err.contains("neither a readable file"), "{err}");
    }

    #[test]
    fn bytes_that_do_not_parse_come_back_as_one_violation() {
        assert_eq!(check(b"not an ical at all").len(), 1);
    }

    #[test]
    fn an_event_owes_a_dtstart_unless_the_calendar_carries_a_method() {
        let event = |extra: &str| {
            format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n{extra}\
                 BEGIN:VEVENT\r\nUID:a\r\nDTSTAMP:20260101T000000Z\r\n\
                 END:VEVENT\r\nEND:VCALENDAR\r\n"
            )
        };

        let violations = check(event("").as_bytes());
        assert_eq!(violations.len(), 1, "{violations:?}");
        assert!(violations[0].contains("DTSTART"), "{violations:?}");

        // NOTE: a calendar carrying a METHOD is a scheduling message, where
        // RFC 5545 3.6.1 makes DTSTART optional.
        assert!(check(event("METHOD:REQUEST\r\n").as_bytes()).is_empty());
    }

    #[test]
    fn a_todo_and_a_journal_owe_no_dtstart() {
        for kind in ["VTODO", "VJOURNAL"] {
            let item = format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//x//y//EN\r\n\
                 BEGIN:{kind}\r\nUID:a\r\nDTSTAMP:20260101T000000Z\r\n\
                 END:{kind}\r\nEND:VCALENDAR\r\n"
            );

            assert!(check(item.as_bytes()).is_empty(), "{kind}");
        }
    }

    #[test]
    fn a_calendar_missing_its_prodid_is_caught_before_the_server_sees_it() {
        let item = b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:a\r\n\
                     DTSTAMP:20260101T000000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        assert!(!check(item).is_empty());
    }
}
