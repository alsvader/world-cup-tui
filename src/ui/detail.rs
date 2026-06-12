use chrono::Local;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use world_cup_tui::model::{CardColor, KeyEvent, KeyEventKind, Match, MatchStatus};

use crate::app::App;
use crate::ui::{team_slot, theme};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let Some(m) = app.detail_match() else {
        frame.render_widget(
            Paragraph::new(Span::styled("CARGANDO PARTIDO...", theme::muted()))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(theme::border()),
                ),
            area,
        );
        return;
    };

    let [header, timeline] =
        Layout::vertical([Constraint::Length(5), Constraint::Min(0)]).areas(area);
    render_header(frame, m, app.emoji, header);
    render_timeline(frame, &app.events, app.emoji, timeline);
}

fn render_header(frame: &mut Frame, m: &Match, emoji: bool, area: Rect) {
    // ESPN reporta "0" en partidos no iniciados: no es marcador real.
    let score = match (m.status, m.home.score, m.away.score) {
        (MatchStatus::Scheduled, _, _) | (_, None, _) | (_, _, None) => "vs".to_string(),
        (_, Some(h), Some(a)) => format!("{h} - {a}"),
    };
    let score_line = Line::from(Span::styled(
        format!(
            "{} {}  {score}  {} {}",
            team_slot(&m.home, emoji),
            m.home.name.to_uppercase(),
            m.away.name.to_uppercase(),
            team_slot(&m.away, emoji),
        ),
        theme::base().add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);

    let status_line = Line::from(match m.status {
        MatchStatus::Live => vec![
            Span::styled("●", theme::live().add_modifier(Modifier::SLOW_BLINK)),
            Span::styled(
                format!(" EN VIVO · {}", m.clock.as_deref().unwrap_or("")),
                theme::live(),
            ),
        ],
        MatchStatus::HalfTime => vec![Span::styled("◐ MEDIO TIEMPO", theme::live())],
        MatchStatus::Finished => vec![Span::styled(
            format!("FINAL · {}", m.status_detail),
            theme::finished(),
        )],
        MatchStatus::Scheduled => vec![Span::styled(
            m.kickoff
                .map(|k| {
                    format!(
                        "PROGRAMADO · {}",
                        k.with_timezone(&Local).format("%d/%m · %H:%M")
                    )
                })
                .unwrap_or_else(|| "PROGRAMADO".to_string()),
            theme::upcoming(),
        )],
    })
    .alignment(Alignment::Center);

    let venue_line = Line::from(Span::styled(
        match (&m.venue, &m.city) {
            (Some(v), Some(c)) => format!("{v} · {c}"),
            (Some(v), None) => v.clone(),
            (None, Some(c)) => c.clone(),
            (None, None) => String::new(),
        },
        theme::muted(),
    ))
    .alignment(Alignment::Center);

    frame.render_widget(
        Paragraph::new(vec![score_line, status_line, venue_line]).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border()),
        ),
        area,
    );
}

fn render_timeline(frame: &mut Frame, events: &[KeyEvent], emoji: bool, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(" EVENTOS ", theme::panel_title()));
    if events.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("SIN EVENTOS TODAVÍA", theme::muted())).block(block),
            area,
        );
        return;
    }
    // Orden cronológico; si no caben todos, se muestran los más recientes.
    let visible = area.height.saturating_sub(2) as usize;
    let skip = events.len().saturating_sub(visible);
    let lines: Vec<Line> = events
        .iter()
        .skip(skip)
        .map(|ev| event_line(ev, emoji))
        .collect();
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Icono del evento según la política de emoji. Todos los strings ocupan
/// exactamente 3 celdas en pantalla — no se usa padding de `format!` porque
/// cuenta chars, y un emoji es 1 char que pinta 2 celdas.
fn event_icon(kind: &KeyEventKind, emoji: bool) -> &'static str {
    match (kind, emoji) {
        (KeyEventKind::Goal { .. }, true) => "⚽ ",
        (KeyEventKind::Goal { .. }, false) => "G  ",
        (KeyEventKind::Card(CardColor::Yellow), true) => "🟨 ",
        (KeyEventKind::Card(CardColor::Yellow), false) => "A  ",
        (KeyEventKind::Card(CardColor::Red), true) => "🟥 ",
        (KeyEventKind::Card(CardColor::Red), false) => "R  ",
        (KeyEventKind::Substitution, true) => "🔁 ",
        (KeyEventKind::Substitution, false) => "<> ",
        (KeyEventKind::Other, _) => "·  ",
    }
}

fn event_line(ev: &KeyEvent, emoji: bool) -> Line<'static> {
    let icon = event_icon(&ev.kind, emoji);
    let minute = format!("{:>7}", ev.minute);
    let who = || {
        let player = ev.player.clone().unwrap_or_default();
        match &ev.team {
            Some(t) => format!("{player} ({t})"),
            None => player,
        }
    };
    match &ev.kind {
        KeyEventKind::Goal { detail } => {
            let suffix = detail
                .as_deref()
                .map(|d| format!(" — {}", goal_detail_es(d)))
                .unwrap_or_default();
            Line::from(Span::styled(
                format!(" {icon}{minute}  GOL  {}{suffix}", who()),
                theme::goal(),
            ))
        }
        KeyEventKind::Card(CardColor::Yellow) => Line::from(Span::styled(
            format!(" {icon}{minute}  {}", who()),
            theme::card_yellow(),
        )),
        KeyEventKind::Card(CardColor::Red) => Line::from(Span::styled(
            format!(" {icon}{minute}  {}", who()),
            theme::card_red(),
        )),
        KeyEventKind::Substitution => Line::from(Span::styled(
            format!(" {icon}{minute}  {}", ev.text.clone()),
            theme::muted(),
        )),
        KeyEventKind::Other => Line::from(Span::styled(
            format!(" {icon}{minute}  {}", ev.text.clone()),
            theme::finished(),
        )),
    }
}

fn goal_detail_es(detail: &str) -> String {
    match detail {
        "Goal - Header" => "de cabeza".to_string(),
        "Goal - Free-kick" | "Goal - Free Kick" => "de tiro libre".to_string(),
        "Penalty - Scored" => "de penal".to_string(),
        "Own Goal" => "autogol".to_string(),
        "Goal - Volley" => "de volea".to_string(),
        other => other.to_lowercase(),
    }
}
