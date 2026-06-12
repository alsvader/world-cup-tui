mod detail;
mod list;
pub mod theme;

use chrono::Local;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use world_cup_tui::flags::flag_emoji;
use world_cup_tui::model::Team;

use crate::app::{App, View};

/// Slot de identidad de una selección: bandera emoji si la política lo
/// permite y el equipo tiene ISO-2; trigrama FIFA en caso contrario.
pub fn team_slot(team: &Team, emoji: bool) -> String {
    if emoji && let Some(flag) = flag_emoji(&team.abbrev) {
        return flag;
    }
    team.abbrev.clone()
}

pub fn render(frame: &mut Frame, app: &mut App) {
    // Fondo del sistema de diseño en toda la pantalla.
    frame.render_widget(Block::default().style(theme::base()), frame.area());

    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_header(frame, header);
    match app.view {
        View::List => list::render(frame, app, body),
        View::Detail => detail::render(frame, app, body),
    }
    render_footer(frame, app, footer);
}

/// Barra superior del dashboard: marca a la izquierda, fecha/hora local a la
/// derecha (sin segundos, para no parpadear cada segundo).
fn render_header(frame: &mut Frame, area: Rect) {
    let clock = Local::now()
        .format("%d %b %Y · %H:%M")
        .to_string()
        .to_uppercase();
    let [left, right] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(clock.len() as u16 + 1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            " WORLD CUP TUI",
            theme::base().add_modifier(Modifier::BOLD),
        )),
        left,
    );
    frame.render_widget(Paragraph::new(Span::styled(clock, theme::muted())), right);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keys = match app.view {
        View::List => " [Q] SALIR · [R] REFRESCAR · [J/K] NAVEGAR · [ENTER] DETALLE",
        View::Detail => " [ESC] VOLVER · [T] FILTRO · [↑↓] SCROLL · [R] REFRESCAR · [Q] SALIR",
    };
    let mut right_spans = Vec::new();
    if let Some(t) = app.last_update {
        right_spans.push(Span::styled(
            format!("ACTUALIZADO {} ", t.format("%H:%M:%S")),
            theme::refresh(),
        ));
    }
    if app.error.is_some() {
        right_spans.insert(
            0,
            Span::styled("SIN CONEXIÓN — REINTENTANDO · ", theme::error()),
        );
    }
    let right_w: usize = right_spans.iter().map(|s| s.content.chars().count()).sum();
    let [left, right] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(right_w as u16)]).areas(area);
    frame.render_widget(Paragraph::new(Span::styled(keys, theme::muted())), left);
    frame.render_widget(Paragraph::new(Line::from(right_spans)), right);
}
