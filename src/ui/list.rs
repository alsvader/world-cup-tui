//! Vista principal: dashboard de tres paneles (EN VIVO / PRÓXIMOS /
//! FINALIZADOS) según la pantalla "World Cup TUI Dashboard" de Stitch.

use chrono::Local;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

use world_cup_tui::model::{Match, MatchStatus};

use crate::app::{status_rank, App};
use crate::ui::{team_slot, theme};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let order = app.display_order();
    if order.is_empty() && app.last_update.is_none() {
        frame.render_widget(
            Paragraph::new(Span::styled(" CARGANDO PARTIDOS...", theme::muted())).block(
                Block::bordered()
                    .border_style(theme::border())
                    .title(Span::styled(" PARTIDOS DE HOY ", theme::panel_title())),
            ),
            area,
        );
        return;
    }

    // (posición global en display_order, índice en app.matches) por panel.
    let group = |rank: u8| -> Vec<(usize, usize)> {
        order
            .iter()
            .enumerate()
            .filter(|(_, i)| status_rank(&app.matches[**i]) == rank)
            .map(|(pos, i)| (pos, *i))
            .collect()
    };
    let live = group(0);
    let upcoming = group(1);
    let finished = group(2);

    let panel_h = |rows: usize, extra: u16| rows.max(1) as u16 + 2 + extra;
    let [live_a, up_a, fin_a, _] = Layout::vertical([
        Constraint::Length(panel_h(live.len(), 0)),
        Constraint::Length(panel_h(upcoming.len(), 1)), // +1: fila de columnas
        Constraint::Length(panel_h(finished.len(), 0)),
        Constraint::Min(0),
    ])
    .areas(area);

    render_live(frame, app, &live, live_a);
    render_upcoming(frame, app, &upcoming, up_a);
    render_finished(frame, app, &finished, fin_a);
}

fn panel<'a>(title: &'a str, right: Option<Line<'a>>) -> Block<'a> {
    let mut b = Block::bordered()
        .border_style(theme::border())
        .title_top(Line::from(Span::styled(title, theme::panel_title())).left_aligned());
    if let Some(r) = right {
        b = b.title_top(r.right_aligned());
    }
    b
}

fn render_live(frame: &mut Frame, app: &App, rows: &[(usize, usize)], area: Rect) {
    let badge = Line::from(Span::styled(
        format!("[{} ACTIVOS] ", rows.len()),
        theme::live(),
    ));
    let block = panel(" ● EN VIVO ", Some(badge));
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(" SIN PARTIDOS EN VIVO AHORA", theme::muted()))
                .block(block),
            area,
        );
        return;
    }
    let lines: Vec<Line> = rows
        .iter()
        .map(|&(pos, i)| score_row(&app.matches[i], pos == app.selected, true, app.emoji))
        .collect();
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_upcoming(frame: &mut Frame, app: &App, rows: &[(usize, usize)], area: Rect) {
    let utc_hours = chrono::Offset::fix(Local::now().offset()).local_minus_utc() / 3600;
    let tz = Line::from(Span::styled(
        format!("[HOY · UTC{utc_hours:+}] "),
        theme::upcoming(),
    ));
    let block = panel(" ○ PRÓXIMOS ", Some(tz));
    let header = Line::from(Span::styled(
        format!(" {:<9}{:<44}{}", "HORA", "PARTIDO", "SEDE"),
        theme::muted(),
    ));
    if rows.is_empty() {
        let lines = vec![
            header,
            Line::from(Span::styled(" NO QUEDAN PARTIDOS HOY", theme::muted())),
        ];
        frame.render_widget(Paragraph::new(lines).block(block), area);
        return;
    }
    let mut lines = vec![header];
    lines.extend(rows.iter().map(|&(pos, i)| {
        let m = &app.matches[i];
        let kickoff = m
            .kickoff
            .map(|k| k.with_timezone(&Local).format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".into());
        let matchup = format!(
            "{} {} vs {} {}",
            team_slot(&m.home, app.emoji),
            m.home.name.to_uppercase(),
            m.away.name.to_uppercase(),
            team_slot(&m.away, app.emoji),
        );
        let venue = match (&m.venue, &m.city) {
            (Some(v), Some(c)) => format!("{v} · {c}"),
            (Some(v), None) => v.clone(),
            (None, Some(c)) => c.clone(),
            (None, None) => String::new(),
        };
        let style = if pos == app.selected {
            theme::selected()
        } else {
            theme::base()
        };
        Line::from(Span::styled(
            format!(" {kickoff:<9}{matchup:<44}{venue}"),
            style,
        ))
    }));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_finished(frame: &mut Frame, app: &App, rows: &[(usize, usize)], area: Rect) {
    let block = panel(" ✓ FINALIZADOS ", None);
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(" AÚN NINGUNO", theme::muted())).block(block),
            area,
        );
        return;
    }
    let lines: Vec<Line> = rows
        .iter()
        .map(|&(pos, i)| score_row(&app.matches[i], pos == app.selected, false, app.emoji))
        .collect();
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// Fila con marcador en caja: ` 62'   MEXICO 🇲🇽   [ 2 - 0 ]  🇿🇦 SOUTH AFRICA   sede`.
/// El slot de identidad mide 3 celdas en ambos modos: trigrama ASCII de 3
/// chars, o bandera (2 celdas) + 1 espacio de padding de `{:<3}`.
fn score_row(m: &Match, selected: bool, live: bool, emoji: bool) -> Line<'static> {
    let left = match m.status {
        MatchStatus::Live => m.clock.clone().unwrap_or_else(|| "··'".into()),
        MatchStatus::HalfTime => "MT".into(),
        MatchStatus::Finished
            if m.kickoff.is_some_and(|k| {
                k.with_timezone(&Local).date_naive() < Local::now().date_naive()
            }) =>
        {
            "AYER".into()
        }
        _ => m.status_detail.clone(),
    };
    let score = match (m.home.score, m.away.score) {
        (Some(h), Some(a)) => format!("[ {h} - {a} ]"),
        _ => "[  vs  ]".into(),
    };
    let venue = m.venue.clone().unwrap_or_default();
    let hid = team_slot(&m.home, emoji);
    let aid = team_slot(&m.away, emoji);
    let text = format!(
        " {left:<8}{:>17} {hid:<3}  {score:^9}  {aid:<3} {:<17}  {venue}",
        m.home.name.to_uppercase(),
        m.away.name.to_uppercase()
    );
    let style = if selected {
        theme::selected()
    } else if live {
        theme::live()
    } else {
        theme::finished()
    };
    Line::from(Span::styled(text, style))
}
