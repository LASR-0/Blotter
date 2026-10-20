//! Smart slot: placing tasks with the AI, using context the deterministic
//! auto-slot can't weigh — what a task is, where it happens, how a person's
//! day actually flows. The app runs it as a chat turn (like the daily
//! briefing); this module says what the AI is asked.
//!
//! The rules the core enforces still hold whatever the AI decides: flexible
//! tasks never land in block-outs or outside planning hours, and in Propose
//! mode nothing moves until the user accepts.

use std::fmt::Write as _;

use chrono::NaiveDate;
use chrono_tz::Tz;

use crate::model::Item;

/// How a smart-slot thread is titled, e.g. "Smart slot · Fri 16 Oct".
pub fn title(today: NaiveDate) -> String {
    format!("Smart slot · {}", today.format("%a %-d %b"))
}

/// The place in a task's notes (an imported LOCATION, or "Location: …"
/// written by hand), if it names one.
fn location(item: &Item) -> Option<&str> {
    item.notes
        .lines()
        .find_map(|l| l.trim().strip_prefix("Location:"))
        .map(str::trim)
        .filter(|l| !l.is_empty())
}

/// One line about a task for the prompt: id, title and what matters for
/// placing it.
fn describe(item: &Item, tz: Tz, default_min: u32) -> String {
    let mut line = format!("- {} \"{}\"", item.id, item.title);
    let minutes = item.duration_min.unwrap_or(default_min);
    let _ = write!(line, " · {minutes} min");
    if item.duration_min.is_none() {
        line.push_str(" (guessed)");
    }
    if !item.tags.is_empty() {
        let _ = write!(line, " · tags: {}", item.tags.join(", "));
    }
    if item.priority > 0 {
        let _ = write!(line, " · priority {}", item.priority);
    }
    if let Some(due) = item.due_at {
        let _ = write!(
            line,
            " · due {}",
            due.with_timezone(&tz).format("%a %-d %b %H:%M")
        );
    }
    if let Some(from) = item.not_before {
        let _ = write!(line, " · not before {}", from.format("%a %-d %b"));
    }
    if let (Some(s), Some(e)) = (item.window_start, item.window_end) {
        let _ = write!(
            line,
            " · usually {}–{}",
            s.format("%H:%M"),
            e.format("%H:%M")
        );
    }
    if let Some(place) = location(item) {
        let _ = write!(line, " · at {place}");
    }
    let notes: String = item
        .notes
        .lines()
        .filter(|l| !l.trim_start().starts_with("Location:"))
        .collect::<Vec<_>>()
        .join(" ");
    let notes = notes.trim();
    if !notes.is_empty() {
        // The user's own context (quick add's "// …"), so it gets room.
        let short: String = notes.chars().take(300).collect();
        let _ = write!(line, " · context: {short}");
    }
    line
}

/// What the AI is asked to place `tasks` (open, unscheduled or to be
/// rescheduled) from `today`.
pub fn prompt(tasks: &[Item], tz: Tz, today: NaiveDate, default_min: u32) -> String {
    let mut p = String::from(
        "Smart slot these tasks: find each one a good time, the way a thoughtful person \
would plan their own week rather than packing tasks back to back.\n\nTasks:\n",
    );
    for t in tasks {
        p.push_str(&describe(t, tz, default_min));
        p.push('\n');
    }
    let _ = write!(
        p,
        "\nToday is {}. First read the calendar for the next two weeks (list_agenda, \
list_blockout_windows, list_calendar_events, list_holidays) so you know what's already \
there.\n\n\
How to place them:\n\
- Fit the kind of task to the time. Personal, home, social and errand tasks suit weekends \
and evenings; work tasks suit working days, around the work block-out if there is one. \
Health and exercise suit mornings or early evenings. Use the tags, the title and the context \
to tell.\n\
- A task's context is the user's own note on it (when it can happen, what it needs, who it \
involves). Follow it over these general rules.\n\
- Holidays are days away from the usual week: never put a work task on one; tasks tagged \
holiday get first pick of them, and other personal tasks may go there too.\n\
- Leave breathing room: at least 15 minutes between items, more after long or demanding \
ones. Don't stack a day solid; spread tasks across days instead of filling one.\n\
- Allow travel time. When a task happens somewhere else (a location, or an errand like \
shopping, a pickup or an appointment), leave time to get there and back around it, and \
group errands in the same area or trip together.\n\
- Respect due dates and priority: anything due soon goes before it's due, with some slack; \
higher priority earlier.\n\
- Be realistic about energy: hard or focused work earlier in the day, light admin later.\n\
- Use find_free_slots to check a time is free (ask for the task's duration plus any travel \
time), then schedule_item for each task. Never put a flexible task inside a block-out; \
if a task can't fit in two weeks, leave it in the Inbox and say why.\n\n\
When you're done, reply with a short list: each task, when you put it, and a few words on \
why (e.g. \"Sat morning, with the other errands\").",
        today.format("%A %-d %B %Y")
    );
    p
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use chrono_tz::Australia::Melbourne;

    use super::*;
    use crate::model::ItemKind;

    fn task(title: &str) -> Item {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 0, 0, 0).unwrap();
        Item::new(format!("id-{title}"), ItemKind::Task, title, now)
    }

    #[test]
    fn prompt_lists_what_matters_for_placing() {
        let mut shop = task("Weekend shopping");
        shop.tags = vec!["shopping".into()];
        shop.notes = "Presents for Mia\n\nLocation: Chadstone".into();
        shop.duration_min = Some(90);
        let mut report = task("Quarterly report");
        report.tags = vec!["work".into()];
        report.priority = 3;
        report.due_at = Some(Utc.with_ymd_and_hms(2026, 10, 20, 6, 0, 0).unwrap());
        let today = NaiveDate::from_ymd_opt(2026, 10, 16).unwrap();
        let p = prompt(&[shop, report], Melbourne, today, 30);

        assert!(p.contains(
            "- id-Weekend shopping \"Weekend shopping\" · 90 min · tags: shopping · at Chadstone · context: Presents for Mia"
        ));
        assert!(p.contains(
            "- id-Quarterly report \"Quarterly report\" · 30 min (guessed) · tags: work · priority 3 · due Tue 20 Oct 17:00"
        ));
        assert!(p.contains("Today is Friday 16 October 2026."));
        for rule in [
            "weekends",
            "at least 15 minutes",
            "travel time",
            "find_free_slots",
            "block-out",
        ] {
            assert!(p.contains(rule), "{rule}");
        }
        assert_eq!(title(today), "Smart slot · Fri 16 Oct");
    }
}
