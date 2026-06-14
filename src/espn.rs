//! Capa anticorrupción sobre la API no documentada de ESPN.
//!
//! Los structs `raw` son deliberadamente permisivos (todo `Option` /
//! `default`): el JSON de ESPN cambia de forma según el estado del partido.
//! El resto de la app solo conoce los tipos garantizados de `crate::model`.

use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Days, Local, NaiveDate, NaiveDateTime, Utc};

use crate::model::{CardColor, KeyEvent, KeyEventKind, Match, MatchStatus, Team};

const BASE: &str = "https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world";

/// Primer día con partidos de la Copa 2026 (fase de grupos).
pub fn tournament_start() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 6, 11).expect("inicio del torneo")
}

/// Día calendario local a cargar antes de `earliest_loaded`; `None` si ya en el límite.
pub fn previous_jornada_target(earliest_loaded: NaiveDate) -> Option<NaiveDate> {
    let target = earliest_loaded.checked_sub_days(Days::new(1))?;
    (target >= tournament_start()).then_some(target)
}

mod raw {
    use serde::Deserialize;

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Scoreboard {
        #[serde(default)]
        pub events: Vec<Event>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Event {
        #[serde(default)]
        pub id: String,
        #[serde(default)]
        pub date: String,
        #[serde(default)]
        pub competitions: Vec<Competition>,
        #[serde(default)]
        pub status: Status,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Competition {
        #[serde(default)]
        pub competitors: Vec<Competitor>,
        pub venue: Option<Venue>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Competitor {
        #[serde(default)]
        pub home_away: String,
        pub score: Option<String>,
        pub team: Option<TeamInfo>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct TeamInfo {
        #[serde(default)]
        pub display_name: String,
        #[serde(default)]
        pub abbreviation: String,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Venue {
        #[serde(default)]
        pub full_name: String,
        #[serde(default)]
        pub address: Address,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Address {
        #[serde(default)]
        pub city: String,
        #[serde(default)]
        pub country: String,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Status {
        pub display_clock: Option<String>,
        #[serde(default, rename = "type")]
        pub status_type: StatusType,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct StatusType {
        #[serde(default)]
        pub state: String,
        #[serde(default)]
        pub name: String,
        #[serde(default)]
        pub short_detail: String,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Summary {
        #[serde(default)]
        pub key_events: Vec<RawKeyEvent>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RawKeyEvent {
        #[serde(default, rename = "type")]
        pub event_type: EventType,
        #[serde(default)]
        pub text: String,
        #[serde(default)]
        pub short_text: String,
        #[serde(default)]
        pub clock: Clock,
        #[serde(default)]
        pub period: Period,
        pub team: Option<EventTeam>,
        #[serde(default)]
        pub participants: Vec<Participant>,
        #[serde(default)]
        pub scoring_play: bool,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EventType {
        #[serde(default)]
        pub text: String,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Clock {
        #[serde(default)]
        pub display_value: String,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Period {
        #[serde(default)]
        pub number: u32,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EventTeam {
        #[serde(default)]
        pub display_name: String,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Participant {
        pub athlete: Option<Athlete>,
    }

    #[derive(Debug, Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Athlete {
        #[serde(default)]
        pub display_name: String,
    }
}

pub struct Client {
    http: reqwest::Client,
}

impl Client {
    pub fn new() -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("world-cup-tui/0.1")
            .build()
            .context("no se pudo construir el cliente HTTP")?;
        Ok(Self { http })
    }

    pub async fn fetch_scoreboard(&self) -> Result<Vec<Match>> {
        // Rango amplio (ayer → mañana): el agrupado por fecha de ESPN no
        // coincide con el día local del usuario; se filtra del lado cliente.
        let today = Local::now().date_naive();
        let from = today.checked_sub_days(Days::new(1)).unwrap_or(today);
        let to = today.checked_add_days(Days::new(1)).unwrap_or(today);
        let body = self.fetch_scoreboard_body(from, to).await?;
        Ok(filter_relevant(parse_scoreboard(&body)?, today))
    }

    /// Partidos finalizados de un día calendario local (carga bajo demanda).
    pub async fn fetch_scoreboard_day(&self, day: NaiveDate) -> Result<Vec<Match>> {
        if day < tournament_start() {
            return Ok(vec![]);
        }
        let body = self.fetch_scoreboard_body(day, day).await?;
        Ok(filter_finished_on_day(&parse_scoreboard(&body)?, day))
    }

    async fn fetch_scoreboard_body(&self, from: NaiveDate, to: NaiveDate) -> Result<String> {
        let dates = format!("{}-{}", from.format("%Y%m%d"), to.format("%Y%m%d"));
        self.http
            .get(format!("{BASE}/scoreboard?dates={dates}"))
            .send()
            .await
            .context("fallo de red al pedir el scoreboard")?
            .error_for_status()
            .context("el scoreboard respondió con error HTTP")?
            .text()
            .await
            .context("no se pudo leer el cuerpo del scoreboard")
    }

    pub async fn fetch_summary(&self, event_id: &str) -> Result<Vec<KeyEvent>> {
        let body = self
            .http
            .get(format!("{BASE}/summary?event={event_id}"))
            .send()
            .await
            .context("fallo de red al pedir el summary")?
            .error_for_status()
            .context("el summary respondió con error HTTP")?
            .text()
            .await
            .context("no se pudo leer el cuerpo del summary")?;
        parse_summary(&body)
    }
}

pub fn parse_scoreboard(json: &str) -> Result<Vec<Match>> {
    let sb: raw::Scoreboard =
        serde_json::from_str(json).context("JSON de scoreboard con forma inesperada")?;
    Ok(sb.events.into_iter().filter_map(normalize_event).collect())
}

/// Partidos que entran en la ventana del poll periódico (hoy + finalizados ayer).
pub fn is_in_poll_window(m: &Match, today: NaiveDate) -> bool {
    match m.kickoff {
        None => true,
        Some(k) => {
            let date = k.with_timezone(&Local).date_naive();
            let yesterday = today.checked_sub_days(Days::new(1));
            date == today || (Some(date) == yesterday && m.status == MatchStatus::Finished)
        }
    }
}

/// Partidos relevantes para el dashboard: todos los de hoy (hora local) y los
/// FINALIZADOS de ayer. Sin fecha de inicio se conservan (no clasificables).
pub fn filter_relevant(matches: Vec<Match>, today: NaiveDate) -> Vec<Match> {
    matches
        .into_iter()
        .filter(|m| is_in_poll_window(m, today))
        .collect()
}

/// Solo finalizados cuya fecha de inicio local coincide con `day`.
pub fn filter_finished_on_day(matches: &[Match], day: NaiveDate) -> Vec<Match> {
    matches
        .iter()
        .filter(|m| {
            m.status == MatchStatus::Finished
                && m.kickoff
                    .is_some_and(|k| k.with_timezone(&Local).date_naive() == day)
        })
        .cloned()
        .collect()
}

pub fn parse_summary(json: &str) -> Result<Vec<KeyEvent>> {
    let s: raw::Summary =
        serde_json::from_str(json).context("JSON de summary con forma inesperada")?;
    Ok(s.key_events.into_iter().map(normalize_key_event).collect())
}

fn normalize_event(ev: raw::Event) -> Option<Match> {
    let comp = ev.competitions.into_iter().next()?;
    let mut home = None;
    let mut away = None;
    for c in comp.competitors {
        let team = normalize_team(&c);
        match c.home_away.as_str() {
            "home" => home = Some(team),
            "away" => away = Some(team),
            _ => {}
        }
    }
    let status = match ev.status.status_type.state.as_str() {
        "pre" => MatchStatus::Scheduled,
        "post" => MatchStatus::Finished,
        "in" if ev.status.status_type.name == "STATUS_HALFTIME" => MatchStatus::HalfTime,
        "in" => MatchStatus::Live,
        _ => MatchStatus::Scheduled,
    };
    let (venue, city) = comp
        .venue
        .map(|v| {
            let mut city = v.address.city;
            if !v.address.country.is_empty() {
                if city.is_empty() {
                    city = v.address.country;
                } else {
                    city = format!("{city}, {}", v.address.country);
                }
            }
            (Some(v.full_name), Some(city))
        })
        .unwrap_or((None, None));
    Some(Match {
        id: ev.id,
        kickoff: parse_kickoff(&ev.date),
        home: home?,
        away: away?,
        status,
        clock: ev.status.display_clock.filter(|c| !c.is_empty()),
        status_detail: ev.status.status_type.short_detail,
        venue: venue.filter(|v| !v.is_empty()),
        city: city.filter(|c| !c.is_empty()),
    })
}

fn normalize_team(c: &raw::Competitor) -> Team {
    let info = c.team.as_ref();
    Team {
        name: info.map(|t| t.display_name.clone()).unwrap_or_default(),
        abbrev: info.map(|t| t.abbreviation.clone()).unwrap_or_default(),
        score: c.score.as_deref().and_then(|s| s.parse().ok()),
    }
}

fn normalize_key_event(ev: raw::RawKeyEvent) -> KeyEvent {
    let type_text = ev.event_type.text.as_str();
    let kind = if ev.scoring_play {
        // "Goal - Header" / "Penalty - Scored" / "Own Goal" → conservar la variante completa.
        let detail = (type_text != "Goal").then(|| type_text.to_string());
        KeyEventKind::Goal { detail }
    } else {
        match type_text {
            "Yellow Card" => KeyEventKind::Card(CardColor::Yellow),
            "Red Card" => KeyEventKind::Card(CardColor::Red),
            "Substitution" => KeyEventKind::Substitution,
            _ => KeyEventKind::Other,
        }
    };
    let player = ev
        .participants
        .first()
        .and_then(|p| p.athlete.as_ref())
        .map(|a| a.display_name.clone())
        .filter(|n| !n.is_empty());
    let text = if ev.text.is_empty() {
        ev.short_text
    } else {
        ev.text
    };
    KeyEvent {
        minute: ev.clock.display_value,
        period: ev.period.number,
        kind,
        player,
        team: ev.team.map(|t| t.display_name).filter(|t| !t.is_empty()),
        text,
    }
}

/// ESPN usa fechas tipo "2026-06-11T19:00Z" (sin segundos, no es RFC 3339 estricto).
fn parse_kickoff(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%MZ")
        .ok()
        .map(|n| n.and_utc())
}
