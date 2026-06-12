//! Traducción a terminal del sistema de diseño de `DESIGN.md` (raíz del
//! proyecto). Todos los colores de la UI viven aquí; nada de hardcodear
//! colores en las vistas.
//!
//! Si el terminal no anuncia truecolor (`COLORTERM`), cada token cae a su
//! aproximación ANSI-256: los emuladores sin 24-bit (p. ej. Terminal.app)
//! malinterpretan las secuencias `38;2;r;g;b` y producen colores erráticos.

use std::env;
use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};

fn truecolor() -> bool {
    static TC: OnceLock<bool> = OnceLock::new();
    *TC.get_or_init(|| {
        env::var("COLORTERM")
            .map(|v| {
                let v = v.to_lowercase();
                v.contains("truecolor") || v.contains("24bit")
            })
            .unwrap_or(false)
    })
}

/// Token de color: RGB exacto del DS si hay truecolor, índice ANSI-256 si no.
fn token(rgb: (u8, u8, u8), ansi: u8) -> Color {
    if truecolor() {
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    } else {
        Color::Indexed(ansi)
    }
}

// Paleta base (tokens del frontmatter de DESIGN.md).
fn bg() -> Color {
    token((0x14, 0x13, 0x13), 233) // surface
}
fn text() -> Color {
    token((0xe5, 0xe2, 0xe1), 254) // on-surface
}
fn text_muted() -> Color {
    token((0x91, 0x90, 0x95), 245) // outline
}
fn border_c() -> Color {
    token((0x47, 0x46, 0x4a), 238) // outline-variant
}
fn primary() -> Color {
    token((0xc8, 0xc6, 0xc8), 251) // primary
}
fn on_primary() -> Color {
    token((0x31, 0x30, 0x32), 236) // on-primary
}
fn error_c() -> Color {
    token((0xff, 0xb4, 0xab), 217) // error
}

// Semántica de estado (sección "Colors" de DESIGN.md).
fn live_c() -> Color {
    token((0x4a, 0xde, 0x80), 78) // verde vibrante: en vivo
}
fn upcoming_c() -> Color {
    token((0x60, 0xa5, 0xfa), 75) // azul: próximos
}
fn goal_c() -> Color {
    token((0x10, 0xb9, 0x81), 36) // esmeralda: goles
}
fn refresh_c() -> Color {
    token((0x22, 0xd3, 0xee), 45) // cyan técnico: polling
}

// Tarjetas: semántica futbolística, fuera del DS pero necesaria.
fn card_yellow_c() -> Color {
    token((0xfa, 0xcc, 0x15), 220)
}

pub fn base() -> Style {
    Style::default().fg(text()).bg(bg())
}

pub fn muted() -> Style {
    Style::default().fg(text_muted()).bg(bg())
}

pub fn border() -> Style {
    Style::default().fg(border_c()).bg(bg())
}

/// Títulos de panel: label-caps — siempre en MAYÚSCULAS, en negrita.
pub fn panel_title() -> Style {
    Style::default()
        .fg(text())
        .bg(bg())
        .add_modifier(Modifier::BOLD)
}

pub fn live() -> Style {
    Style::default()
        .fg(live_c())
        .bg(bg())
        .add_modifier(Modifier::BOLD)
}

pub fn upcoming() -> Style {
    Style::default().fg(upcoming_c()).bg(bg())
}

pub fn finished() -> Style {
    Style::default().fg(text_muted()).bg(bg())
}

pub fn goal() -> Style {
    Style::default()
        .fg(goal_c())
        .bg(bg())
        .add_modifier(Modifier::BOLD)
}

pub fn refresh() -> Style {
    Style::default().fg(refresh_c()).bg(bg())
}

pub fn error() -> Style {
    Style::default().fg(error_c()).bg(bg())
}

pub fn card_yellow() -> Style {
    Style::default().fg(card_yellow_c()).bg(bg())
}

pub fn card_red() -> Style {
    Style::default()
        .fg(error_c())
        .bg(bg())
        .add_modifier(Modifier::BOLD)
}

/// Estado activo/seleccionado: inversión explícita primary/on-primary del DS.
/// Colores concretos en vez de `Modifier::REVERSED`, que se renderiza de
/// forma inconsistente entre emuladores.
pub fn selected() -> Style {
    Style::default()
        .fg(on_primary())
        .bg(primary())
        .add_modifier(Modifier::BOLD)
}
