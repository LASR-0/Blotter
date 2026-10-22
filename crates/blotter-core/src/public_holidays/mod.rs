//! Public holidays from Nager.Date (https://date.nager.at): which country
//! and region the user is likely in, read from their time zone, and the
//! holidays that apply there out of the API's answer. Fetching is the caller's
//! job (the app, or Blotter Sync); this is pure but for [`Upkeep::apply`].

mod zones;

use std::collections::BTreeMap;

use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// The API's base URL.
pub const API: &str = "https://date.nager.at/api/v3";

/// Zones that tell the state or province too (ISO 3166-2, as Nager.Date
/// names them). Elsewhere the user picks a region, if their country has any.
const ZONE_REGIONS: &[(&str, &str)] = &[
    ("America/Edmonton", "CA-AB"),
    ("America/Halifax", "CA-NS"),
    ("America/Moncton", "CA-NB"),
    ("America/Regina", "CA-SK"),
    ("America/St_Johns", "CA-NL"),
    ("America/Toronto", "CA-ON"),
    ("America/Vancouver", "CA-BC"),
    ("America/Whitehorse", "CA-YT"),
    ("America/Winnipeg", "CA-MB"),
    ("Australia/ACT", "AU-ACT"),
    ("Australia/Adelaide", "AU-SA"),
    ("Australia/Brisbane", "AU-QLD"),
    ("Australia/Canberra", "AU-ACT"),
    ("Australia/Darwin", "AU-NT"),
    ("Australia/Hobart", "AU-TAS"),
    ("Australia/Lindeman", "AU-QLD"),
    ("Australia/Melbourne", "AU-VIC"),
    ("Australia/NSW", "AU-NSW"),
    ("Australia/North", "AU-NT"),
    ("Australia/Perth", "AU-WA"),
    ("Australia/Queensland", "AU-QLD"),
    ("Australia/South", "AU-SA"),
    ("Australia/Sydney", "AU-NSW"),
    ("Australia/Tasmania", "AU-TAS"),
    ("Australia/Victoria", "AU-VIC"),
    ("Australia/West", "AU-WA"),
    ("Canada/Atlantic", "CA-NS"),
    ("Canada/Mountain", "CA-AB"),
    ("Canada/Newfoundland", "CA-NL"),
    ("Canada/Pacific", "CA-BC"),
    ("Canada/Saskatchewan", "CA-SK"),
    ("Canada/Yukon", "CA-YT"),
];

/// Where the user probably is, from their time zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Guess {
    /// ISO 3166-1 alpha-2, e.g. `AU`.
    pub country: String,
    /// ISO 3166-2, e.g. `AU-VIC`, when the zone tells it.
    pub region: Option<String>,
}

/// The country (and region, where it shows) of an IANA zone name.
pub fn guess(zone: &str) -> Option<Guess> {
    let find = |table: &[(&str, &'static str)]| {
        table
            .binary_search_by(|(z, _)| (*z).cmp(zone))
            .ok()
            .map(|i| table[i].1)
    };
    let country = find(zones::ZONE_COUNTRIES)?;
    Some(Guess {
        country: country.to_owned(),
        region: find(ZONE_REGIONS).map(str::to_owned),
    })
}

/// One holiday as Nager.Date sends it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    date: NaiveDate,
    local_name: String,
    #[serde(default)]
    global: bool,
    #[serde(default)]
    counties: Option<Vec<String>>,
    #[serde(default)]
    types: Vec<String>,
}

fn parse(json: &str) -> Result<Vec<Entry>> {
    serde_json::from_str(json)
        .map_err(|e| CoreError::invalid(format!("unexpected public holiday data: {e}")))
}

/// A public holiday on one day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PublicHoliday {
    /// `YYYY-MM-DD`.
    #[schemars(with = "String")]
    pub date: NaiveDate,
    pub name: String,
}

/// The public holidays (not bank or school days) in a year's answer that
/// apply in `region`: the national ones, plus that region's own. Two on one
/// day become one holiday with both names.
pub fn select(json: &str, region: Option<&str>) -> Result<Vec<PublicHoliday>> {
    let mut days: BTreeMap<NaiveDate, Vec<String>> = BTreeMap::new();
    for e in parse(json)? {
        let public = e.types.is_empty() || e.types.iter().any(|t| t == "Public");
        let here = e.global
            || region.is_some_and(|r| {
                e.counties
                    .as_ref()
                    .is_some_and(|c| c.iter().any(|x| x == r))
            });
        if public && here {
            let names = days.entry(e.date).or_default();
            if !names.contains(&e.local_name) {
                names.push(e.local_name);
            }
        }
    }
    Ok(days
        .into_iter()
        .map(|(date, names)| PublicHoliday {
            date,
            name: names.join(" / "),
        })
        .collect())
}

/// The regions a year's answer names, sorted, for the region picker.
pub fn regions(json: &str) -> Result<Vec<String>> {
    let mut out: Vec<String> = parse(json)?
        .into_iter()
        .flat_map(|e| e.counties.unwrap_or_default())
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

/// Where the user's public holidays come from, kept (setting
/// `public_holidays`) so the app can fetch the next year's in time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub country: String,
    pub region: Option<String>,
    /// Fetch each next year's holidays automatically.
    pub keep_updated: bool,
    /// The last year imported.
    pub through_year: i32,
}

impl Config {
    /// The year the app should fetch now, if any: next year, once it isn't in.
    pub fn due(&self, today: NaiveDate) -> Option<i32> {
        let next = chrono::Datelike::year(&today) + 1;
        (self.keep_updated && self.through_year < next).then_some(next)
    }

    /// `public:AU-VIC`, or `public:DE` without a region.
    pub fn source(&self) -> String {
        format!(
            "{}{}",
            crate::model::PUBLIC_SOURCE,
            self.region.as_deref().unwrap_or(&self.country)
        )
    }
}

/// The address of a year's public holidays for `country`.
pub fn year_url(country: &str, year: i32) -> String {
    format!("{API}/PublicHolidays/{year}/{country}")
}

/// An automatic fetch of the next year's public holidays that is due. Each
/// replica that does upkeep (the app in Local only mode, Blotter Sync once
/// devices are paired) asks [`upkeep`], fetches [`Upkeep::url`] with its own
/// HTTP, and hands the answer to [`Upkeep::apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upkeep {
    pub country: String,
    pub region: Option<String>,
    pub year: i32,
}

/// The fetch that is due now, if the user keeps public holidays up to date
/// and the next year's aren't in yet.
pub fn upkeep(planner: &crate::Planner) -> Result<Option<Upkeep>> {
    let config: Option<Config> =
        crate::store::settings::get(planner.db().conn(), crate::settings::KEY_PUBLIC_HOLIDAYS)?;
    let Some(config) = config else {
        return Ok(None);
    };
    let today = planner
        .now()
        .with_timezone(&planner.settings()?.tz)
        .date_naive();
    Ok(config.due(today).map(|year| Upkeep {
        country: config.country,
        region: config.region,
        year,
    }))
}

impl Upkeep {
    pub fn url(&self) -> String {
        year_url(&self.country, self.year)
    }

    /// Imports the year's holidays from what [`Self::url`] answered, as an
    /// automatic (system) change.
    pub fn apply(&self, planner: &mut crate::Planner, json: &str) -> Result<crate::OpOutcome> {
        let holidays = select(json, self.region.as_deref())?;
        planner.execute(
            "import_public_holidays",
            serde_json::json!({
                "country": self.country,
                "region": self.region,
                "years": [self.year],
                "holidays": holidays,
            }),
            &crate::model::Actor::System,
        )
    }
}

/// A country Nager.Date covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Country {
    pub country_code: String,
    pub name: String,
}

/// The API's country list, sorted by name.
pub fn countries(json: &str) -> Result<Vec<Country>> {
    let mut out: Vec<Country> = serde_json::from_str(json)
        .map_err(|e| CoreError::invalid(format!("unexpected country list: {e}")))?;
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zones_tell_the_country_and_sometimes_the_region() {
        assert_eq!(
            guess("Australia/Melbourne"),
            Some(Guess {
                country: "AU".into(),
                region: Some("AU-VIC".into())
            })
        );
        assert_eq!(
            guess("Australia/Canberra").unwrap().region.as_deref(),
            Some("AU-ACT")
        );
        assert_eq!(
            guess("Europe/Berlin"),
            Some(Guess {
                country: "DE".into(),
                region: None
            })
        );
        assert_eq!(
            guess("Asia/Calcutta").unwrap().country,
            "IN",
            "old names too"
        );
        assert_eq!(guess("Etc/UTC"), None);
        assert!(
            zones::ZONE_COUNTRIES.windows(2).all(|w| w[0].0 < w[1].0)
                && ZONE_REGIONS.windows(2).all(|w| w[0].0 < w[1].0),
            "sorted for binary search"
        );
    }

    const AU_2026: &str = r#"[
      {"date":"2026-01-01","localName":"New Year's Day","name":"New Year's Day","countryCode":"AU","global":true,"counties":null,"types":["Public"]},
      {"date":"2026-03-02","localName":"Labour Day","name":"Labour Day","countryCode":"AU","global":false,"counties":["AU-WA"],"types":["Public"]},
      {"date":"2026-03-09","localName":"Labour Day","name":"Labour Day","countryCode":"AU","global":false,"counties":["AU-VIC"],"types":["Public"]},
      {"date":"2026-04-25","localName":"Anzac Day","name":"Anzac Day","countryCode":"AU","global":true,"counties":null,"types":["Public"]},
      {"date":"2026-11-03","localName":"Melbourne Cup","name":"Melbourne Cup","countryCode":"AU","global":false,"counties":["AU-VIC"],"types":["Public"]},
      {"date":"2026-12-24","localName":"Christmas Eve","name":"Christmas Eve","countryCode":"AU","global":true,"counties":null,"types":["Optional"]}
    ]"#;

    #[test]
    fn select_keeps_national_and_regional_public_holidays() {
        let vic = select(AU_2026, Some("AU-VIC")).unwrap();
        let names: Vec<&str> = vic.iter().map(|h| h.name.as_str()).collect();
        assert_eq!(
            names,
            ["New Year's Day", "Labour Day", "Anzac Day", "Melbourne Cup"]
        );
        assert_eq!(vic[1].date, NaiveDate::from_ymd_opt(2026, 3, 9).unwrap());

        let national = select(AU_2026, None).unwrap();
        assert_eq!(national.len(), 2);
        assert_eq!(regions(AU_2026).unwrap(), ["AU-VIC", "AU-WA"]);
        assert!(select("not json", None).is_err());
    }

    #[test]
    fn two_on_one_day_are_joined() {
        let json = r#"[
          {"date":"2027-03-26","localName":"Good Friday","global":true,"types":["Public"]},
          {"date":"2027-03-26","localName":"Purim","global":true,"types":["Public"]}
        ]"#;
        assert_eq!(select(json, None).unwrap()[0].name, "Good Friday / Purim");
    }

    #[test]
    fn countries_sorted_by_name() {
        let json = r#"[{"countryCode":"NZ","name":"New Zealand"},{"countryCode":"AU","name":"Australia"}]"#;
        let c = countries(json).unwrap();
        assert_eq!(c[0].country_code, "AU");
    }
}

#[cfg(test)]
mod upkeep_tests {
    use chrono::Datelike;
    use serde_json::json;

    use super::*;
    use crate::model::Actor;

    fn answer(year: i32) -> String {
        json!([
            { "date": format!("{year}-01-01"), "localName": "New Year's Day", "global": true, "types": ["Public"] },
            { "date": format!("{year}-11-02"), "localName": "Melbourne Cup", "global": false,
              "counties": ["AU-VIC"], "types": ["Public"] },
            { "date": format!("{year}-11-03"), "localName": "Elsewhere", "global": false,
              "counties": ["AU-NSW"], "types": ["Public"] }
        ])
        .to_string()
    }

    #[test]
    fn nothing_is_due_until_holidays_are_imported() {
        let p = crate::planner::tests::planner();
        assert_eq!(upkeep(&p).unwrap(), None);
    }

    #[test]
    fn next_years_fetch_is_due_then_applied_then_not() {
        let mut p = crate::planner::tests::planner();
        let year = p.now().with_timezone(&p.settings().unwrap().tz).year();
        let holidays = select(&answer(year), Some("AU-VIC")).unwrap();
        p.execute(
            "import_public_holidays",
            json!({ "country": "AU", "region": "AU-VIC", "years": [year], "holidays": holidays }),
            &Actor::User,
        )
        .unwrap();

        let due = upkeep(&p).unwrap().unwrap();
        assert_eq!(
            due,
            Upkeep {
                country: "AU".into(),
                region: Some("AU-VIC".into()),
                year: year + 1
            }
        );
        assert_eq!(due.url(), format!("{API}/PublicHolidays/{}/AU", year + 1));

        let out = due.apply(&mut p, &answer(year + 1)).unwrap();
        assert!(out.batch_id.is_some());
        assert!(out.changes.iter().all(|c| c.actor == Actor::System));
        let added = out.changes.iter().filter(|c| c.entity == "holiday").count();
        assert_eq!(added, 2, "New Year's Day and Melbourne Cup, not NSW's");
        assert_eq!(upkeep(&p).unwrap(), None, "next year is in");
    }

    #[test]
    fn upkeep_waits_when_turned_off() {
        let mut p = crate::planner::tests::planner();
        let year = p.now().with_timezone(&p.settings().unwrap().tz).year();
        p.execute(
            "import_public_holidays",
            json!({ "country": "AU", "region": "AU-VIC", "years": [year], "holidays": [],
                    "keep_updated": false }),
            &Actor::User,
        )
        .unwrap();
        assert_eq!(upkeep(&p).unwrap(), None);
    }
}
