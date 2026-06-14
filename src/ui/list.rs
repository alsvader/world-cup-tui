//! Vista principal: dashboard de tres paneles (EN VIVO / PRÓXIMOS /
//! FINALIZADOS) según la pantalla "World Cup TUI Dashboard" de Stitch.

use chrono::{Datelike, Days, Local, NaiveDate};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use world_cup_tui::model::{Match, MatchStatus};

use crate::app::{App, FinishedLine, scroll_window, status_rank};
use crate::ui::{team_slot, theme};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
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
    let [live_a, up_a, fin_a] = Layout::vertical([
        Constraint::Length(panel_h(live.len(), 0)),
        Constraint::Length(panel_h(upcoming.len(), 1)), // +1: fila de columnas
        Constraint::Min(6),
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

fn render_finished(frame: &mut Frame, app: &mut App, rows: &[(usize, usize)], area: Rect) {
    let title_right = finished_panel_hint(app);
    if rows.is_empty() {
        let msg = if app.can_load_previous() {
            " AÚN NINGUNO — PULSA [P] PARA JORNADAS ANTERIORES"
        } else if app.history_loading {
            " CARGANDO JORNADA ANTERIOR..."
        } else {
            " AÚN NINGUNO"
        };
        frame.render_widget(
            Paragraph::new(Span::styled(msg, theme::muted()))
                .block(panel(" ✓ FINALIZADOS ", title_right)),
            area,
        );
        return;
    }

    let finished_lines = app.finished_lines();
    let total = finished_lines.len();
    let visible = area.height.saturating_sub(2) as usize;
    app.finished_max_offset = total.saturating_sub(visible);
    if app
        .finished_scroll
        .is_some_and(|s| s >= app.finished_max_offset)
    {
        app.finished_scroll = None;
    }

    let sel_pos = app.selected;
    for (li, line) in finished_lines.iter().enumerate() {
        if let FinishedLine::Match(i) = line
            && rows.iter().any(|&(pos, idx)| pos == sel_pos && idx == *i)
        {
            app.ensure_finished_line_visible(li, visible);
            break;
        }
    }

    let (start, above, below) = scroll_window(total, visible, app.finished_scroll);

    let mut block = panel(" ✓ FINALIZADOS ", title_right);
    if above > 0 {
        block = block.title_top(
            Line::from(Span::styled(format!("▲ {above} "), theme::refresh())).right_aligned(),
        );
    }
    if below > 0 {
        block = block.title_bottom(
            Line::from(Span::styled(format!("▼ {below} "), theme::refresh())).right_aligned(),
        );
    }

    let display_lines: Vec<Line> = finished_lines
        .iter()
        .skip(start)
        .take(visible)
        .map(|line| match line {
            FinishedLine::Separator(d) => {
                Line::from(Span::styled(jornada_separator_label(*d), theme::muted()))
            }
            FinishedLine::Match(i) => {
                let selected = rows.iter().any(|&(pos, idx)| pos == sel_pos && idx == *i);
                score_row(&app.matches[*i], selected, false, app.emoji)
            }
        })
        .collect();

    frame.render_widget(Paragraph::new(display_lines).block(block), area);
}

fn finished_panel_hint(app: &App) -> Option<Line<'static>> {
    if app.history_loading {
        Some(Line::from(Span::styled("[···] ", theme::refresh())))
    } else if app.can_load_previous() {
        Some(Line::from(Span::styled("[P] MÁS ", theme::finished())))
    } else {
        None
    }
}

/// Fila con marcador en caja: ` 62'   MEXICO 🇲🇽   [ 2 - 0 ]  🇿🇦 SOUTH AFRICA   sede`.
fn score_row(m: &Match, selected: bool, live: bool, emoji: bool) -> Line<'static> {
    let left = if live {
        match m.status {
            MatchStatus::Live => m.clock.clone().unwrap_or_else(|| "··'".into()),
            MatchStatus::HalfTime => "MT".into(),
            _ => m.status_detail.clone(),
        }
    } else {
        finished_left_label(m)
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

fn finished_left_label(m: &Match) -> String {
    let today = Local::now().date_naive();
    let yesterday = today.checked_sub_days(Days::new(1));
    match m.kickoff {
        Some(k) => {
            let d = k.with_timezone(&Local).date_naive();
            if d == today {
                if m.status_detail.is_empty() {
                    "FT".into()
                } else {
                    m.status_detail.clone()
                }
            } else if Some(d) == yesterday {
                "AYER".into()
            } else {
                jornada_row_label(d)
            }
        }
        None => m.status_detail.clone(),
    }
}

fn jornada_row_label(d: NaiveDate) -> String {
    const DAYS: [&str; 7] = ["DOM", "LUN", "MAR", "MIÉ", "JUE", "VIE", "SÁB"];
    let idx = d.weekday().num_days_from_sunday() as usize;
    format!("{} {:02}/{:02}", DAYS[idx], d.day(), d.month())
}

fn jornada_separator_label(d: NaiveDate) -> String {
    const DAYS: [&str; 7] = ["DOM", "LUN", "MAR", "MIÉ", "JUE", "VIE", "SÁB"];
    const MONTHS: [&str; 12] = [
        "ENE", "FEB", "MAR", "ABR", "MAY", "JUN", "JUL", "AGO", "SEP", "OCT", "NOV", "DIC",
    ];
    let idx = d.weekday().num_days_from_sunday() as usize;
    format!(
        "─── {} {} {} ───",
        DAYS[idx],
        d.day(),
        MONTHS[d.month() as usize - 1]
    )
}
