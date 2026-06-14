mod app;
mod ui;

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::Instant;

use world_cup_tui::espn;
use world_cup_tui::model::{KeyEvent as MatchEvent, Match};

use crate::app::{App, View};

pub enum DataMsg {
    Matches(Vec<Match>),
    HistoryMatches {
        date: chrono::NaiveDate,
        matches: Vec<Match>,
    },
    HistoryLoadFailed,
    Events {
        id: String,
        events: Vec<MatchEvent>,
    },
    Error(String),
}

pub enum Cmd {
    OpenDetail(String),
    CloseDetail,
    Refresh,
    LoadPreviousJornada(chrono::NaiveDate),
}

const SCOREBOARD_LIVE_SECS: u64 = 30;
const SUMMARY_LIVE_SECS: u64 = 15;
const IDLE_SECS: u64 = 120;

fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--version") {
        println!("world-cup-tui 0.1.0 — historial de jornadas ([p])");
        return Ok(());
    }
    let cli_override = std::env::args().fold(None, |acc, a| match a.as_str() {
        "--flags" => Some(true),
        "--no-flags" => Some(false),
        _ => acc,
    });
    let emoji = world_cup_tui::flags::emoji_enabled(cli_override);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let (data_tx, data_rx) = mpsc::unbounded_channel();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    rt.spawn(poller(data_tx, cmd_rx));

    // ratatui::init() instala también un panic hook que restaura la terminal.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, data_rx, cmd_tx, emoji);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    mut data_rx: UnboundedReceiver<DataMsg>,
    cmd_tx: UnboundedSender<Cmd>,
    emoji: bool,
) -> Result<()> {
    let mut app = App::new(emoji);
    loop {
        while let Ok(msg) = data_rx.try_recv() {
            app.apply(msg);
        }
        if app.needs_clear {
            app.needs_clear = false;
            // Fuerza la reescritura de TODAS las celdas del siguiente frame
            // (la UI pinta fondo en cada celda, así que sobrescribe cualquier
            // residuo de banderas emoji) sin un clear físico de pantalla:
            // sin fase en blanco, sin flash. swap_buffers() deja ambos
            // buffers vacíos, de modo que el diff del próximo draw ve todo
            // como cambiado.
            terminal.swap_buffers();
        }
        terminal.draw(|frame| ui::render(frame, &mut app))?;
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
            && key.kind == event::KeyEventKind::Press
        {
            handle_key(&mut app, &cmd_tx, key.code);
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, cmd_tx: &UnboundedSender<Cmd>, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('r') => {
            let _ = cmd_tx.send(Cmd::Refresh);
        }
        _ => match app.view {
            View::List => match code {
                KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if let Some(target) = app.try_start_history_load() {
                        let _ = cmd_tx.send(Cmd::LoadPreviousJornada(target));
                    }
                }
                KeyCode::Enter => {
                    if let Some(id) = app.open_selected() {
                        let _ = cmd_tx.send(Cmd::OpenDetail(id));
                    }
                }
                _ => {}
            },
            View::Detail => match code {
                KeyCode::Esc => {
                    app.close_detail();
                    let _ = cmd_tx.send(Cmd::CloseDetail);
                }
                KeyCode::Char('t') => app.toggle_timeline_mode(),
                KeyCode::Up | KeyCode::Char('k') => app.timeline_scroll_up(),
                KeyCode::Down | KeyCode::Char('j') => app.timeline_scroll_down(),
                _ => {}
            },
        },
    }
}

/// Tarea de datos: única dueña del I/O de red. Publica resultados por el canal
/// y ajusta su cadencia según haya o no partidos en vivo.
async fn poller(data_tx: UnboundedSender<DataMsg>, mut cmd_rx: UnboundedReceiver<Cmd>) {
    let client = match espn::Client::new() {
        Ok(c) => c,
        Err(e) => {
            let _ = data_tx.send(DataMsg::Error(e.to_string()));
            return;
        }
    };
    let mut detail: Option<String> = None;
    let mut any_live = false;
    let mut next_scoreboard = Instant::now();
    let mut next_summary: Option<Instant> = None;

    loop {
        let now = Instant::now();

        if now >= next_scoreboard {
            match client.fetch_scoreboard().await {
                Ok(matches) => {
                    any_live = matches.iter().any(|m| m.is_live());
                    let _ = data_tx.send(DataMsg::Matches(matches));
                }
                Err(e) => {
                    let _ = data_tx.send(DataMsg::Error(e.to_string()));
                }
            }
            let secs = if any_live {
                SCOREBOARD_LIVE_SECS
            } else {
                IDLE_SECS
            };
            next_scoreboard = Instant::now() + Duration::from_secs(secs);
        }

        if let Some(id) = detail.clone()
            && next_summary.is_some_and(|t| now >= t)
        {
            match client.fetch_summary(&id).await {
                Ok(events) => {
                    let _ = data_tx.send(DataMsg::Events { id, events });
                }
                Err(e) => {
                    let _ = data_tx.send(DataMsg::Error(e.to_string()));
                }
            }
            let secs = if any_live {
                SUMMARY_LIVE_SECS
            } else {
                IDLE_SECS
            };
            next_summary = Some(Instant::now() + Duration::from_secs(secs));
        }

        let mut deadline = next_scoreboard;
        if let Some(t) = next_summary {
            deadline = deadline.min(t);
        }

        tokio::select! {
            cmd = cmd_rx.recv() => match cmd {
                Some(Cmd::OpenDetail(id)) => {
                    detail = Some(id);
                    next_summary = Some(Instant::now());
                }
                Some(Cmd::CloseDetail) => {
                    detail = None;
                    next_summary = None;
                }
                Some(Cmd::Refresh) => {
                    next_scoreboard = Instant::now();
                    if detail.is_some() {
                        next_summary = Some(Instant::now());
                    }
                }
                Some(Cmd::LoadPreviousJornada(day)) => {
                    match client.fetch_scoreboard_day(day).await {
                        Ok(matches) => {
                            let _ = data_tx.send(DataMsg::HistoryMatches {
                                date: day,
                                matches,
                            });
                        }
                        Err(e) => {
                            let _ = data_tx.send(DataMsg::Error(e.to_string()));
                            let _ = data_tx.send(DataMsg::HistoryLoadFailed);
                        }
                    }
                }
                // La UI terminó: no queda nadie escuchando.
                None => return,
            },
            _ = tokio::time::sleep_until(deadline) => {}
        }
    }
}
