// This file is part of Calendula, a CLI to manage calendars.
//
// Copyright (C) 2025 soywod <clement.douin@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use anyhow::Result;
use chrono::{Datelike, Local};
use clap::Parser;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

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

/// List all events.
///
/// This command allows you to list iCalendars from a given calendar.
#[derive(Debug, Parser)]
pub struct AgendaCommand {
    /// The identifier of the CalDAV calendar to list iCalendars from.
    #[arg(name = "CALENDAR-ID")]
    calendar_id: String,

    #[arg(name = "DATE")]
    date: Vec<String>,

    #[arg(short = '1', long)]
    one: bool,

    #[arg(short = '3', long)]
    three: bool,

    #[arg(short = 'n', long)]
    months: Option<u32>,

    #[arg(short = 'S', long)]
    span: bool,

    #[arg(short = 's', long)]
    sunday: bool,

    #[arg(short = 'm', long)]
    monday: bool,

    #[arg(short = 'j', long)]
    julian: bool,

    #[arg(long)]
    reform: Option<String>,

    #[arg(long)]
    iso: bool,

    #[arg(short = 'y', long)]
    year: bool,

    #[arg(short = 'Y', long)]
    twelve: bool,

    #[arg(short = 'w', long)]
    week: bool,

    #[arg(short = 'v', long)]
    vertical: bool,
}

impl AgendaCommand {
    pub fn execute(self, _printer: &mut impl Printer, _account: Account) -> Result<()> {
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
        };

        // Reform year
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

        // Week options
        if self.monday {
            ctl.weekstart = 1;
        }
        if self.sunday {
            ctl.weekstart = 0;
        }

        // Display options
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

        // Week numbers (ONLY affects display, not logic)
        if self.week {
            ctl.weektype = if ctl.weekstart == 1 { 0x100 } else { 0x200 };
            ctl.week_width = ctl.day_width * DAYS_IN_WEEK + WNUM_LEN - 1;
        } else {
            ctl.week_width = ctl.day_width * DAYS_IN_WEEK - 1;
        }

        // Parse date arguments
        #[allow(unused_assignments)]
        let (mut req_day, mut req_month, mut req_year) = (0, 0, 0);
        let mut yflag = self.year;
        let yflag_cap = self.twelve;

        match self.date.len() {
            3 => {
                // day month year
                req_day = self.date[0].parse().unwrap_or(1);
                req_month = parse_month(&self.date[1]);
                req_year = self.date[2].parse().unwrap_or(now.year());
                let dm = DAYS_IN_MONTH[leap_year(&ctl, req_year)][req_month];
                if req_day > dm as i32 {
                    eprintln!("illegal day value: use 1-{}", dm);
                    std::process::exit(1);
                }
                req_day = day_in_year(&ctl, req_day, req_month, req_year);
            }
            2 => {
                // month year
                req_month = parse_month(&self.date[0]);
                req_year = self.date[1].parse().unwrap_or(now.year());
            }
            1 => {
                // year only - show whole year
                req_year = self.date[0].parse().unwrap_or(now.year());
                if req_year < 1 {
                    eprintln!("illegal year value: use positive integer");
                    std::process::exit(1);
                }
                if req_year == now.year() {
                    req_day = now.ordinal() as i32;
                }
                req_month = now.month() as usize;
                if ctl.num_months == 0 {
                    yflag = true;
                }
            }
            _ => {
                // no arguments - current month
                req_day = now.ordinal() as i32;
                req_month = now.month() as usize;
                req_year = now.year();
            }
        }

        ctl.req.day = req_day;
        ctl.req.month = req_month;
        ctl.req.year = req_year;

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

        if yflag || yflag_cap {
            yearly(&ctl);
        } else {
            monthly(&ctl);
        }

        Ok(())
    }
}

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
}

#[derive(Clone)]
struct CalRequest {
    day: i32,
    month: usize,
    year: i32,
    start_month: usize,
}

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

const FULL_MONTH: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const WEEKDAYS: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

fn parse_month(s: &str) -> usize {
    if let Ok(m) = s.parse::<usize>() {
        if m >= 1 && m <= 12 {
            return m;
        }
    }
    let lower = s.to_lowercase();
    for (i, name) in FULL_MONTH.iter().enumerate() {
        if name.to_lowercase().starts_with(&lower) {
            return i + 1;
        }
    }
    1
}

fn leap_year(ctl: &CalControl, year: i32) -> usize {
    if year <= ctl.reform_year {
        if year % 4 == 0 {
            1
        } else {
            0
        }
    } else {
        if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
            1
        } else {
            0
        }
    }
}

fn headers_init(ctl: &mut CalControl) {
    let year_str = format!("{}", ctl.req.year);
    for month_name in &FULL_MONTH {
        if ctl.week_width < month_name.len() + year_str.len() + 1 {
            ctl.header_hint = true;
            break;
        }
    }
}

fn day_in_year(ctl: &CalControl, day: i32, month: usize, year: i32) -> i32 {
    let leap = leap_year(ctl, year);
    let mut d = day;
    for i in 1..month {
        d += DAYS_IN_MONTH[leap][i] as i32;
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

    if year == ctl.reform_year && yday >= YDAY_AFTER_MISSING {
        fday -= NUMBER_MISSING_DAYS;
    }

    if yday + fday < 7 {
        return week_number(31, 12, year - 1, ctl);
    }

    if ctl.weektype == 0x100 && yday >= 363 {
        let dow = day_in_week(ctl, day, month, year);
        let dow31 = day_in_week(ctl, 31, 12, year);
        if dow >= 1 && dow <= 3 && dow31 >= 1 && dow31 <= 3 {
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

    for i in 0..MAXDAYS {
        if first_week_day > 0 {
            first_week_day -= 1;
            continue;
        }
        if j < month_days {
            if month.year == ctl.reform_year
                && month.month == REFORMATION_MONTH
                && (j == 3 || j == 247)
            {
                j += NUMBER_MISSING_DAYS;
            }
            month.days[i] = j;
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

fn center(s: &str, width: usize, sep: usize) {
    let len = s.len();
    let pad = if width > len { (width - len) / 2 } else { 0 };
    print!("{}{}{}", " ".repeat(pad), s, " ".repeat(width - len - pad));
    if sep > 0 {
        print!("{}", " ".repeat(sep));
    }
}

fn cal_output_header(months: &[CalMonth], ctl: &CalControl) {
    for (i, m) in months.iter().enumerate() {
        let out = if ctl.header_hint || ctl.header_year {
            format!("{}", FULL_MONTH[m.month - 1])
        } else {
            format!("{} {}", FULL_MONTH[m.month - 1], m.year)
        };
        center(
            &out,
            ctl.week_width,
            if i < months.len() - 1 {
                ctl.gutter_width
            } else {
                0
            },
        );
    }
    println!();

    if ctl.header_hint && !ctl.header_year {
        for (i, m) in months.iter().enumerate() {
            center(
                &format!("{}", m.year),
                ctl.week_width,
                if i < months.len() - 1 {
                    ctl.gutter_width
                } else {
                    0
                },
            );
        }
        println!();
    }

    for (i, _) in months.iter().enumerate() {
        if ctl.weektype != 0 {
            if ctl.julian {
                print!("{}", " ".repeat(ctl.day_width - 1));
            } else {
                print!("   ");
            }
        }

        for d in 0..DAYS_IN_WEEK {
            let wd = (d + ctl.weekstart) % DAYS_IN_WEEK;
            if d > 0 {
                print!(" ");
            }
            print!("{:>2}", WEEKDAYS[wd]);
        }

        if i < months.len() - 1 {
            print!("{}", " ".repeat(ctl.gutter_width));
        }
    }
    println!();
}

fn cal_output_months(months: &[CalMonth], ctl: &CalControl) {
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
                    print!("{:2}", m.weeks[week_line]);
                } else {
                    print!("  ");
                }
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

                    if reqday == day || is_today {
                        print!(
                            "{}\x1b[7m{:width$}\x1b[0m",
                            " ".repeat(skip - if ctl.julian { 3 } else { 2 }),
                            day,
                            width = if ctl.julian { 3 } else { 2 }
                        );
                    } else {
                        print!("{:width$}", day, width = skip);
                    }
                } else {
                    print!("{}", " ".repeat(skip));
                }

                if skip < ctl.day_width {
                    skip += 1;
                }
            }

            if mi < months.len() - 1 {
                print!("{}", " ".repeat(ctl.gutter_width));
            }
        }
        println!();
    }
}

fn cal_vert_output_header(months: &[CalMonth], ctl: &CalControl) {
    print!("{}", " ".repeat(ctl.day_width + 1));

    let month_width = ctl.day_width * (MAXDAYS / DAYS_IN_WEEK);

    for (i, m) in months.iter().enumerate() {
        let out = if ctl.header_hint || ctl.header_year {
            format!("{}", FULL_MONTH[m.month - 1])
        } else {
            format!("{} {}", FULL_MONTH[m.month - 1], m.year)
        };
        print!("{:<width$}", out, width = month_width);
        if i < months.len() - 1 {
            print!("{}", " ".repeat(ctl.gutter_width));
        }
    }
    println!();

    if ctl.header_hint && !ctl.header_year {
        print!("{}", " ".repeat(ctl.day_width + 1));
        for (i, m) in months.iter().enumerate() {
            print!("{:<width$}", m.year, width = month_width);
            if i < months.len() - 1 {
                print!("{}", " ".repeat(ctl.gutter_width));
            }
        }
        println!();
    }
}

fn cal_vert_output_months(months: &[CalMonth], ctl: &CalControl) {
    let today = Local::now();

    for i in 0..DAYS_IN_WEEK {
        let wd = (i + ctl.weekstart) % DAYS_IN_WEEK;
        print!("{:<width$}", WEEKDAYS[wd], width = ctl.day_width - 1);

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

                    if reqday == day || is_today {
                        print!(
                            "{}\x1b[7m{:width$}\x1b[0m",
                            " ".repeat(skip - if ctl.julian { 3 } else { 2 }),
                            day,
                            width = if ctl.julian { 3 } else { 2 }
                        );
                    } else {
                        print!("{:width$}", day, width = skip);
                    }
                } else {
                    print!("{}", " ".repeat(skip));
                }
                skip = ctl.day_width;
            }

            if mi < months.len() - 1 {
                print!("{}", " ".repeat(ctl.gutter_width));
            }
        }
        println!();
    }

    if ctl.weektype != 0 {
        print!("{}", " ".repeat(ctl.day_width - 1));
        for (mi, m) in months.iter().enumerate() {
            for week in 0..(MAXDAYS / DAYS_IN_WEEK) {
                if m.weeks[week] > 0 {
                    print!(
                        "{:width$}",
                        m.weeks[week],
                        width = if ctl.julian { 3 } else { 2 }
                    );
                } else {
                    print!("{}", " ".repeat(if ctl.julian { 3 } else { 2 }));
                }
                print!(" ");
            }
            if mi < months.len() - 1 {
                print!("{}", " ".repeat(ctl.gutter_width - 1));
            }
        }
        println!();
    }
}

fn monthly(ctl: &CalControl) {
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

    let rows = (ctl.num_months - 1) / ctl.months_in_row;

    for i in 0..=rows {
        let mut n = ctl.months_in_row;
        if i == rows && ctl.num_months % ctl.months_in_row > 0 {
            n = ctl.num_months % ctl.months_in_row;
        }

        let mut ms = vec![
            CalMonth {
                days: [SPACE; MAXDAYS],
                weeks: [SPACE; MAXDAYS / DAYS_IN_WEEK],
                month,
                year
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
                println!();
            }
            cal_vert_output_header(&ms, ctl);
            cal_vert_output_months(&ms, ctl);
        } else {
            cal_output_header(&ms, ctl);
            cal_output_months(&ms, ctl);
        }
    }
}

fn yearly(ctl: &CalControl) {
    if ctl.header_year {
        let year_width =
            ctl.months_in_row * ctl.week_width + (ctl.months_in_row - 1) * ctl.gutter_width;
        center(&format!("{}", ctl.req.year), year_width, 0);
        println!("\n");
    }
    monthly(ctl);
}
