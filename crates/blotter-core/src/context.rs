//! The system prompt for the AI (roadmap §4.6): who it is, the current
//! date, time and timezone, the user's block-outs and planning hours,
//! today's agenda and the Inbox count, and the scheduling rules in plain
//! language. Anything larger is fetched with the read tools on demand.

use std::fmt::Write as _;

use chrono::Weekday;
use chrono_tz::Tz;
use serde_json::{Value, json};

use crate::error::Result;
use crate::model::{Actor, Item};
use crate::planner::Planner;
use crate::store;
use crate::time::Instant;

const RULES: &str = "\
How Blotter works:
- Times in tool arguments are local (YYYY-MM-DDTHH:MM, or YYYY-MM-DD for a day) in the user's \
timezone. Tool results show instants in UTC with a Z; convert them before talking about times.
- Unscheduled tasks live in the Inbox. Scheduling puts them on the calendar; unscheduling sends \
them back.
- Items are flexible (tasks, by default) or fixed (events and appointments). A flexible item can \
never be placed inside a block-out, outside planning hours, or over other busy time. A fixed item \
goes exactly where it's put, even inside a block-out; overlapping fixed items are allowed but \
flagged as conflicts.
- Don't work out free time yourself: use find_free_slots to see options, or auto_slot to place \
Inbox tasks, which honours block-outs, planning hours, due dates and priority.
- Look things up (list_agenda, search_items, list_inbox) before changing them, and use the ids \
the tools return. Never invent ids.
- When a tool returns an error, read it, fix the arguments and try again, or tell the user what \
went wrong.

How to answer:
- Be brief and concrete: say what you changed or found, with local times like 14:30.
- Only change what the user asked for. If a request is ambiguous, ask first.";

/// Builds the system prompt from the planner's current state.
pub fn system_prompt(planner: &mut Planner) -> Result<String> {
    let settings = planner.settings()?;
    let tz = settings.tz;
    let now = planner.now();
    let local = now.with_timezone(&tz);
    let summary = planner
        .execute("get_day_summary", json!({}), &Actor::System)?
        .result;
    let items: Vec<Item> = serde_json::from_value(summary["items"].clone()).unwrap_or_default();
    let blockouts = store::blockouts::list(planner.db().conn())?;

    let mut out = String::from(
        "You are the planning assistant inside Blotter, the user's keyboard-first daily \
         planner. You read and change the planner only through the tools provided.\n\n",
    );
    let _ = writeln!(out, "Now: {} ({tz}).", local.format("%A %-d %B %Y, %H:%M"));

    out.push_str("\nPlanning hours (when flexible tasks may be placed):\n");
    for day in [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ] {
        let hours = match settings.planning_hours.for_day(day) {
            Some(span) => format!(
                "{}–{}",
                span.start.format("%H:%M"),
                span.end.format("%H:%M")
            ),
            None => "none".to_owned(),
        };
        let _ = writeln!(out, "- {day}: {hours}");
    }

    out.push_str("\nBlock-outs (flexible tasks never go here):\n");
    if blockouts.is_empty() {
        out.push_str("- none\n");
    }
    for b in &blockouts {
        let days = serde_json::to_value(b.weekdays)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .map(|d| {
                d.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        let fixed = if b.permits_fixed {
            ""
        } else {
            " (warns on fixed items too)"
        };
        let _ = writeln!(
            out,
            "- {}: {days} {}–{}{fixed}",
            b.label,
            b.start_time.format("%H:%M"),
            b.end_time.format("%H:%M"),
        );
    }

    let _ = writeln!(out, "\nToday ({}):", local.format("%a %-d %b"));
    if items.is_empty() {
        out.push_str("- nothing scheduled\n");
    }
    for item in &items {
        let _ = writeln!(out, "- {}", agenda_line(item, tz));
    }
    let count = |key: &str| summary[key].as_u64().unwrap_or(0);
    let _ = writeln!(
        out,
        "Inbox: {} unscheduled. Overdue: {}. Conflicts today: {}.",
        count("inbox_count"),
        count("overdue_count"),
        summary["conflicts"].as_array().map_or(0, Vec::len),
    );

    out.push('\n');
    out.push_str(RULES);
    Ok(out)
}

/// One agenda line: time, title and what matters about it, with its id.
fn agenda_line(item: &Item, tz: Tz) -> String {
    let local = |t: Instant| t.with_timezone(&tz).format("%H:%M").to_string();
    let when = match (item.all_day, item.start_at, item.end_at) {
        (true, _, _) => "all day".to_owned(),
        (_, Some(s), Some(e)) => format!("{}–{}", local(s), local(e)),
        (_, Some(s), None) => local(s),
        _ => "unscheduled".to_owned(),
    };
    let tag = |v: Value| v.as_str().unwrap_or_default().to_owned();
    let mut flags = vec![
        tag(serde_json::to_value(item.kind).unwrap_or_default()),
        tag(serde_json::to_value(item.anchoring).unwrap_or_default()),
    ];
    let status = tag(serde_json::to_value(item.status).unwrap_or_default());
    if status != "open" {
        flags.push(status);
    }
    if item.rrule.is_some() || item.parent_id.is_some() {
        flags.push("recurring".to_owned());
    }
    if let Some(due) = item.due_at {
        let d = due.with_timezone(&tz);
        flags.push(format!(
            "due {} {}",
            d.format("%a %-d %b"),
            d.format("%H:%M")
        ));
    }
    format!(
        "{when} {} [{}] (id {})",
        item.title,
        flags.join(", "),
        item.id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::tests::planner;

    #[test]
    fn prompt_carries_time_hours_blockouts_agenda_and_rules() {
        let mut p = planner();
        p.execute(
            "create_blockout",
            json!({ "label": "Work", "weekdays": ["mon","tue","wed","thu","fri"],
                    "start_time": "08:30", "end_time": "17:00" }),
            &Actor::User,
        )
        .unwrap();
        p.execute(
            "create_item",
            json!({ "title": "Dentist", "kind": "event", "start": "2026-10-14T15:00",
                    "duration_min": 45 }),
            &Actor::User,
        )
        .unwrap();
        p.execute("create_item", json!({ "title": "Buy milk" }), &Actor::User)
            .unwrap();

        let prompt = system_prompt(&mut p).unwrap();
        assert!(
            prompt.contains("Now: Wednesday 14 October 2026, 09:00 (Australia/Melbourne)"),
            "{prompt}"
        );
        assert!(prompt.contains("- Mon: 07:00–22:00"), "{prompt}");
        assert!(
            prompt.contains("- Work: mon tue wed thu fri 08:30–17:00"),
            "{prompt}"
        );
        assert!(
            prompt.contains("15:00–15:45 Dentist [event, fixed]"),
            "{prompt}"
        );
        assert!(prompt.contains("Inbox: 1 unscheduled."), "{prompt}");
        assert!(prompt.contains("find_free_slots"));
    }

    #[test]
    fn empty_planner_still_reads_well() {
        let mut p = planner();
        let prompt = system_prompt(&mut p).unwrap();
        assert!(prompt.contains("Block-outs (flexible tasks never go here):\n- none"));
        assert!(prompt.contains("- nothing scheduled"));
    }
}
