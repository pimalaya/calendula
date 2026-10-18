//! # Event agenda
//!
//! The `calendula event agenda` command, drawing a cal(1)-style grid of
//! the selected calendar and listing the events its days hold.
//!
//! The grid is a port of util-linux cal, so it keeps cal's Julian
//! calendar for the dates before the reform year `--reform` sets. That
//! is why the day arithmetic here is hand-rolled: chrono only knows the
//! proleptic Gregorian calendar.
//!
//! Month and weekday names follow the time locale, as they do in cal:
//! `LC_ALL`, then `LC_TIME`, then `LANG`, falling back to the English
//! names of the POSIX locale.

use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt::{self, Write},
};

use anyhow::{Result, bail};
use chrono::{Datelike, Local, Locale, Months, NaiveDate, NaiveDateTime};
use clap::Parser;
use pimalaya_cli::printer::Printer;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::shared::{
    arg::CalendarIdArg,
    client::CalendarClient,
    event::Event,
    ical::IcalFamily,
    item::{CalendarItemQuery, CalendarTimeRange},
};

const DAYS_IN_WEEK: usize = 7;
const MAXDAYS: usize = 42;
const MONTHS_IN_YEAR: usize = 12;
const SPACE: i32 = -1;
const DAY_LEN: usize = 3;
const WNUM_LEN: usize = 3;
const MONTHS_IN_YEAR_ROW: usize = 3;
const REFORMATION_MONTH: usize = 9;
const NUMBER_MISSING_DAYS: i32 = 11;
const YDAY_AFTER_MISSING: i32 = 258;
const DEFAULT_REFORM_YEAR: i32 = 1752;

/// Display a calendar grid and the events it holds.
///
/// The grid follows cal(1): the current month by default, with the days
/// carrying an event highlighted. Every collected event is then listed
/// by start time under the grid.
///
/// JSON output: an object mapping each start datetime to the labels of
/// every event starting at it.
#[derive(Debug, Parser)]
pub struct EventAgendaCommand {
    #[command(flatten)]
    calendar: CalendarIdArg,

    /// Date the calendar is drawn at. Defaults to today.
    ///
    /// Accepts `YEAR`, `MONTH YEAR` or `DAY MONTH YEAR`, the month
    /// being a number or a name, in English or in the language of the
    /// time locale. A lone year shows the whole year.
    #[arg(name = "DATE")]
    date_args: Vec<String>,

    /// Display single month output.
    #[arg(short = '1', long)]
    one: bool,

    /// Display three months spanning the date.
    #[arg(short = '3', long)]
    three: bool,

    /// Display that many months, starting from the month containing
    /// the date.
    #[arg(short = 'n', long)]
    months: Option<u32>,

    /// Display months spanning the date.
    #[arg(short = 'S', long)]
    span: bool,

    /// Display Sunday as the first day of the week.
    #[arg(short = 's', long)]
    sunday: bool,

    /// Display Monday as the first day of the week.
    #[arg(short = 'm', long)]
    monday: bool,

    /// Use day-of-year (ordinal) numbering, from 1 to 366.
    ///
    /// This does not switch between the Gregorian and the Julian
    /// calendar system: `--reform` is what controls that.
    #[arg(short = 'j', long)]
    julian: bool,

    /// Adoption date of the Gregorian calendar reform.
    ///
    /// Dates before it use the Julian calendar system, dates after it
    /// the Gregorian one. Accepts `1752` (the default), `gregorian`,
    /// `iso` or `julian`.
    #[arg(long)]
    reform: Option<String>,

    /// Display the proleptic Gregorian calendar exclusively.
    ///
    /// Week numbers and the first day of the week are left untouched;
    /// see `--reform`.
    #[arg(long)]
    iso: bool,

    /// Display a calendar for the whole year.
    #[arg(short = 'y', long)]
    year: bool,

    /// Display a calendar for the next twelve months.
    #[arg(short = 'Y', long)]
    twelve: bool,

    /// Display week numbers in the calendar.
    ///
    /// The numbering follows ISO-8601 when the week starts on Monday,
    /// and the US format otherwise.
    #[arg(short = 'w', long)]
    week: bool,

    /// Display using a vertical layout (aka ncal(1) mode).
    #[arg(short = 'v', long)]
    vertical: bool,
}

impl EventAgendaCommand {
    pub fn execute(self, printer: &mut impl Printer, mut client: CalendarClient) -> Result<()> {
        let now = Local::now();

        let mut ctl = CalControl {
            reform_year: DEFAULT_REFORM_YEAR,
            num_months: 0,
            span_months: false,
            months_in_row: 0,
            weekstart: 0,
            weektype: 0,
            day_width: DAY_LEN,
            week_width: 0,
            gutter_width: 2,
            julian: false,
            header_year: false,
            header_hint: false,
            vertical: false,
            req: CalRequest {
                day: 0,
                month: 0,
                year: 0,
                start_month: 0,
            },
            names: Names::new(time_locale()),
            all_events: Vec::new(),
            events: BTreeMap::new(),
        };

        if self.iso
            || self.reform.as_deref() == Some("iso")
            || self.reform.as_deref() == Some("gregorian")
        {
            ctl.reform_year = i32::MIN;
        } else if self.reform.as_deref() == Some("1752") {
            ctl.reform_year = 1752;
        } else if self.reform.as_deref() == Some("julian") {
            ctl.reform_year = i32::MAX;
        }

        if self.monday {
            ctl.weekstart = 1;
        }
        if self.sunday {
            ctl.weekstart = 0;
        }

        if self.julian {
            ctl.day_width = DAY_LEN + 1;
        }
        if self.one {
            ctl.num_months = 1;
        }
        if self.three {
            ctl.num_months = 3;
            ctl.span_months = true;
        }
        if let Some(n) = self.months {
            ctl.num_months = n as usize;
        }
        if self.span {
            ctl.span_months = true;
        }

        ctl.julian = self.julian;
        ctl.vertical = self.vertical;

        if self.week {
            ctl.weektype = if ctl.weekstart == 1 { 0x100 } else { 0x200 };
            ctl.week_width = ctl.day_width * DAYS_IN_WEEK + WNUM_LEN - 1;
        } else {
            ctl.week_width = ctl.day_width * DAYS_IN_WEEK - 1;
        }

        let mut yflag = self.year;
        let yflag_cap = self.twelve;

        match self.date_args.len() {
            3 => {
                ctl.req.day = self.date_args[0].parse().unwrap_or(1);
                ctl.req.month = parse_month(&self.date_args[1], &ctl.names);
                ctl.req.year = self.date_args[2].parse().unwrap_or(now.year());
                let dm = DAYS_IN_MONTH[leap_year(&ctl, ctl.req.year)][ctl.req.month];
                if ctl.req.day > dm as i32 {
                    bail!("Illegal day value: use 1-{dm}");
                }
                ctl.req.day = day_in_year(&ctl, ctl.req.day, ctl.req.month, ctl.req.year);
            }
            2 => {
                ctl.req.month = parse_month(&self.date_args[0], &ctl.names);
                ctl.req.year = self.date_args[1].parse().unwrap_or(now.year());
            }
            1 => {
                ctl.req.year = self.date_args[0].parse().unwrap_or(now.year());
                if ctl.req.year < 1 {
                    bail!("Illegal year value: use positive integer");
                }
                if ctl.req.year == now.year() {
                    ctl.req.day = now.ordinal() as i32;
                }
                ctl.req.month = now.month() as usize;
                if ctl.num_months == 0 {
                    yflag = true;
                }
            }
            _ => {
                ctl.req.day = now.ordinal() as i32;
                ctl.req.month = now.month() as usize;
                ctl.req.year = now.year();
            }
        }

        if yflag || yflag_cap {
            ctl.gutter_width = 3;
            if ctl.num_months == 0 {
                ctl.num_months = MONTHS_IN_YEAR;
            }
            if yflag {
                ctl.req.start_month = 1;
                ctl.header_year = true;
            }
        }

        if ctl.vertical {
            ctl.gutter_width = 1;
        }

        if ctl.num_months > 1 && ctl.months_in_row == 0 {
            ctl.months_in_row = MONTHS_IN_YEAR_ROW;
        } else if ctl.months_in_row == 0 {
            ctl.months_in_row = 1;
        }

        if ctl.num_months == 0 {
            ctl.num_months = 1;
        }

        headers_init(&mut ctl);

        // NOTE: the events are fetched once the grid is settled, over the
        // days it draws, so a recurring series lists each occurrence those
        // days hold instead of once at its first start.
        let calendar_id = client.account.calendar_id(self.calendar.id)?;
        let range = match drawn_days(&ctl) {
            Some((first, last)) => CalendarTimeRange::from_days(Some(first), Some(last))?,
            None => None,
        };
        let items = client.list_items(
            &calendar_id,
            CalendarItemQuery {
                range: range.as_ref(),
                kind: IcalFamily::Event.kind(),
                ..Default::default()
            },
        )?;
        ctl.all_events = items
            .iter()
            .flat_map(|item| Event::occurrences(item, range.as_ref()))
            .collect();

        let mut grid = String::new();

        if yflag || yflag_cap {
            yearly(&mut grid, &mut ctl)?;
        } else {
            monthly(&mut grid, &mut ctl)?;
        }

        printer.out(EventAgendaOutput::new(grid, ctl.events).with_locale(ctl.names.locale))
    }
}

/// The rendering options a run resolved, plus the events it collects.
#[derive(Clone)]
struct CalControl {
    reform_year: i32,
    num_months: usize,
    span_months: bool,
    months_in_row: usize,
    weekstart: usize,
    weektype: usize,
    day_width: usize,
    week_width: usize,
    gutter_width: usize,
    julian: bool,
    header_year: bool,
    header_hint: bool,
    vertical: bool,
    req: CalRequest,
    names: Names,
    all_events: Vec<Event>,
    events: BTreeMap<NaiveDateTime, Vec<AgendaEvent>>,
}

/// The date the calendar is drawn at, as the arguments resolved it.
#[derive(Clone)]
struct CalRequest {
    day: i32,
    month: usize,
    year: i32,
    start_month: usize,
}

/// One month of the grid: its six week rows and their week numbers.
#[derive(Clone)]
struct CalMonth {
    days: [i32; MAXDAYS],
    weeks: [i32; MAXDAYS / DAYS_IN_WEEK],
    month: usize,
    year: i32,
}

const DAYS_IN_MONTH: [[usize; 13]; 2] = [
    [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

/// The locale month and weekday names are drawn in.
///
/// Reads `LC_ALL`, then `LC_TIME`, then `LANG`, the first one set and
/// not empty winning, as setlocale(3) does. Its codeset and modifier
/// (`es_ES.UTF-8@euro`) are dropped; `C`, `POSIX` or a locale chrono
/// does not know fall back to the POSIX one, whose names are English.
fn time_locale() -> Locale {
    let var = ["LC_ALL", "LC_TIME", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|value| !value.is_empty());

    var.map_or(Locale::POSIX, |value| parse_locale(&value))
}

fn parse_locale(value: &str) -> Locale {
    let name = value.split(['.', '@']).next().unwrap_or_default();
    Locale::try_from(name).unwrap_or(Locale::POSIX)
}

/// The month and weekday names of one locale, as the grid prints them.
#[derive(Clone)]
struct Names {
    locale: Locale,
    /// Full month names, January first.
    months: [String; 12],
    /// Weekday names cut to two characters, as cal does, Sunday first.
    weekdays: [String; 7],
}

impl Names {
    fn new(locale: Locale) -> Self {
        // Any year works: only the names are read. 2023-01-01 is a
        // Sunday, so the weekdays come out Sunday first.
        let months = std::array::from_fn(|i| {
            let date = NaiveDate::from_ymd_opt(2023, i as u32 + 1, 1).unwrap();
            date.format_localized("%B", locale).to_string()
        });
        let weekdays = std::array::from_fn(|i| {
            let date = NaiveDate::from_ymd_opt(2023, 1, i as u32 + 1).unwrap();
            let abbr = date.format_localized("%a", locale).to_string();
            abbr.chars().take(2).collect()
        });

        Self {
            locale,
            months,
            weekdays,
        }
    }

    fn month(&self, month: usize) -> &str {
        &self.months[month - 1]
    }
}

/// The width a name takes on screen: its characters, not its bytes, so
/// `sá` or `août` stay aligned.
fn text_width(s: &str) -> usize {
    s.chars().count()
}

fn parse_month(s: &str, names: &Names) -> usize {
    if let Ok(m) = s.parse::<usize>()
        && (1..=12).contains(&m)
    {
        return m;
    }
    let lower = s.to_lowercase();
    let english = Names::new(Locale::POSIX);
    for list in [&names.months, &english.months] {
        for (i, name) in list.iter().enumerate() {
            if name.to_lowercase().starts_with(&lower) {
                return i + 1;
            }
        }
    }
    1
}

fn leap_year(ctl: &CalControl, year: i32) -> usize {
    if year <= ctl.reform_year {
        if year % 4 == 0 { 1 } else { 0 }
    } else if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
        1
    } else {
        0
    }
}

fn headers_init(ctl: &mut CalControl) {
    let year_str = format!("{}", ctl.req.year);
    for month_name in &ctl.names.months {
        if ctl.week_width < text_width(month_name) + year_str.len() + 1 {
            ctl.header_hint = true;
            break;
        }
    }
}

fn day_in_year(ctl: &CalControl, day: i32, month: usize, year: i32) -> i32 {
    let leap = leap_year(ctl, year);
    let mut d = day;
    for days in DAYS_IN_MONTH[leap].iter().take(month).skip(1) {
        d += *days as i32;
    }
    d
}

fn day_in_week(ctl: &CalControl, day: i32, month: usize, year: i32) -> i32 {
    const REFORM: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    const OLD: [i32; 12] = [5, 1, 0, 3, 5, 1, 3, 6, 2, 4, 0, 2];

    let m = if month > 0 && month <= 12 {
        month - 1
    } else {
        0
    };
    let mut y = year;

    if year != ctl.reform_year + 1 {
        y -= if month < 3 { 1 } else { 0 };
    } else {
        y -= if month < 3 { 1 } else { 0 } + 14;
    }

    if ctl.reform_year < year
        || (year == ctl.reform_year && REFORMATION_MONTH < month)
        || (year == ctl.reform_year && month == REFORMATION_MONTH && 13 < day)
    {
        return ((y as i64 + (y / 4) as i64 - (y / 100) as i64
            + (y / 400) as i64
            + REFORM[m] as i64
            + day as i64)
            % 7) as i32;
    }

    if year < ctl.reform_year
        || (year == ctl.reform_year && month < REFORMATION_MONTH)
        || (year == ctl.reform_year && month == REFORMATION_MONTH && day < 3)
    {
        return ((y as i64 + (y / 4) as i64 + OLD[m] as i64 + day as i64) % 7) as i32;
    }

    -1
}

fn week_number(day: i32, month: usize, year: i32, ctl: &CalControl) -> i32 {
    let wday = day_in_week(ctl, 1, 1, year);
    let mut fday = if ctl.weektype & 0x100 != 0 {
        wday + if wday >= 5 { -2 } else { 5 }
    } else {
        wday + 6
    };

    let mut m = month;
    if day > 31 {
        m = 1;
    }

    let yday = day_in_year(ctl, day, m, year);

    // NOTE: yday still counts the 11 days the reform year dropped, so
    // the offset takes them back out before the week is divided.
    if year == ctl.reform_year && yday >= YDAY_AFTER_MISSING {
        fday -= NUMBER_MISSING_DAYS;
    }

    if yday + fday < 7 {
        return week_number(31, 12, year - 1, ctl);
    }

    if ctl.weektype == 0x100 && yday >= 363 {
        let dow = day_in_week(ctl, day, month, year);
        let dow31 = day_in_week(ctl, 31, 12, year);
        if (1..=3).contains(&dow) && (1..=3).contains(&dow31) {
            return week_number(1, 1, year + 1, ctl);
        }
    }

    (yday + fday) / 7
}

fn cal_fill_month(month: &mut CalMonth, ctl: &CalControl) {
    let mut first_week_day = day_in_week(ctl, 1, month.month, month.year);
    let leap = leap_year(ctl, month.year);

    let mut j = if ctl.julian {
        day_in_year(ctl, 1, month.month, month.year)
    } else {
        1
    };

    let mut month_days = j + DAYS_IN_MONTH[leap][month.month] as i32;

    if ctl.weekstart != 0 {
        first_week_day -= ctl.weekstart as i32;
        if first_week_day < 0 {
            first_week_day = 7 - ctl.weekstart as i32;
        }
        month_days += ctl.weekstart as i32 - 1;
    }

    month.days = [SPACE; MAXDAYS];
    let mut weeklines = 0;

    for slot in month.days.iter_mut() {
        if first_week_day > 0 {
            first_week_day -= 1;
            continue;
        }
        if j < month_days {
            // NOTE: the reform year has no September 3 to 13, so the
            // grid jumps the 11 dropped days instead of drawing them.
            if month.year == ctl.reform_year
                && month.month == REFORMATION_MONTH
                && (j == 3 || j == 247)
            {
                j += NUMBER_MISSING_DAYS;
            }
            *slot = j;
            j += 1;
        } else {
            weeklines += 1;
        }
    }

    if ctl.weektype != 0 {
        let mut weeknum = week_number(1, month.month, month.year, ctl);
        let mut weeklines_count = MAXDAYS / DAYS_IN_WEEK - weeklines / DAYS_IN_WEEK;

        for i in 0..(MAXDAYS / DAYS_IN_WEEK) {
            if weeklines_count > 0 {
                if weeknum > 52 {
                    weeknum =
                        week_number(month.days[i * DAYS_IN_WEEK], month.month, month.year, ctl);
                }
                month.weeks[i] = weeknum;
                weeknum += 1;
                weeklines_count -= 1;
            } else {
                month.weeks[i] = SPACE;
            }
        }
    }
}

fn center(grid: &mut String, s: &str, width: usize, sep: usize) -> fmt::Result {
    let len = text_width(s);
    let pad = if width > len { (width - len) / 2 } else { 0 };
    write!(grid, "{}", " ".repeat(pad))?;
    write!(grid, "{s}")?;
    write!(grid, "{}", " ".repeat(width - len - pad))?;
    if sep > 0 {
        write!(grid, "{}", " ".repeat(sep))?;
    }
    Ok(())
}

fn cal_output_header(grid: &mut String, months: &[CalMonth], ctl: &CalControl) -> fmt::Result {
    for (i, m) in months.iter().enumerate() {
        let out = if ctl.header_hint || ctl.header_year {
            ctl.names.month(m.month).to_string()
        } else {
            format!("{} {}", ctl.names.month(m.month), m.year)
        };
        center(
            grid,
            &out,
            ctl.week_width,
            if i < months.len() - 1 {
                ctl.gutter_width
            } else {
                0
            },
        )?;
    }
    writeln!(grid)?;

    if ctl.header_hint && !ctl.header_year {
        for (i, m) in months.iter().enumerate() {
            center(
                grid,
                &format!("{}", m.year),
                ctl.week_width,
                if i < months.len() - 1 {
                    ctl.gutter_width
                } else {
                    0
                },
            )?;
        }
        writeln!(grid)?;
    }

    for (i, _) in months.iter().enumerate() {
        if ctl.weektype != 0 {
            if ctl.julian {
                write!(grid, "{}", " ".repeat(ctl.day_width - 1))?;
            } else {
                write!(grid, "   ")?;
            }
        }

        for d in 0..DAYS_IN_WEEK {
            let wd = (d + ctl.weekstart) % DAYS_IN_WEEK;
            if d > 0 {
                write!(grid, " ")?;
            }
            write!(grid, "{:>2}", ctl.names.weekdays[wd])?;
        }

        if i < months.len() - 1 {
            write!(grid, "{}", " ".repeat(ctl.gutter_width))?;
        }
    }

    writeln!(grid)
}

fn cal_output_months(grid: &mut String, months: &[CalMonth], ctl: &mut CalControl) -> fmt::Result {
    let today = Local::now();

    for week_line in 0..(MAXDAYS / DAYS_IN_WEEK) {
        for (mi, m) in months.iter().enumerate() {
            let mut reqday = 0;
            if m.month == ctl.req.month && m.year == ctl.req.year {
                reqday = if ctl.julian {
                    ctl.req.day
                } else {
                    ctl.req.day + 1 - day_in_year(ctl, 1, m.month, m.year)
                };
            }

            if ctl.weektype != 0 {
                if m.weeks[week_line] > 0 {
                    write!(grid, "{:2}", m.weeks[week_line])?;
                } else {
                    write!(grid, "  ")?;
                }
                write!(grid, " ")?;
            }

            let mut skip = if ctl.weektype != 0 {
                ctl.day_width
            } else {
                ctl.day_width - 1
            };

            for d in 0..DAYS_IN_WEEK {
                let idx = week_line * DAYS_IN_WEEK + d;
                let day = m.days[idx];

                if day > 0 {
                    let is_today = m.month == today.month() as usize
                        && m.year == today.year()
                        && day == today.day() as i32;

                    let (y, mm, dd) = if ctl.julian {
                        // NOTE: --julian fills the grid with ordinal
                        // days, so the calendar date has to be
                        // recovered before an event can match it.
                        let mut julian_day = day;
                        let leap = leap_year(ctl, m.year);
                        let mut month_idx = 1;
                        while month_idx <= 12 && julian_day > DAYS_IN_MONTH[leap][month_idx] as i32
                        {
                            julian_day -= DAYS_IN_MONTH[leap][month_idx] as i32;
                            month_idx += 1;
                        }
                        (m.year, month_idx as u32, julian_day as u32)
                    } else {
                        (m.year, m.month as u32, day as u32)
                    };

                    let has_event = collect_events(ctl, y, mm, dd);

                    if reqday == day || is_today {
                        write!(
                            grid,
                            "{}\x1b[7m{:width$}\x1b[0m",
                            " ".repeat(skip - if ctl.julian { 3 } else { 2 }),
                            day,
                            width = if ctl.julian { 3 } else { 2 },
                        )?;
                    } else if has_event {
                        write!(
                            grid,
                            "{}\x1b[44m{:width$}\x1b[0m",
                            " ".repeat(skip - if ctl.julian { 3 } else { 2 }),
                            day,
                            width = if ctl.julian { 3 } else { 2 }
                        )?;
                    } else {
                        write!(grid, "{:width$}", day, width = skip)?;
                    }
                } else {
                    write!(grid, "{}", " ".repeat(skip))?;
                }

                if skip < ctl.day_width {
                    skip += 1;
                }
            }

            if mi < months.len() - 1 {
                write!(grid, "{}", " ".repeat(ctl.gutter_width))?;
            }
        }
        writeln!(grid)?;
    }

    Ok(())
}

fn cal_vert_output_header(grid: &mut String, months: &[CalMonth], ctl: &CalControl) -> fmt::Result {
    write!(grid, "{}", " ".repeat(ctl.day_width + 1))?;

    let month_width = ctl.day_width * (MAXDAYS / DAYS_IN_WEEK);

    for (i, m) in months.iter().enumerate() {
        let out = if ctl.header_hint || ctl.header_year {
            ctl.names.month(m.month).to_string()
        } else {
            format!("{} {}", ctl.names.month(m.month), m.year)
        };
        write!(grid, "{:<width$}", out, width = month_width)?;
        if i < months.len() - 1 {
            write!(grid, "{}", " ".repeat(ctl.gutter_width))?;
        }
    }
    writeln!(grid)?;

    if ctl.header_hint && !ctl.header_year {
        write!(grid, "{}", " ".repeat(ctl.day_width + 1))?;
        for (i, m) in months.iter().enumerate() {
            write!(grid, "{:<width$}", m.year, width = month_width)?;
            if i < months.len() - 1 {
                write!(grid, "{}", " ".repeat(ctl.gutter_width))?;
            }
        }
        writeln!(grid)?;
    }

    Ok(())
}

fn cal_vert_output_months(
    grid: &mut String,
    months: &[CalMonth],
    ctl: &mut CalControl,
) -> fmt::Result {
    let today = Local::now();

    for i in 0..DAYS_IN_WEEK {
        let wd = (i + ctl.weekstart) % DAYS_IN_WEEK;
        write!(
            grid,
            "{:<width$}",
            ctl.names.weekdays[wd],
            width = ctl.day_width - 1
        )?;

        for (mi, m) in months.iter().enumerate() {
            let mut reqday = 0;
            if m.month == ctl.req.month && m.year == ctl.req.year {
                reqday = if ctl.julian {
                    ctl.req.day
                } else {
                    ctl.req.day + 1 - day_in_year(ctl, 1, m.month, m.year)
                };
            }

            let mut skip = ctl.day_width;
            for week in 0..(MAXDAYS / DAYS_IN_WEEK) {
                let d = i + DAYS_IN_WEEK * week;
                let day = m.days[d];

                if day > 0 {
                    let is_today = m.month == today.month() as usize
                        && m.year == today.year()
                        && day == today.day() as i32;

                    let (y, mm, dd) = if ctl.julian {
                        // NOTE: --julian fills the grid with ordinal
                        // days, so the calendar date has to be
                        // recovered before an event can match it.
                        let mut julian_day = day;
                        let leap = leap_year(ctl, m.year);
                        let mut month_idx = 1;
                        while month_idx <= 12 && julian_day > DAYS_IN_MONTH[leap][month_idx] as i32
                        {
                            julian_day -= DAYS_IN_MONTH[leap][month_idx] as i32;
                            month_idx += 1;
                        }
                        (m.year, month_idx as u32, julian_day as u32)
                    } else {
                        (m.year, m.month as u32, day as u32)
                    };

                    let has_event = collect_events(ctl, y, mm, dd);

                    if reqday == day || is_today {
                        write!(
                            grid,
                            "{}\x1b[7m{:width$}\x1b[0m",
                            " ".repeat(skip - if ctl.julian { 3 } else { 2 }),
                            day,
                            width = if ctl.julian { 3 } else { 2 },
                        )?;
                    } else if has_event {
                        write!(
                            grid,
                            "{}\x1b[44m{:width$}\x1b[0m",
                            " ".repeat(skip - if ctl.julian { 3 } else { 2 }),
                            day,
                            width = if ctl.julian { 3 } else { 2 },
                        )?;
                    } else {
                        write!(grid, "{:width$}", day, width = skip)?;
                    }
                } else {
                    write!(grid, "{}", " ".repeat(skip))?;
                }
                skip = ctl.day_width;
            }

            if mi < months.len() - 1 {
                write!(grid, "{}", " ".repeat(ctl.gutter_width))?;
            }
        }
        writeln!(grid)?;
    }

    if ctl.weektype != 0 {
        write!(grid, "{}", " ".repeat(ctl.day_width - 1))?;
        for (mi, m) in months.iter().enumerate() {
            for week in 0..(MAXDAYS / DAYS_IN_WEEK) {
                if m.weeks[week] > 0 {
                    write!(
                        grid,
                        "{:width$}",
                        m.weeks[week],
                        width = if ctl.julian { 3 } else { 2 }
                    )?;
                } else {
                    write!(grid, "{}", " ".repeat(if ctl.julian { 3 } else { 2 }))?;
                }
                write!(grid, " ")?;
            }
            if mi < months.len() - 1 {
                write!(grid, "{}", " ".repeat(ctl.gutter_width - 1))?;
            }
        }
        writeln!(grid)?;
    }

    Ok(())
}

/// Whether an event starts on the day `(y, m, d)`, collecting all of
/// them into `ctl.events`.
///
/// An instant keeps every event starting at it rather than the last one
/// seen: two unrelated meetings at 09:00 are two meetings, and so are
/// two resources a collection holds under one `UID`.
fn collect_events(ctl: &mut CalControl, y: i32, m: u32, d: u32) -> bool {
    let starting: Vec<(NaiveDateTime, AgendaEvent)> = ctl
        .all_events
        .iter()
        .filter_map(|event| {
            let start = event.start_at()?;
            let on_day = start.year() == y && start.month() == m && start.day() == d;
            on_day.then(|| {
                let collected = AgendaEvent {
                    id: event.id.clone(),
                    label: event.label().to_owned(),
                };

                (start, collected)
            })
        })
        .collect();

    let has_event = !starting.is_empty();

    for (start, event) in starting {
        ctl.events.entry(start).or_default().push(event);
    }

    has_event
}

/// The first month the grid draws, as `(year, month)`: the requested
/// one, moved back by half the month count when the months span it.
fn first_month(ctl: &CalControl) -> (i32, usize) {
    let mut month = if ctl.req.start_month > 0 {
        ctl.req.start_month
    } else {
        ctl.req.month
    };
    let mut year = ctl.req.year;

    if ctl.span_months {
        let new_month = month as i32 - ctl.num_months as i32 / 2;
        if new_month < 1 {
            let nm = -new_month;
            year -= (nm / MONTHS_IN_YEAR as i32) + 1;
            month = if nm as usize > MONTHS_IN_YEAR {
                MONTHS_IN_YEAR - (nm as usize % MONTHS_IN_YEAR)
            } else {
                MONTHS_IN_YEAR - nm as usize
            };
        } else {
            month = new_month as usize;
        }
    }

    (year, month)
}

/// The first and last days the grid draws, both inclusive, or `None`
/// for a grid chrono cannot place.
fn drawn_days(ctl: &CalControl) -> Option<(NaiveDate, NaiveDate)> {
    let (year, month) = first_month(ctl);
    let first = NaiveDate::from_ymd_opt(year, month as u32, 1)?;
    let last = first
        .checked_add_months(Months::new(ctl.num_months as u32))?
        .pred_opt()?;
    Some((first, last))
}

fn monthly(grid: &mut String, ctl: &mut CalControl) -> fmt::Result {
    let (mut year, mut month) = first_month(ctl);

    let rows = (ctl.num_months - 1) / ctl.months_in_row;

    for i in 0..=rows {
        let mut n = ctl.months_in_row;
        if i == rows && !ctl.num_months.is_multiple_of(ctl.months_in_row) {
            n = ctl.num_months % ctl.months_in_row;
        }

        let mut ms = vec![
            CalMonth {
                days: [SPACE; MAXDAYS],
                weeks: [SPACE; MAXDAYS / DAYS_IN_WEEK],
                month,
                year,
            };
            n
        ];

        for m in ms.iter_mut() {
            m.month = month;
            m.year = year;
            cal_fill_month(m, ctl);
            month += 1;
            if month > MONTHS_IN_YEAR {
                year += 1;
                month = 1;
            }
        }

        if ctl.vertical {
            if i > 0 {
                writeln!(grid)?;
            }
            cal_vert_output_header(grid, &ms, ctl)?;
            cal_vert_output_months(grid, &ms, ctl)?;
        } else {
            cal_output_header(grid, &ms, ctl)?;
            cal_output_months(grid, &ms, ctl)?;
        }
    }

    Ok(())
}

fn yearly(grid: &mut String, ctl: &mut CalControl) -> fmt::Result {
    if ctl.header_year {
        let year_width =
            ctl.months_in_row * ctl.week_width + (ctl.months_in_row - 1) * ctl.gutter_width;
        center(grid, &format!("{}", ctl.req.year), year_width, 0)?;
        writeln!(grid)?;
    }
    monthly(grid, ctl)
}

/// One event the agenda collected while painting the grid.
#[derive(Clone)]
struct AgendaEvent {
    /// The id of the item the event was projected from.
    ///
    /// Two resources of one calendar may carry one `UID`, so the id is
    /// what tells them apart.
    id: String,
    /// What the agenda prints for the event: its summary, falling back
    /// to its description.
    label: String,
}

/// The agenda output: the drawn grid, plus the events it holds grouped
/// by DTSTART.
///
/// An instant holds every event starting at it, so a calendar holding
/// two meetings at 09:00 renders two lines and reports two labels.
pub struct EventAgendaOutput {
    grid: String,
    events: BTreeMap<NaiveDateTime, Vec<AgendaEvent>>,
    /// The locale the event times are stamped in; the JSON payload
    /// ignores it.
    locale: Locale,
}

impl EventAgendaOutput {
    /// Takes the grid and the collected events, ordering the events of
    /// one instant by label then by item id.
    ///
    /// Two copies of one event share a label, so the id is what breaks
    /// the tie and makes the order total: the same calendar renders the
    /// same way twice.
    fn new(grid: String, mut events: BTreeMap<NaiveDateTime, Vec<AgendaEvent>>) -> Self {
        for at_instant in events.values_mut() {
            at_instant.sort_by(|a, b| a.label.cmp(&b.label).then_with(|| a.id.cmp(&b.id)));
        }

        Self {
            grid,
            events,
            locale: Locale::POSIX,
        }
    }

    /// Stamps the event times with the month names of `locale`.
    fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }
}

impl fmt::Display for EventAgendaOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)?;

        for (date, events) in &self.events {
            // NaiveDateTime has no format_localized: the names live in the date.
            let day = date.date().format_localized("%b %d", self.locale);
            let stamp = format!("{day}, {}", date.format("%R"));
            let padding = " ".repeat(text_width(&stamp) + 2);

            for (nth, event) in events.iter().enumerate() {
                match nth {
                    0 => writeln!(f, "{stamp}: {}", event.label)?,
                    _ => writeln!(f, "{padding}{}", event.label)?,
                }
            }
        }

        Ok(())
    }
}

impl Serialize for EventAgendaOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.events.len()))?;

        for (date, events) in &self.events {
            let labels: Vec<&str> = events.iter().map(|event| event.label.as_str()).collect();
            map.serialize_entry(date, &labels)?;
        }

        map.end()
    }
}

/// Describes exactly what the hand-written [`Serialize`] above emits,
/// an instant keyed to the labels of the events starting at it, so the
/// printed shape and the published schema cannot drift apart.
impl JsonSchema for EventAgendaOutput {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("EventAgendaOutput")
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <BTreeMap<String, Vec<String>> as JsonSchema>::json_schema(generator)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use serde_json::to_string;

    use super::*;

    fn instant(hour: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 8, 14)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap()
    }

    fn agenda(collected: Vec<(NaiveDateTime, &str, &str)>) -> EventAgendaOutput {
        let mut events: BTreeMap<NaiveDateTime, Vec<AgendaEvent>> = BTreeMap::new();

        for (start, id, label) in collected {
            let event = AgendaEvent {
                id: id.into(),
                label: label.into(),
            };

            events.entry(start).or_default().push(event);
        }

        EventAgendaOutput::new(String::new(), events)
    }

    fn grid(year: i32, month: usize, num_months: usize, span_months: bool) -> CalControl {
        CalControl {
            reform_year: DEFAULT_REFORM_YEAR,
            num_months,
            span_months,
            months_in_row: 0,
            weekstart: 0,
            weektype: 0,
            day_width: DAY_LEN,
            week_width: 0,
            gutter_width: 2,
            julian: false,
            header_year: false,
            header_hint: false,
            vertical: false,
            req: CalRequest {
                day: 0,
                month,
                year,
                start_month: 0,
            },
            names: Names::new(Locale::POSIX),
            all_events: Vec::new(),
            events: BTreeMap::new(),
        }
    }

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn one_month_draws_its_own_days() {
        assert_eq!(
            drawn_days(&grid(2026, 2, 1, false)),
            Some((day(2026, 2, 1), day(2026, 2, 28)))
        );
    }

    /// `-3` centres on the month, so January reaches back into December.
    #[test]
    fn three_spanning_months_reach_across_the_year() {
        assert_eq!(
            drawn_days(&grid(2027, 1, 3, true)),
            Some((day(2026, 12, 1), day(2027, 2, 28)))
        );
    }

    #[test]
    fn a_year_draws_january_to_december() {
        let mut ctl = grid(2026, 10, 12, false);
        ctl.req.start_month = 1;

        assert_eq!(drawn_days(&ctl), Some((day(2026, 1, 1), day(2026, 12, 31))));
    }

    #[test]
    fn two_events_at_one_instant_both_render_under_one_time() {
        let agenda = agenda(vec![
            (instant(9), "1", "Pre demo woonies"),
            (instant(9), "2", "Pre demo MINIS"),
            (instant(8), "3", "Breakfast"),
        ]);

        assert_eq!(
            agenda.to_string(),
            concat!(
                "Aug 14, 08:00: Breakfast\n",
                "Aug 14, 09:00: Pre demo MINIS\n",
                "               Pre demo woonies\n",
            )
        );
    }

    #[test]
    fn two_events_at_one_instant_both_reach_the_json_payload() {
        let agenda = agenda(vec![
            (instant(9), "1", "Pre demo woonies"),
            (instant(9), "2", "Pre demo MINIS"),
        ]);

        assert_eq!(
            to_string(&agenda).unwrap(),
            r#"{"2026-08-14T09:00:00":["Pre demo MINIS","Pre demo woonies"]}"#
        );
    }

    /// Two copies of one event share a label, so only the id orders
    /// them.
    #[test]
    fn two_events_sharing_a_label_are_ordered_by_their_ids() {
        let agenda = agenda(vec![
            (instant(9), "2", "Stand-up"),
            (instant(9), "1", "Stand-up"),
        ]);
        let ids: Vec<&str> = agenda.events[&instant(9)]
            .iter()
            .map(|event| event.id.as_str())
            .collect();

        assert_eq!(ids, ["1", "2"]);
    }

    /// A lone event still prints one line: only its JSON value became a
    /// list.
    #[test]
    fn a_lone_event_renders_as_one_line() {
        let agenda = agenda(vec![(instant(9), "1", "Stand-up")]);

        assert_eq!(agenda.to_string(), "Aug 14, 09:00: Stand-up\n");
        assert_eq!(
            to_string(&agenda).unwrap(),
            r#"{"2026-08-14T09:00:00":["Stand-up"]}"#
        );
    }

    #[test]
    fn the_locale_drops_its_codeset_and_modifier() {
        assert_eq!(parse_locale("es_ES.UTF-8"), Locale::es_ES);
        assert_eq!(parse_locale("de_DE.UTF-8@euro"), Locale::de_DE);
        assert_eq!(parse_locale("fr_FR"), Locale::fr_FR);
    }

    #[test]
    fn an_unknown_locale_falls_back_to_posix() {
        assert_eq!(parse_locale("C"), Locale::POSIX);
        assert_eq!(parse_locale("C.UTF-8"), Locale::POSIX);
        assert_eq!(parse_locale("xx_YY"), Locale::POSIX);
    }

    #[test]
    fn posix_names_are_the_english_ones() {
        let names = Names::new(Locale::POSIX);

        assert_eq!(names.month(10), "October");
        assert_eq!(names.weekdays.join(" "), "Su Mo Tu We Th Fr Sa");
    }

    #[test]
    fn spanish_names_match_cal() {
        let names = Names::new(Locale::es_ES);

        assert_eq!(names.month(10), "octubre");
        assert_eq!(names.weekdays.join(" "), "do lu ma mi ju vi sá");
    }

    #[test]
    fn a_month_is_read_in_english_or_in_the_locale() {
        let names = Names::new(Locale::es_ES);

        assert_eq!(parse_month("octubre", &names), 10);
        assert_eq!(parse_month("October", &names), 10);
        assert_eq!(parse_month("ago", &names), 8);
        assert_eq!(parse_month("10", &names), 10);
    }

    #[test]
    fn a_header_with_accents_stays_centred() {
        let mut grid = String::new();
        center(&mut grid, "sá", 4, 0).unwrap();

        assert_eq!(grid, " sá ");
    }

    #[test]
    fn event_times_are_stamped_in_the_locale() {
        let agenda = agenda(vec![
            (instant(9), "1", "Stand-up"),
            (instant(9), "2", "Café"),
        ])
        .with_locale(Locale::es_ES);

        assert_eq!(
            agenda.to_string(),
            "ago 14, 09:00: Café\n               Stand-up\n"
        );
    }
}
