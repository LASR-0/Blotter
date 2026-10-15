//! The daily briefing (roadmap §4.6, D28): on request, or each morning
//! when turned on (off by default; 07:30), the AI summarises the day and
//! proposes slots for Inbox items. The app runs it as a chat turn; this
//! module holds what it asks and when the morning one is due.

use chrono::{NaiveDate, NaiveTime};
use chrono_tz::Tz;
use rusqlite::Connection;

use crate::error::Result;
use crate::settings::{KEY_BRIEFING_ENABLED, KEY_BRIEFING_TIME, KEY_LAST_BRIEFING};
use crate::store;
use crate::time::{self, Instant};

/// The default morning time (D28).
pub const DEFAULT_TIME: &str = "07:30";

/// What the AI is asked for a briefing.
pub const PROMPT: &str = "Give me my daily briefing. In a few short lines: what today holds \
(fixed commitments first, with times), any conflicts or overdue tasks, and how much free time \
is left within planning hours. Then find slots for the Inbox tasks that fit today or the next \
few days (auto_slot or schedule_item), respecting block-outs and due dates. Keep it brief.";

/// Whether the morning briefing is on, and when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BriefingSettings {
    pub enabled: bool,
    pub time: NaiveTime,
}

pub fn load(conn: &Connection) -> Result<BriefingSettings> {
    let default = NaiveTime::from_hms_opt(7, 30, 0).unwrap_or(NaiveTime::MIN);
    Ok(BriefingSettings {
        enabled: store::settings::get(conn, KEY_BRIEFING_ENABLED)?.unwrap_or(false),
        time: store::settings::get::<String>(conn, KEY_BRIEFING_TIME)?
            .and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M").ok())
            .unwrap_or(default),
    })
}

/// The local date a morning briefing is due for: it's on, its time has
/// passed today, and today's hasn't run.
pub fn due(conn: &Connection, tz: Tz, now: Instant) -> Result<Option<NaiveDate>> {
    let settings = load(conn)?;
    if !settings.enabled {
        return Ok(None);
    }
    let local = now.with_timezone(&tz);
    let today = local.date_naive();
    let last = store::settings::get::<String>(conn, KEY_LAST_BRIEFING)?
        .and_then(|s| s.parse::<NaiveDate>().ok());
    if last.is_some_and(|d| d >= today) || local.time() < settings.time {
        return Ok(None);
    }
    Ok(Some(today))
}

/// Records that `date`'s morning briefing ran (before it runs, so a
/// failure can't make it repeat every minute).
pub fn mark_done(conn: &Connection, date: NaiveDate, now: Instant) -> Result<()> {
    store::settings::set(conn, KEY_LAST_BRIEFING, &time::fmt_date(date), now)
}

/// The briefing thread's title, e.g. "Briefing · Thu 15 Oct".
pub fn title(date: NaiveDate) -> String {
    format!("Briefing · {}", date.format("%a %-d %b"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use chrono::{TimeZone, Utc};

    /// 2026-10-15 at `h:m` in Melbourne (UTC+11).
    fn at(h: u32, m: u32) -> Instant {
        let local = chrono_tz::Australia::Melbourne
            .with_ymd_and_hms(2026, 10, 15, h, m, 0)
            .unwrap();
        local.with_timezone(&Utc)
    }

    #[test]
    fn due_once_a_day_after_its_time_when_on() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let tz = chrono_tz::Australia::Melbourne;
        assert_eq!(due(c, tz, at(9, 0)).unwrap(), None, "off by default");

        store::settings::set(c, KEY_BRIEFING_ENABLED, &true, at(6, 0)).unwrap();
        assert_eq!(due(c, tz, at(7, 29)).unwrap(), None, "before 07:30");
        let today = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap();
        assert_eq!(due(c, tz, at(7, 30)).unwrap(), Some(today));

        mark_done(c, today, at(7, 30)).unwrap();
        assert_eq!(due(c, tz, at(12, 0)).unwrap(), None, "once a day");

        store::settings::set(c, KEY_BRIEFING_TIME, &"06:00", at(6, 0)).unwrap();
        assert_eq!(
            load(c).unwrap().time,
            NaiveTime::from_hms_opt(6, 0, 0).unwrap()
        );
    }

    #[test]
    fn titles_name_the_day() {
        assert_eq!(
            title(NaiveDate::from_ymd_opt(2026, 10, 15).unwrap()),
            "Briefing · Thu 15 Oct"
        );
    }
}
