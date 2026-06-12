use chrono::{DateTime, Local};
use world_cup_tui::model::{KeyEvent, Match, MatchStatus};

use crate::DataMsg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMode {
    /// Solo goles y tarjetas (default).
    Key,
    /// Todos los eventos del feed.
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Home,
    Away,
    Neutral,
}

pub struct App {
    pub view: View,
    pub matches: Vec<Match>,
    /// Posición seleccionada dentro de `display_order()`.
    pub selected: usize,
    pub detail_id: Option<String>,
    pub events: Vec<KeyEvent>,
    pub last_update: Option<DateTime<Local>>,
    pub error: Option<String>,
    pub should_quit: bool,
    /// Política de emoji resuelta al arranque (banderas e iconos del timeline).
    pub emoji: bool,
    /// Pide un repintado completo del siguiente frame. Necesario porque las
    /// banderas emoji son pares de regional indicators que el terminal liga
    /// en un glifo: el diff parcial de ratatui puede dejar mitades huérfanas
    /// (o ligarlas con mitades nuevas formando otra bandera) cuando el
    /// contenido cambia de posición.
    pub needs_clear: bool,
    pub timeline_mode: TimelineMode,
    /// `None` = pegado al fondo (seguir-en-vivo); `Some(offset)` = fijo.
    pub timeline_scroll: Option<usize>,
    /// Último offset máximo conocido, escrito por el render (la altura del
    /// panel solo se conoce ahí) y leído por el manejo de teclas.
    pub timeline_max_offset: usize,
}

impl App {
    pub fn new(emoji: bool) -> Self {
        Self {
            emoji,
            needs_clear: false,
            timeline_mode: TimelineMode::Key,
            timeline_scroll: None,
            timeline_max_offset: 0,
            view: View::List,
            matches: Vec::new(),
            selected: 0,
            detail_id: None,
            events: Vec::new(),
            last_update: None,
            error: None,
            should_quit: false,
        }
    }

    /// Índices de `matches` en orden de presentación: en vivo, próximos, finalizados.
    pub fn display_order(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.matches.len()).collect();
        idx.sort_by_key(|&i| (status_rank(&self.matches[i]), self.matches[i].kickoff));
        idx
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.matches.len() {
            self.selected += 1;
        }
    }

    pub fn selected_match(&self) -> Option<&Match> {
        self.display_order()
            .get(self.selected)
            .map(|&i| &self.matches[i])
    }

    /// Abre el detalle del partido seleccionado y devuelve su id para el poller.
    pub fn open_selected(&mut self) -> Option<String> {
        let id = self.selected_match()?.id.clone();
        self.detail_id = Some(id.clone());
        self.events.clear();
        self.view = View::Detail;
        self.needs_clear = true;
        self.timeline_mode = TimelineMode::Key;
        self.timeline_scroll = None;
        Some(id)
    }

    pub fn toggle_timeline_mode(&mut self) {
        self.timeline_mode = match self.timeline_mode {
            TimelineMode::Key => TimelineMode::All,
            TimelineMode::All => TimelineMode::Key,
        };
        self.timeline_scroll = None;
        self.needs_clear = true;
    }

    pub fn timeline_scroll_up(&mut self) {
        let cur = self.timeline_scroll.unwrap_or(self.timeline_max_offset);
        self.timeline_scroll = Some(cur.saturating_sub(1));
    }

    pub fn timeline_scroll_down(&mut self) {
        if let Some(cur) = self.timeline_scroll {
            // Llegar al fondo restablece el modo seguir-en-vivo.
            self.timeline_scroll = if cur + 1 >= self.timeline_max_offset {
                None
            } else {
                Some(cur + 1)
            };
        }
    }

    /// Vuelve a la lista conservando la selección.
    pub fn close_detail(&mut self) {
        self.view = View::List;
        self.detail_id = None;
        self.events.clear();
        self.needs_clear = true;
    }

    pub fn detail_match(&self) -> Option<&Match> {
        let id = self.detail_id.as_deref()?;
        self.matches.iter().find(|m| m.id == id)
    }

    pub fn apply(&mut self, msg: DataMsg) {
        match msg {
            DataMsg::Matches(matches) => {
                // Si los partidos cambian de posición o estado, las filas se
                // mueven de panel y las banderas quedan en celdas distintas.
                let shape = |ms: &[Match]| -> Vec<(String, MatchStatus)> {
                    ms.iter().map(|m| (m.id.clone(), m.status)).collect()
                };
                if shape(&self.matches) != shape(&matches) {
                    self.needs_clear = true;
                }
                self.matches = matches;
                if !self.matches.is_empty() && self.selected >= self.matches.len() {
                    self.selected = self.matches.len() - 1;
                }
                self.last_update = Some(Local::now());
                self.error = None;
            }
            DataMsg::Events { id, events } => {
                // Descartar respuestas tardías de un detalle ya cerrado.
                if self.detail_id.as_deref() == Some(id.as_str()) {
                    self.events = events;
                    self.last_update = Some(Local::now());
                    self.error = None;
                }
            }
            DataMsg::Error(e) => {
                // Se conservan los últimos datos válidos; solo se anota el aviso.
                self.error = Some(e);
            }
        }
    }
}

pub fn status_rank(m: &Match) -> u8 {
    match m.status {
        MatchStatus::Live | MatchStatus::HalfTime => 0,
        MatchStatus::Scheduled => 1,
        MatchStatus::Finished => 2,
    }
}

pub fn event_column(ev: &KeyEvent, m: &Match) -> Column {
    match ev.team.as_deref() {
        Some(t) if t == m.home.name => Column::Home,
        Some(t) if t == m.away.name => Column::Away,
        _ => Column::Neutral,
    }
}

/// Filas visibles del timeline según el modo: índices en `events` + columna.
/// En modo Key solo goles/tarjetas con equipo asignable.
pub fn timeline_rows(events: &[KeyEvent], m: &Match, mode: TimelineMode) -> Vec<(usize, Column)> {
    use world_cup_tui::model::KeyEventKind;
    events
        .iter()
        .enumerate()
        .filter_map(|(i, ev)| {
            let col = event_column(ev, m);
            match mode {
                // ESPN duplica algunos eventos (pausas) con texto vacío: ruido.
                TimelineMode::All => {
                    (!ev.text.is_empty() || ev.player.is_some()).then_some((i, col))
                }
                TimelineMode::Key => {
                    (matches!(ev.kind, KeyEventKind::Goal { .. } | KeyEventKind::Card(_))
                        && col != Column::Neutral)
                        .then_some((i, col))
                }
            }
        })
        .collect()
}

/// Ventana de scroll: (primera fila visible, ocultas arriba, ocultas abajo).
/// `scroll: None` = pegado al fondo.
pub fn scroll_window(total: usize, visible: usize, scroll: Option<usize>) -> (usize, usize, usize) {
    let max_offset = total.saturating_sub(visible);
    let start = scroll.map_or(max_offset, |s| s.min(max_offset));
    let below = total.saturating_sub(start + visible);
    (start, start, below)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use world_cup_tui::model::{CardColor, KeyEventKind, Team};

    use super::*;

    fn mk_match() -> Match {
        let team = |name: &str| Team {
            name: name.into(),
            abbrev: name[..3].to_uppercase(),
            score: Some(0),
        };
        Match {
            id: "1".into(),
            kickoff: Some(Utc::now()),
            home: team("Mexico"),
            away: team("South Africa"),
            status: MatchStatus::Live,
            clock: None,
            status_detail: String::new(),
            venue: None,
            city: None,
        }
    }

    fn mk_event(team: Option<&str>, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            minute: "10'".into(),
            period: 1,
            kind,
            player: Some("X".into()),
            team: team.map(Into::into),
            text: "texto".into(),
        }
    }

    #[test]
    fn column_assignment() {
        let m = mk_match();
        let goal = KeyEventKind::Goal { detail: None };
        assert_eq!(
            event_column(&mk_event(Some("Mexico"), goal.clone()), &m),
            Column::Home
        );
        assert_eq!(
            event_column(&mk_event(Some("South Africa"), goal.clone()), &m),
            Column::Away
        );
        assert_eq!(
            event_column(&mk_event(Some("Otro"), goal.clone()), &m),
            Column::Neutral
        );
        assert_eq!(event_column(&mk_event(None, goal), &m), Column::Neutral);
    }

    #[test]
    fn key_mode_filters_to_goals_and_cards_with_team() {
        let m = mk_match();
        let mut empty_dup = mk_event(Some("Mexico"), KeyEventKind::Other);
        empty_dup.text = String::new();
        empty_dup.player = None;
        let events = vec![
            mk_event(None, KeyEventKind::Other), // kickoff
            mk_event(Some("Mexico"), KeyEventKind::Goal { detail: None }),
            mk_event(Some("South Africa"), KeyEventKind::Card(CardColor::Yellow)),
            mk_event(Some("Mexico"), KeyEventKind::Substitution),
            mk_event(None, KeyEventKind::Goal { detail: None }), // sin equipo
            empty_dup,                                           // duplicado vacío
        ];
        let key = timeline_rows(&events, &m, TimelineMode::Key);
        assert_eq!(key, vec![(1, Column::Home), (2, Column::Away)]);
        let all = timeline_rows(&events, &m, TimelineMode::All);
        assert_eq!(all.len(), 5, "el duplicado con texto vacío se omite");
        assert_eq!(all[0].1, Column::Neutral);
    }

    #[test]
    fn scroll_window_sticky_and_clamped() {
        // Pegado al fondo: muestra las últimas `visible`.
        assert_eq!(scroll_window(10, 4, None), (6, 6, 0));
        // Scrolleado arriba: posición fija, indica ocultas abajo.
        assert_eq!(scroll_window(10, 4, Some(2)), (2, 2, 4));
        // Al tope.
        assert_eq!(scroll_window(10, 4, Some(0)), (0, 0, 6));
        // Offset fuera de rango se clampa al fondo.
        assert_eq!(scroll_window(10, 4, Some(99)), (6, 6, 0));
        // Todo cabe: sin ocultas, scroll irrelevante.
        assert_eq!(scroll_window(3, 10, None), (0, 0, 0));
        assert_eq!(scroll_window(3, 10, Some(5)), (0, 0, 0));
        // Lista vacía.
        assert_eq!(scroll_window(0, 5, None), (0, 0, 0));
    }
}
