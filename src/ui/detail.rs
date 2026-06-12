use chrono::Local;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use world_cup_tui::model::{CardColor, KeyEvent, KeyEventKind, Match, MatchStatus};

use crate::app::{scroll_window, timeline_rows, App, Column, TimelineMode};
use crate::ui::{team_slot, theme};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(m) = app.detail_match().cloned() else {
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
    render_header(frame, &m, app.emoji, header);
    render_timeline(frame, app, &m, timeline);
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

/// Timeline en dos columnas (estilo match centre): cronología compartida,
/// minuto al centro, evento en la columna de su equipo. Encabezado de
/// equipos fijo; el cuerpo scrollea con sticky bottom.
fn render_timeline(frame: &mut Frame, app: &mut App, m: &Match, area: Rect) {
    let rows = timeline_rows(&app.events, m, app.timeline_mode);
    let total = rows.len();
    // bordes (2) + fila fija de encabezado de columnas (1)
    let visible = area.height.saturating_sub(3) as usize;
    app.timeline_max_offset = total.saturating_sub(visible);
    // Clamp persistente: si el offset quedó fuera de rango (lista o panel
    // cambiaron), vuelve al modo seguir-en-vivo.
    if app.timeline_scroll.is_some_and(|s| s >= app.timeline_max_offset) {
        app.timeline_scroll = None;
    }
    let (start, above, below) = scroll_window(total, visible, app.timeline_scroll);

    let title = match app.timeline_mode {
        TimelineMode::Key => " EVENTOS — GOLES Y TARJETAS [T: TODO] ",
        TimelineMode::All => " EVENTOS — TODO [T: CLAVE] ",
    };
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title_top(Line::from(Span::styled(title, theme::panel_title())).left_aligned());
    if above > 0 {
        block = block
            .title_top(Line::from(Span::styled(format!("▲ {above} "), theme::refresh())).right_aligned());
    }
    if below > 0 {
        block = block.title_bottom(
            Line::from(Span::styled(format!("▼ {below} "), theme::refresh())).right_aligned(),
        );
    }

    let inner_w = area.width.saturating_sub(2) as usize;
    let minute_w = 7usize;
    let col_w = inner_w.saturating_sub(minute_w + 2) / 2;

    let mut lines = vec![columns_header(m, app.emoji, col_w, minute_w)];
    if rows.is_empty() {
        let msg = match app.timeline_mode {
            TimelineMode::Key => "SIN GOLES NI TARJETAS TODAVÍA — [T] VER TODO",
            TimelineMode::All => "SIN EVENTOS TODAVÍA",
        };
        lines.push(Line::from(Span::styled(msg, theme::muted())).alignment(Alignment::Center));
    } else {
        for &(i, col) in rows.iter().skip(start).take(visible) {
            lines.push(event_row(&app.events[i], col, app.emoji, col_w, minute_w, inner_w));
        }
    }
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Fila fija con la identidad de cada equipo sobre su columna.
fn columns_header(m: &Match, emoji: bool, col_w: usize, minute_w: usize) -> Line<'static> {
    let home = format!("{} {}", team_slot(&m.home, emoji), m.home.name.to_uppercase());
    let away = format!("{} {}", m.away.name.to_uppercase(), team_slot(&m.away, emoji));
    let style = theme::base().add_modifier(Modifier::BOLD);
    Line::from(vec![
        Span::styled(pad_to(fit(&home, col_w), col_w), style),
        Span::raw(" ".repeat(minute_w + 2)),
        Span::styled(pad_left_to(fit(&away, col_w), col_w), style),
    ])
}

fn event_row(
    ev: &KeyEvent,
    col: Column,
    emoji: bool,
    col_w: usize,
    minute_w: usize,
    inner_w: usize,
) -> Line<'static> {
    if col == Column::Neutral {
        // Solo aparece en vista TODO: kickoff, medio tiempo, pausas...
        return Line::from(Span::styled(fit(&ev.text, inner_w), theme::finished()))
            .alignment(Alignment::Center);
    }
    let content = pad_to(fit(&event_text(ev, emoji), col_w), col_w);
    let minute = Span::styled(format!(" {:^w$} ", ev.minute, w = minute_w), theme::muted());
    let style = event_style(&ev.kind);
    match col {
        Column::Home => Line::from(vec![
            Span::styled(content, style),
            minute,
            Span::raw(" ".repeat(col_w)),
        ]),
        _ => Line::from(vec![
            Span::raw(" ".repeat(col_w)),
            minute,
            Span::styled(content, style),
        ]),
    }
}

fn event_text(ev: &KeyEvent, emoji: bool) -> String {
    let icon = event_icon(&ev.kind, emoji);
    let player = ev.player.clone().unwrap_or_else(|| ev.text.clone());
    match &ev.kind {
        KeyEventKind::Goal { detail } => {
            let suffix = detail
                .as_deref()
                .map(|d| format!(" — {}", goal_detail_es(d)))
                .unwrap_or_default();
            format!("{icon}{player}{suffix}")
        }
        _ => format!("{icon}{player}"),
    }
}

fn event_style(kind: &KeyEventKind) -> Style {
    match kind {
        KeyEventKind::Goal { .. } => theme::goal(),
        KeyEventKind::Card(CardColor::Yellow) => theme::card_yellow(),
        KeyEventKind::Card(CardColor::Red) => theme::card_red(),
        KeyEventKind::Substitution => theme::muted(),
        KeyEventKind::Other => theme::finished(),
    }
}

/// Truncado consciente del ancho de celda (los emoji miden 2), con elipsis.
fn fit(s: &str, w: usize) -> String {
    if UnicodeWidthStr::width(s) <= w {
        return s.to_string();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if used + cw > w.saturating_sub(1) {
            break;
        }
        out.push(c);
        used += cw;
    }
    out.push('…');
    out
}

fn pad_to(s: String, w: usize) -> String {
    let pad = w.saturating_sub(UnicodeWidthStr::width(s.as_str()));
    s + &" ".repeat(pad)
}

fn pad_left_to(s: String, w: usize) -> String {
    let pad = w.saturating_sub(UnicodeWidthStr::width(s.as_str()));
    " ".repeat(pad) + &s
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
