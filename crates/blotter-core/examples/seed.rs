//! Fills a database with a demo week, through the real operation layer.
//!
//!     cargo run -p blotter-core --example seed -- <path/to/blotter.db>
//!
//! Dates are relative to the current week (Monday start) in the OS timezone,
//! so the demo always lands on screen. Tasks dated before today are marked
//! done, except "Renew car registration", which is left overdue. Run it against a scratch database (BLOTTER_DATA_DIR), not your
//! real one.

use std::path::PathBuf;

use blotter_core::{Actor, Database, Planner};
use chrono::{Datelike, Duration, Local};
use serde_json::{Value, json};

/// (day offset from Monday, kind, title, start HH:MM, minutes, tag)
const TIMED: &[(i64, &str, &str, &str, u32, &str)] = &[
    (0, "task", "Groceries", "17:30", 45, "shopping"),
    (0, "task", "Renew car registration", "19:30", 20, "home"),
    (1, "task", "Pay electricity bill", "07:45", 15, "home"),
    (1, "event", "Telehealth consult", "11:30", 45, "health"),
    (1, "task", "Gym", "17:30", 45, "health"),
    (1, "task", "Read", "20:30", 30, "personal"),
    (2, "event", "Yoga class", "17:30", 60, "health"),
    (2, "task", "Meal prep", "19:00", 60, "home"),
    (3, "event", "Dentist", "15:00", 45, "health"),
    (
        3,
        "event",
        "Parent–teacher interview",
        "15:30",
        20,
        "personal",
    ),
    (3, "task", "Pay credit card", "19:00", 15, "home"),
    (4, "event", "Team lunch", "12:00", 60, "work"),
    (4, "event", "Dinner with Jess", "18:00", 180, "personal"),
    (5, "task", "Farmers market", "09:00", 60, "shopping"),
    (5, "event", "Mia's party", "14:00", 120, "personal"),
    (6, "task", "Long run", "10:00", 90, "personal"),
    (6, "task", "Weekly review", "16:00", 30, "work"),
];

/// Left undone on a past day so the Diary shows an Overdue section.
const LEFT_OVERDUE: &str = "Renew car registration";

const INBOX: &[&str] = &[
    "Call plumber",
    "Book flu shot",
    "Fix the leaking tap",
    "Renew passport",
    "Weekend shopping for presents",
    "Review insurance renewal",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .ok_or("usage: seed <path/to/blotter.db>")?
        .into();
    let tz = iana_time_zone::get_timezone()?
        .parse()
        .unwrap_or(chrono_tz::UTC);
    let mut planner = Planner::new(Database::open(&path)?, tz);
    let mut run = |op: &str, args: Value| -> Result<Value, Box<dyn std::error::Error>> {
        Ok(planner.execute(op, args, &Actor::System)?.result)
    };

    let today = Local::now().date_naive();
    let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    let day = |d: i64| monday + Duration::days(d);

    let mut timed: Vec<(i64, &str, &str, &str, u32, &str)> = TIMED.to_vec();
    for d in 0..5 {
        timed.push((d, "task", "Morning run", "07:15", 30, "personal"));
        timed.push((d, "event", "Standup", "09:00", 15, "work"));
    }
    for (d, kind, title, hm, minutes, tag) in timed {
        let item = run(
            "create_item",
            json!({ "title": title, "kind": kind, "start": format!("{}T{hm}", day(d)),
                    "duration_min": minutes, "tags": [tag] }),
        )?;
        if kind == "task" && day(d) < today && title != LEFT_OVERDUE {
            run("complete_item", json!({ "id": item["id"] }))?;
        }
    }

    run(
        "create_item",
        json!({ "title": "Bin night", "kind": "event", "date": day(2).to_string(), "tags": ["home"] }),
    )?;
    run(
        "create_item",
        json!({ "title": "Mum visiting", "kind": "event", "date": day(5).to_string(),
                "end_date": day(7).to_string(), "tags": ["personal"] }),
    )?;
    for title in INBOX {
        run("create_item", json!({ "title": title, "duration_min": 30 }))?;
    }
    run(
        "create_blockout",
        json!({ "label": "Work", "weekdays": ["mon", "tue", "wed", "thu", "fri"],
                "start_time": "08:30", "end_time": "17:00", "icon": "briefcase" }),
    )?;
    run(
        "create_blockout",
        json!({ "label": "Sleep", "weekdays": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"],
                "start_time": "23:00", "end_time": "06:30", "permits_fixed": false }),
    )?;

    println!("Seeded the week of {monday} into {}", path.display());
    Ok(())
}
