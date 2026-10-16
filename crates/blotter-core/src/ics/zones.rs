//! Time zones in iCalendar files: reading TZIDs (IANA names, Outlook's
//! Windows names, prefixed paths) and writing VTIMEZONE definitions.

use chrono::{Datelike, Duration, NaiveDate, Offset, TimeZone, Utc, Weekday};
use chrono_tz::{OffsetName, Tz};

use super::text::Writer;
use crate::time::Instant;

/// Windows zone names (as Outlook and Exchange write them) to IANA, after
/// CLDR's windowsZones mapping (the "001" territory of each).
const WINDOWS_ZONES: &[(&str, &str)] = &[
    ("AUS Eastern Standard Time", "Australia/Sydney"),
    ("E. Australia Standard Time", "Australia/Brisbane"),
    ("Cen. Australia Standard Time", "Australia/Adelaide"),
    ("AUS Central Standard Time", "Australia/Darwin"),
    ("Aus Central W. Standard Time", "Australia/Eucla"),
    ("W. Australia Standard Time", "Australia/Perth"),
    ("Tasmania Standard Time", "Australia/Hobart"),
    ("Lord Howe Standard Time", "Australia/Lord_Howe"),
    ("Norfolk Standard Time", "Pacific/Norfolk"),
    ("New Zealand Standard Time", "Pacific/Auckland"),
    ("Chatham Islands Standard Time", "Pacific/Chatham"),
    ("Fiji Standard Time", "Pacific/Fiji"),
    ("Tonga Standard Time", "Pacific/Tongatapu"),
    ("Samoa Standard Time", "Pacific/Apia"),
    ("Line Islands Standard Time", "Pacific/Kiritimati"),
    ("West Pacific Standard Time", "Pacific/Port_Moresby"),
    ("Central Pacific Standard Time", "Pacific/Guadalcanal"),
    ("Hawaiian Standard Time", "Pacific/Honolulu"),
    ("Alaskan Standard Time", "America/Anchorage"),
    ("Pacific Standard Time", "America/Los_Angeles"),
    ("Pacific Standard Time (Mexico)", "America/Tijuana"),
    ("US Mountain Standard Time", "America/Phoenix"),
    ("Mountain Standard Time", "America/Denver"),
    ("Mountain Standard Time (Mexico)", "America/Mazatlan"),
    ("Central Standard Time", "America/Chicago"),
    ("Central Standard Time (Mexico)", "America/Mexico_City"),
    ("Canada Central Standard Time", "America/Regina"),
    ("Central America Standard Time", "America/Guatemala"),
    ("Eastern Standard Time", "America/New_York"),
    ("Eastern Standard Time (Mexico)", "America/Cancun"),
    ("US Eastern Standard Time", "America/Indiana/Indianapolis"),
    ("Cuba Standard Time", "America/Havana"),
    ("SA Pacific Standard Time", "America/Bogota"),
    ("Venezuela Standard Time", "America/Caracas"),
    ("Atlantic Standard Time", "America/Halifax"),
    ("SA Western Standard Time", "America/La_Paz"),
    ("Pacific SA Standard Time", "America/Santiago"),
    ("Paraguay Standard Time", "America/Asuncion"),
    ("Central Brazilian Standard Time", "America/Cuiaba"),
    ("Newfoundland Standard Time", "America/St_Johns"),
    ("E. South America Standard Time", "America/Sao_Paulo"),
    ("Argentina Standard Time", "America/Argentina/Buenos_Aires"),
    ("SA Eastern Standard Time", "America/Cayenne"),
    ("Montevideo Standard Time", "America/Montevideo"),
    ("Greenland Standard Time", "America/Godthab"),
    ("Azores Standard Time", "Atlantic/Azores"),
    ("Cape Verde Standard Time", "Atlantic/Cape_Verde"),
    ("UTC", "UTC"),
    ("Coordinated Universal Time", "UTC"),
    ("GMT Standard Time", "Europe/London"),
    ("Greenwich Standard Time", "Atlantic/Reykjavik"),
    ("Morocco Standard Time", "Africa/Casablanca"),
    ("W. Europe Standard Time", "Europe/Berlin"),
    ("Central Europe Standard Time", "Europe/Budapest"),
    ("Romance Standard Time", "Europe/Paris"),
    ("Central European Standard Time", "Europe/Warsaw"),
    ("W. Central Africa Standard Time", "Africa/Lagos"),
    ("Namibia Standard Time", "Africa/Windhoek"),
    ("GTB Standard Time", "Europe/Bucharest"),
    ("E. Europe Standard Time", "Europe/Chisinau"),
    ("FLE Standard Time", "Europe/Kiev"),
    ("Kaliningrad Standard Time", "Europe/Kaliningrad"),
    ("Egypt Standard Time", "Africa/Cairo"),
    ("Libya Standard Time", "Africa/Tripoli"),
    ("South Africa Standard Time", "Africa/Johannesburg"),
    ("Israel Standard Time", "Asia/Jerusalem"),
    ("Jordan Standard Time", "Asia/Amman"),
    ("Middle East Standard Time", "Asia/Beirut"),
    ("Syria Standard Time", "Asia/Damascus"),
    ("Turkey Standard Time", "Europe/Istanbul"),
    ("Belarus Standard Time", "Europe/Minsk"),
    ("Russian Standard Time", "Europe/Moscow"),
    ("Arabic Standard Time", "Asia/Baghdad"),
    ("Arab Standard Time", "Asia/Riyadh"),
    ("E. Africa Standard Time", "Africa/Nairobi"),
    ("Iran Standard Time", "Asia/Tehran"),
    ("Arabian Standard Time", "Asia/Dubai"),
    ("Azerbaijan Standard Time", "Asia/Baku"),
    ("Georgian Standard Time", "Asia/Tbilisi"),
    ("Caucasus Standard Time", "Asia/Yerevan"),
    ("Mauritius Standard Time", "Indian/Mauritius"),
    ("Afghanistan Standard Time", "Asia/Kabul"),
    ("West Asia Standard Time", "Asia/Tashkent"),
    ("Ekaterinburg Standard Time", "Asia/Yekaterinburg"),
    ("Pakistan Standard Time", "Asia/Karachi"),
    ("India Standard Time", "Asia/Kolkata"),
    ("Sri Lanka Standard Time", "Asia/Colombo"),
    ("Nepal Standard Time", "Asia/Kathmandu"),
    ("Central Asia Standard Time", "Asia/Almaty"),
    ("Bangladesh Standard Time", "Asia/Dhaka"),
    ("Myanmar Standard Time", "Asia/Yangon"),
    ("SE Asia Standard Time", "Asia/Bangkok"),
    ("N. Central Asia Standard Time", "Asia/Novosibirsk"),
    ("North Asia Standard Time", "Asia/Krasnoyarsk"),
    ("China Standard Time", "Asia/Shanghai"),
    ("North Asia East Standard Time", "Asia/Irkutsk"),
    ("Singapore Standard Time", "Asia/Singapore"),
    ("Taipei Standard Time", "Asia/Taipei"),
    ("Ulaanbaatar Standard Time", "Asia/Ulaanbaatar"),
    ("Tokyo Standard Time", "Asia/Tokyo"),
    ("Korea Standard Time", "Asia/Seoul"),
    ("Yakutsk Standard Time", "Asia/Yakutsk"),
    ("Vladivostok Standard Time", "Asia/Vladivostok"),
    ("Magadan Standard Time", "Asia/Magadan"),
    ("Dateline Standard Time", "Etc/GMT+12"),
    ("UTC-11", "Etc/GMT+11"),
    ("UTC-02", "Etc/GMT+2"),
    ("UTC+12", "Etc/GMT-12"),
];

/// The zone a TZID names: an IANA name (also after a path prefix such as
/// `/freeassociation.sourceforge.net/Tzfile/Europe/London`), or a Windows
/// name as Outlook writes it.
pub fn resolve_tzid(tzid: &str) -> Option<Tz> {
    let tzid = tzid.trim().trim_matches('"');
    if let Ok(tz) = tzid.parse::<Tz>() {
        return Some(tz);
    }
    if let Some((_, iana)) = WINDOWS_ZONES
        .iter()
        .find(|(w, _)| w.eq_ignore_ascii_case(tzid))
    {
        return iana.parse().ok();
    }
    // Try each suffix after a slash, longest first.
    tzid.match_indices('/')
        .find_map(|(i, _)| tzid[i + 1..].parse::<Tz>().ok())
}

fn offset_secs(tz: Tz, t: Instant) -> i32 {
    tz.offset_from_utc_datetime(&t.naive_utc())
        .fix()
        .local_minus_utc()
}

/// `+1100`, `-0330`.
fn fmt_offset(secs: i32) -> String {
    let sign = if secs < 0 { '-' } else { '+' };
    let m = secs.abs() / 60;
    format!("{sign}{:02}{:02}", m / 60, m % 60)
}

/// The zone's offset changes during `year`: (instant, offset before, offset after).
fn transitions(tz: Tz, year: i32) -> Vec<(Instant, i32, i32)> {
    let Some(start) = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).single() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut day = start;
    while day.year() == year {
        let next = day + Duration::days(1);
        let (a, b) = (offset_secs(tz, day), offset_secs(tz, next));
        if a != b {
            // The change happened in (day, next]; narrow it to the second.
            let (mut lo, mut hi) = (day, next);
            while hi - lo > Duration::seconds(1) {
                let mid = lo + (hi - lo) / 2;
                if offset_secs(tz, mid) == a {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            out.push((hi, a, b));
        }
        day = next;
    }
    out
}

/// Which `weekday` of its month `date` is: 1–4, or -1 for the last.
fn nth_weekday(date: NaiveDate) -> i32 {
    let next_month_same = date + Duration::days(7);
    if next_month_same.month() != date.month() {
        -1
    } else {
        (date.day0() / 7 + 1) as i32
    }
}

/// The `n`th (or last, for -1) `weekday` of a month.
fn weekday_in_month(year: i32, month: u32, weekday: Weekday, n: i32) -> Option<NaiveDate> {
    if n > 0 {
        let first = NaiveDate::from_ymd_opt(year, month, 1)?;
        let ahead = (7 + weekday.num_days_from_monday() as i64
            - first.weekday().num_days_from_monday() as i64)
            % 7;
        Some(first + Duration::days(ahead + 7 * (i64::from(n) - 1)))
    } else {
        let (ny, nm) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let last = NaiveDate::from_ymd_opt(ny, nm, 1)?.pred_opt()?;
        let back = (7 + last.weekday().num_days_from_monday() as i64
            - weekday.num_days_from_monday() as i64)
            % 7;
        Some(last - Duration::days(back))
    }
}

const WEEKDAYS: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];

/// Writes a VTIMEZONE for `tz`. Its rules are the zone's current ones
/// (taken from `year`), as yearly rules starting in 1970, which is how
/// most calendar programs write them. Clients that know the IANA name use
/// their own rules anyway; this is for the ones that don't.
pub fn write_vtimezone(w: &mut Writer, tz: Tz, year: i32) {
    w.begin("VTIMEZONE");
    w.line("TZID", tz.name());
    let changes = transitions(tz, year);
    if changes.is_empty() {
        let Some(mid) = Utc.with_ymd_and_hms(year, 6, 1, 0, 0, 0).single() else {
            w.end("VTIMEZONE");
            return;
        };
        let off = offset_secs(tz, mid);
        w.begin("STANDARD");
        w.line("DTSTART", "19700101T000000");
        w.line("TZOFFSETFROM", &fmt_offset(off));
        w.line("TZOFFSETTO", &fmt_offset(off));
        if let Some(name) = tz.offset_from_utc_datetime(&mid.naive_utc()).abbreviation() {
            w.text("TZNAME", name);
        }
        w.end("STANDARD");
    } else {
        for (at, from, to) in changes {
            let kind = if to > from { "DAYLIGHT" } else { "STANDARD" };
            // DTSTART is the wall-clock moment of the change, in the old offset.
            let local = at.naive_utc() + Duration::seconds(i64::from(from));
            let n = nth_weekday(local.date());
            let wd = local.weekday();
            let onset = weekday_in_month(1970, local.month(), wd, n)
                .map_or(local, |d| d.and_time(local.time()));
            w.begin(kind);
            w.line("DTSTART", &onset.format("%Y%m%dT%H%M%S").to_string());
            w.line("TZOFFSETFROM", &fmt_offset(from));
            w.line("TZOFFSETTO", &fmt_offset(to));
            w.line(
                "RRULE",
                &format!(
                    "FREQ=YEARLY;BYMONTH={};BYDAY={n}{}",
                    local.month(),
                    WEEKDAYS[wd.num_days_from_monday() as usize]
                ),
            );
            if let Some(name) = tz.offset_from_utc_datetime(&at.naive_utc()).abbreviation() {
                w.text("TZNAME", name);
            }
            w.end(kind);
        }
    }
    w.end("VTIMEZONE");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_windows_zone_maps_to_a_known_iana_zone() {
        for (win, iana) in WINDOWS_ZONES {
            assert!(iana.parse::<Tz>().is_ok(), "{win} → {iana}");
        }
        assert_eq!(
            resolve_tzid("AUS Eastern Standard Time"),
            Some(chrono_tz::Australia::Sydney)
        );
        assert_eq!(
            resolve_tzid("\"Europe/London\""),
            Some(chrono_tz::Europe::London)
        );
        assert_eq!(
            resolve_tzid("/freeassociation.sourceforge.net/Tzfile/Europe/London"),
            Some(chrono_tz::Europe::London)
        );
        assert_eq!(resolve_tzid("Customized Time Zone"), None);
    }

    #[test]
    fn melbourne_vtimezone_has_both_rules() {
        let mut w = Writer::default();
        write_vtimezone(&mut w, chrono_tz::Australia::Melbourne, 2026);
        let out = w.finish();
        // Summer time ends on the first Sunday in April at 03:00 and starts
        // on the first Sunday in October at 02:00.
        assert!(out.contains("BEGIN:STANDARD\r\nDTSTART:19700405T030000\r\nTZOFFSETFROM:+1100\r\nTZOFFSETTO:+1000\r\nRRULE:FREQ=YEARLY;BYMONTH=4;BYDAY=1SU"), "{out}");
        assert!(out.contains("BEGIN:DAYLIGHT\r\nDTSTART:19701004T020000\r\nTZOFFSETFROM:+1000\r\nTZOFFSETTO:+1100\r\nRRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=1SU"), "{out}");
        assert!(out.contains("TZNAME:AEDT"));
    }

    #[test]
    fn last_sunday_rules_and_fixed_zones() {
        let mut w = Writer::default();
        write_vtimezone(&mut w, chrono_tz::Europe::London, 2026);
        let out = w.finish();
        assert!(
            out.contains("RRULE:FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"),
            "{out}"
        );
        assert!(
            out.contains("RRULE:FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"),
            "{out}"
        );

        let mut w = Writer::default();
        write_vtimezone(&mut w, chrono_tz::Australia::Brisbane, 2026);
        let out = w.finish();
        assert!(
            out.contains("TZOFFSETFROM:+1000\r\nTZOFFSETTO:+1000"),
            "{out}"
        );
        assert!(!out.contains("DAYLIGHT"));
    }
}
