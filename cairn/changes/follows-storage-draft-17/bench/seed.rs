//! Seeds a pimdir calendar store for calendula's benches: one account
//! `bench`, a calendar `bench/calendar` of N events and `bench/holidays` of
//! 200, landed as a CalDAV sync round lands them (StoreObject then
//! UpsertPlacement per member, the round opened, stamped and closed), then
//! the search index built. The bodies are drawn from a fixed seed, so the
//! same N gives byte-identical bodies under io-pimdir 0.19.1 and 7f16b27.
//!
//! usage: seed <dir> <events> [--no-index]

use std::{collections::BTreeSet, env, path::PathBuf, time::Instant};

use io_pimdir::{
    change::PimdirWriteOp,
    client::{
        PimdirStore,
        search::{PimdirIndex, PimdirIndexRefreshOptions},
    },
    collection::{PimdirCheckpoint, PimdirCollectionId, PimdirScope},
    object::PimdirObject,
    placement::{
        PimdirBase, PimdirFlags, PimdirHandle, PimdirLevel, PimdirPlacement, PimdirStatus,
    },
    summary::calendar::derive,
};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len() as u64) as usize]
    }
}

const WORDS: [&str; 40] = [
    "budget", "review", "planning", "quarter", "offsite", "lisbon", "standup", "retro",
    "design", "roadmap", "hiring", "interview", "lunch", "dentist", "school", "football",
    "release", "migration", "pimdir", "calendar", "invoice", "contract", "travel", "train",
    "flight", "hotel", "workshop", "training", "security", "audit", "backup", "server",
    "garden", "piano", "birthday", "dinner", "concert", "doctor", "yoga", "marathon",
];

const PEOPLE: [&str; 10] = [
    "alice", "bob", "carol", "dave", "erin", "frank", "gina", "hugo", "ines", "jules",
];

const PARIS: &str = "BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\nBEGIN:DAYLIGHT\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nTZNAME:CEST\r\nDTSTART:19700329T020000\r\nRRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU\r\nEND:DAYLIGHT\r\nBEGIN:STANDARD\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nTZNAME:CET\r\nDTSTART:19701025T030000\r\nRRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\n";

const OUTLOOK: &str = "BEGIN:VTIMEZONE\r\nTZID:W. Europe Standard Time\r\nBEGIN:STANDARD\r\nDTSTART:16011028T030000\r\nRRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=10\r\nTZOFFSETFROM:+0200\r\nTZOFFSETTO:+0100\r\nEND:STANDARD\r\nBEGIN:DAYLIGHT\r\nDTSTART:16010325T020000\r\nRRULE:FREQ=YEARLY;BYDAY=-1SU;BYMONTH=3\r\nTZOFFSETFROM:+0100\r\nTZOFFSETTO:+0200\r\nEND:DAYLIGHT\r\nEND:VTIMEZONE\r\n";

/// Days from 1970-01-01 to a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn date(days: i64) -> String {
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}{m:02}{d:02}")
}

fn stamp(days: i64, minutes: i64) -> String {
    format!("{}T{:02}{:02}00", date(days), minutes / 60, minutes % 60)
}

fn words(rng: &mut Rng, n: u64) -> String {
    (0..n).map(|_| rng.pick(&WORDS)).collect::<Vec<_>>().join(" ")
}

const WEEKDAYS: [&str; 7] = ["TH", "FR", "SA", "SU", "MO", "TU", "WE"];

/// One calendar object resource, its UID `e<calendar>-<i>@bench`.
fn body(rng: &mut Rng, calendar: &str, i: usize) -> String {
    let uid = format!("e{calendar}-{i}@bench");
    let first = days_from_civil(2016, 1, 1);
    let span = days_from_civil(2031, 1, 1) - first;
    let day = first + rng.below(span as u64) as i64;
    let minutes = 7 * 60 + 15 * rng.below(48) as i64;
    let length = 30 * (1 + rng.below(4) as i64);
    let shape = rng.below(100);

    let mut zones = String::new();
    let (dtstart, dtend, rule, zone): (String, String, Option<String>, Option<&str>) = match shape
    {
        0..=35 => {
            let at = stamp(day, minutes);
            let end = stamp(day, (minutes + length).min(23 * 60 + 59));
            (format!("DTSTART:{at}Z"), format!("DTEND:{end}Z"), None, None)
        }
        36..=59 => {
            zones.push_str(PARIS);
            let at = stamp(day, minutes);
            let end = stamp(day, (minutes + length).min(23 * 60 + 59));
            (
                format!("DTSTART;TZID=Europe/Paris:{at}"),
                format!("DTEND;TZID=Europe/Paris:{end}"),
                None,
                Some("Europe/Paris"),
            )
        }
        60..=69 => {
            let days = 1 + rng.below(3) as i64;
            (
                format!("DTSTART;VALUE=DATE:{}", date(day)),
                format!("DTEND;VALUE=DATE:{}", date(day + days)),
                None,
                None,
            )
        }
        70..=71 => {
            let at = stamp(day, minutes);
            let end = stamp(day, (minutes + length).min(23 * 60 + 59));
            (format!("DTSTART:{at}"), format!("DTEND:{end}"), None, None)
        }
        _ => {
            let start = first + rng.below((span - 2 * 365) as u64) as i64;
            let weekday = WEEKDAYS[(start.rem_euclid(7)) as usize];
            let at = stamp(start, minutes);
            let end = stamp(start, (minutes + length).min(23 * 60 + 59));
            match shape {
                72..=81 => {
                    zones.push_str(PARIS);
                    let until = format!("{}T220000Z", date(start + 730));
                    (
                        format!("DTSTART;TZID=Europe/Paris:{at}"),
                        format!("DTEND;TZID=Europe/Paris:{end}"),
                        Some(format!("FREQ=WEEKLY;BYDAY={weekday};UNTIL={until}")),
                        Some("Europe/Paris"),
                    )
                }
                82..=87 => {
                    zones.push_str(PARIS);
                    (
                        format!("DTSTART;TZID=Europe/Paris:{at}"),
                        format!("DTEND;TZID=Europe/Paris:{end}"),
                        Some(format!("FREQ=WEEKLY;BYDAY={weekday}")),
                        Some("Europe/Paris"),
                    )
                }
                88..=91 => (
                    format!("DTSTART:{at}Z"),
                    format!("DTEND:{end}Z"),
                    Some(format!("FREQ=DAILY;COUNT={}", 10 + rng.below(20))),
                    None,
                ),
                92..=94 => {
                    let (_, _, d) = civil_from_days(start);
                    (
                        format!("DTSTART:{at}Z"),
                        format!("DTEND:{end}Z"),
                        Some(format!("FREQ=MONTHLY;BYMONTHDAY={}", d.min(28))),
                        None,
                    )
                }
                95..=97 => (
                    format!("DTSTART;VALUE=DATE:{}", date(start)),
                    format!("DTEND;VALUE=DATE:{}", date(start + 1)),
                    Some(String::from("FREQ=YEARLY")),
                    None,
                ),
                _ => {
                    zones.push_str(OUTLOOK);
                    (
                        format!("DTSTART;TZID=W. Europe Standard Time:{at}"),
                        format!("DTEND;TZID=W. Europe Standard Time:{end}"),
                        Some(format!("FREQ=WEEKLY;BYDAY={weekday}")),
                        Some("W. Europe Standard Time"),
                    )
                }
            }
        }
    };

    let summary_words = 2 + rng.below(3);
    let summary = words(rng, summary_words);
    let description_words = 5 + rng.below(25);
    let description = words(rng, description_words);
    let mut event = String::new();
    event.push_str("BEGIN:VEVENT\r\n");
    event.push_str(&format!("UID:{uid}\r\n"));
    event.push_str("DTSTAMP:20261101T120000Z\r\n");
    event.push_str(&format!("{dtstart}\r\n{dtend}\r\n"));
    event.push_str(&format!("SUMMARY:{summary}\r\n"));
    event.push_str(&format!("DESCRIPTION:{description}\r\n"));
    if rng.below(2) == 0 {
        event.push_str(&format!("LOCATION:Room {}\r\n", 1 + rng.below(9)));
    }
    if rng.below(4) == 0 {
        let organizer = rng.pick(&PEOPLE);
        event.push_str(&format!(
            "ORGANIZER;CN={organizer}:mailto:{organizer}@example.org\r\n"
        ));
        for _ in 0..1 + rng.below(4) {
            let attendee = rng.pick(&PEOPLE);
            event.push_str(&format!(
                "ATTENDEE;CN={attendee};PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:{attendee}@example.org\r\n"
            ));
        }
    }
    let mut overrides = String::new();
    if let Some(rule) = &rule {
        event.push_str(&format!("RRULE:{rule}\r\n"));
        let timed = !dtstart.contains("VALUE=DATE");
        let start_value = dtstart.rsplit(':').next().unwrap().to_owned();
        let prefix = dtstart.split(':').next().unwrap().replacen("DTSTART", "", 1);
        if timed && rng.below(10) < 3 {
            let at = &start_value[9..];
            let start_days = {
                let y: i64 = start_value[0..4].parse().unwrap();
                let m: i64 = start_value[4..6].parse().unwrap();
                let d: i64 = start_value[6..8].parse().unwrap();
                days_from_civil(y, m, d)
            };
            let step = if rule.contains("DAILY") { 1 } else { 7 };
            if !rule.contains("MONTHLY") {
                for k in 0..1 + rng.below(3) as i64 {
                    let excluded = start_days + step * (2 + 3 * k);
                    event.push_str(&format!("EXDATE{prefix}:{}T{at}\r\n", date(excluded)));
                }
                if rng.below(2) == 0 {
                    let moved = start_days + step * 4;
                    let instance = format!("{}T{at}", date(moved));
                    let new_start = format!("{}T{at}", date(moved + 1));
                    overrides.push_str("BEGIN:VEVENT\r\n");
                    overrides.push_str(&format!("UID:{uid}\r\n"));
                    overrides.push_str("DTSTAMP:20261101T120000Z\r\n");
                    overrides.push_str(&format!("RECURRENCE-ID{prefix}:{instance}\r\n"));
                    overrides.push_str(&format!("DTSTART{prefix}:{new_start}\r\n"));
                    overrides.push_str(&format!("SUMMARY:{summary} (moved)\r\n"));
                    overrides.push_str("END:VEVENT\r\n");
                }
            }
        }
    }
    if rng.below(10) == 0 {
        event.push_str("BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT15M\r\nDESCRIPTION:Reminder\r\nEND:VALARM\r\n");
    }
    event.push_str("END:VEVENT\r\n");
    let _ = zone;

    format!(
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//pimalaya//calendula bench//EN\r\n{zones}{event}{overrides}END:VCALENDAR\r\n"
    )
}

fn land(store: &PimdirStore, calendar: &str, events: usize) {
    let id = format!("bench/{calendar}");
    store.ensure_collection(&id, "text/calendar").unwrap();
    let mut source = PimdirStore::open(store_dir())
        .unwrap()
        .for_account("bench")
        .for_source("bench-caldav");
    let collection = PimdirCollectionId(id.clone());
    source
        .write(vec![PimdirWriteOp::OpenRound {
            collection: collection.clone(),
            scope: PimdirScope::unbounded(),
            band: false,
        }])
        .unwrap();

    let mut rng = Rng(0x00c0_ffee ^ (calendar.len() as u64));
    let blobs = io_pimdir::client::blobs::PimdirBlobs::open(store_dir());
    let mut ops = Vec::new();
    let mut handles = Vec::new();
    for i in 0..events {
        let text = body(&mut rng, calendar, i);
        let bytes = text.into_bytes();
        let hash = blobs.hash(&bytes);
        let derived = derive(&bytes);
        let handle = PimdirHandle(format!("e{i}.ics"));
        let known = PimdirFlags::Known(BTreeSet::new());
        ops.push(PimdirWriteOp::StoreObject {
            object: PimdirObject {
                hash: hash.clone(),
                size: bytes.len(),
            },
            body: Some(bytes),
        });
        ops.push(PimdirWriteOp::UpsertPlacement(PimdirPlacement {
            collection: collection.clone(),
            handle: handle.clone(),
            link_id: Some(derived.link_id),
            object: Some(hash.clone()),
            level: PimdirLevel::Full,
            summary: derived.summary,
            sort_key: derived.sort_key,
            flags: known.clone(),
            status: PimdirStatus::Clean,
            conflict_revision: None,
            conflict_object: None,
            base: Some(PimdirBase {
                flags: known,
                revision: Some(String::from("1")),
                object: Some(hash),
            }),
            origin: None,
            #[cfg(feature = "v20")]
            server_object_id: None,
        }));
        handles.push(handle);
        if handles.len() == 1000 || i + 1 == events {
            ops.push(PimdirWriteOp::Stamp {
                collection: collection.clone(),
                handles: std::mem::take(&mut handles),
            });
            source.write(std::mem::take(&mut ops)).unwrap();
        }
    }
    source
        .write(vec![PimdirWriteOp::CloseRound {
            collection,
            coverage: PimdirScope::unbounded(),
            checkpoint: Some(PimdirCheckpoint(b"1".to_vec())),
        }])
        .unwrap();
}

fn store_dir() -> PathBuf {
    PathBuf::from(env::args().nth(1).expect("a store directory"))
}

fn main() {
    let events: usize = env::args()
        .nth(2)
        .expect("a number of events")
        .parse()
        .unwrap();
    let index = !env::args().any(|arg| arg == "--no-index");
    let dir = store_dir();
    std::fs::create_dir_all(&dir).unwrap();

    let started = Instant::now();
    {
        let store = PimdirStore::open(&dir).unwrap().for_account("bench");
        drop(store);
    }
    let store = PimdirStore::open(&dir).unwrap().for_account("bench");
    drop(store);
    {
        let store = PimdirStore::open(&dir).unwrap().for_account("bench");
        store
            .ensure_collection("bench/calendar", "text/calendar")
            .unwrap();
        store
            .ensure_collection("bench/holidays", "text/calendar")
            .unwrap();
        store
            .set_collection_role("bench/calendar", Some("default"))
            .unwrap();
        land(&store, "calendar", events);
        land(&store, "holidays", 200);
    }
    eprintln!("landed {events} events in {:.1?}", started.elapsed());

    if index {
        let started = Instant::now();
        let mut index = PimdirIndex::open(&dir).unwrap();
        let report = index
            .refresh(&PimdirIndexRefreshOptions {
                floating_zone: String::from("Europe/Paris"),
                ..Default::default()
            })
            .unwrap();
        eprintln!("indexed in {:.1?}: {report:?}", started.elapsed());
    }
}
