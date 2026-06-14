use chrono::{DateTime, Days, Local, NaiveDate, TimeZone, Utc};
use world_cup_tui::espn::{filter_relevant, parse_scoreboard, parse_summary};
use world_cup_tui::model::{CardColor, KeyEventKind, Match, MatchStatus, Team};

const SCOREBOARD: &str = include_str!("fixtures/scoreboard.json");
const SUMMARY: &str = include_str!("fixtures/summary.json");

#[test]
fn scoreboard_fixture_normalizes() {
    let matches = parse_scoreboard(SCOREBOARD).unwrap();
    assert_eq!(matches.len(), 2);

    let mx = matches
        .iter()
        .find(|m| m.home.name == "Mexico")
        .expect("debe estar México vs Sudáfrica");
    assert_eq!(mx.home.score, Some(2));
    assert_eq!(mx.away.name, "South Africa");
    assert_eq!(mx.away.score, Some(0));
    assert_eq!(mx.status, MatchStatus::Finished);
    assert_eq!(mx.status_detail, "FT");
    assert_eq!(mx.clock.as_deref(), Some("90'+8'"));
    assert!(mx.venue.is_some());
    assert_eq!(mx.city.as_deref(), Some("Mexico City, Mexico"));
    assert!(mx.kickoff.is_some(), "la fecha sin segundos debe parsear");
    assert!(!mx.is_live());
}

#[test]
fn summary_fixture_normalizes_goals_and_cards() {
    let events = parse_summary(SUMMARY).unwrap();
    assert!(events.len() > 30, "el fixture tiene 39 keyEvents");

    let goals: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, KeyEventKind::Goal { .. }))
        .collect();
    assert_eq!(goals.len(), 2);
    assert_eq!(goals[0].player.as_deref(), Some("Julián Quiñones"));
    assert_eq!(goals[0].minute, "9'");
    assert_eq!(goals[0].team.as_deref(), Some("Mexico"));
    assert_eq!(goals[1].player.as_deref(), Some("Raúl Jiménez"));
    assert_eq!(goals[1].minute, "67'");
    assert!(
        matches!(&goals[1].kind, KeyEventKind::Goal { detail: Some(d) } if d == "Goal - Header"),
        "el segundo gol fue de cabeza"
    );

    let reds: Vec<_> = events
        .iter()
        .filter(|e| e.kind == KeyEventKind::Card(CardColor::Red))
        .collect();
    assert_eq!(reds.len(), 3, "el partido terminó con 3 expulsados");
    assert_eq!(reds[0].player.as_deref(), Some("Sphephelo Sithole"));
    assert_eq!(reds[0].minute, "49'");
    assert_eq!(reds[2].minute, "90'+2'");

    let yellows = events
        .iter()
        .filter(|e| e.kind == KeyEventKind::Card(CardColor::Yellow))
        .count();
    assert!(yellows >= 3, "hubo al menos 3 amarillas: {yellows}");
}

#[test]
fn unknown_event_types_become_other_and_are_kept() {
    let events = parse_summary(SUMMARY).unwrap();
    let kickoff = events
        .iter()
        .find(|e| e.kind == KeyEventKind::Other && e.text.contains("First Half ends"))
        .or_else(|| events.iter().find(|e| e.kind == KeyEventKind::Other));
    assert!(
        kickoff.is_some(),
        "tipos no mapeados se conservan como Other"
    );
}

#[test]
fn events_are_chronological() {
    // La cronología es (periodo, minuto): "45'+4'" del 1er tiempo precede
    // al "45'" con que arranca el 2do.
    let events = parse_summary(SUMMARY).unwrap();
    let minute = |s: &str| -> u32 {
        s.trim_end_matches('\'')
            .split("'+")
            .filter_map(|p| p.parse::<u32>().ok())
            .sum()
    };
    let keys: Vec<(u32, u32)> = events
        .iter()
        .filter(|e| !e.minute.is_empty() && e.period > 0)
        .map(|e| (e.period, minute(&e.minute)))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted);
}

/// Kickoff a mediodía en la zona local del test runner; alineado con cómo
/// `filter_relevant` clasifica fechas (`with_timezone(&Local)`).
fn kickoff_local(date: NaiveDate) -> DateTime<Utc> {
    Local
        .from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap())
        .single()
        .expect("fecha local válida")
        .with_timezone(&Utc)
}

fn sample_finished_match(kickoff: DateTime<Utc>) -> Match {
    Match {
        id: "test".into(),
        kickoff: Some(kickoff),
        home: Team {
            name: "A".into(),
            abbrev: "A".into(),
            score: Some(1),
        },
        away: Team {
            name: "B".into(),
            abbrev: "B".into(),
            score: Some(0),
        },
        status: MatchStatus::Finished,
        clock: None,
        status_detail: "FT".into(),
        venue: None,
        city: None,
    }
}

#[test]
fn filter_keeps_today_and_yesterdays_finished() {
    let match_day = NaiveDate::from_ymd_opt(2026, 6, 11).unwrap();
    let next_day = match_day.checked_add_days(Days::new(1)).unwrap();
    let later = match_day.checked_add_days(Days::new(2)).unwrap();

    let finished = vec![
        sample_finished_match(kickoff_local(match_day)),
        sample_finished_match(kickoff_local(match_day)),
    ];

    // Mismo día local: se conservan ambos.
    assert_eq!(filter_relevant(finished.clone(), match_day).len(), 2);
    // Al día siguiente: finalizados de ayer, se conservan ambos.
    assert_eq!(filter_relevant(finished.clone(), next_day).len(), 2);
    // Dos días después: fuera.
    assert_eq!(filter_relevant(finished.clone(), later).len(), 0);

    // Programados de ayer no se conservan.
    let scheduled = finished
        .iter()
        .map(|m| {
            let mut m = m.clone();
            m.status = MatchStatus::Scheduled;
            m
        })
        .collect();
    assert_eq!(filter_relevant(scheduled, next_day).len(), 0);
}

#[test]
fn missing_fields_do_not_break_parsing() {
    // Partido mínimo: sin venue, sin marcador, sin reloj, fecha rara.
    let json = r#"{
      "events": [{
        "id": "x1",
        "date": "no-es-fecha",
        "competitions": [{
          "competitors": [
            {"homeAway": "home", "team": {"displayName": "A", "abbreviation": "A"}},
            {"homeAway": "away", "team": {"displayName": "B", "abbreviation": "B"}}
          ]
        }],
        "status": {"type": {"state": "pre"}}
      }]
    }"#;
    let matches = parse_scoreboard(json).unwrap();
    assert_eq!(matches.len(), 1);
    let m = &matches[0];
    assert_eq!(m.status, MatchStatus::Scheduled);
    assert_eq!(m.home.score, None);
    assert_eq!(m.clock, None);
    assert_eq!(m.venue, None);
    assert_eq!(m.city, None);
    assert_eq!(m.kickoff, None);

    // Scoreboard vacío.
    assert!(parse_scoreboard(r#"{"events": []}"#).unwrap().is_empty());
    assert!(parse_scoreboard("{}").unwrap().is_empty());

    // Summary sin keyEvents.
    assert!(parse_summary("{}").unwrap().is_empty());
}
