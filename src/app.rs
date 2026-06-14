use chrono::{DateTime, Days, Local, NaiveDate};
use world_cup_tui::espn::{is_in_poll_window, previous_jornada_target};
use world_cup_tui::model::{KeyEvent, Match, MatchStatus};

use crate::DataMsg;

/// Línea del panel FINALIZADOS: separador de jornada o fila de partido.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishedLine {
    Separator(NaiveDate),
    Match(usize),
}

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
    /// Día local más antiguo ya incluido en `matches` (inicialmente ayer).
    pub earliest_loaded: Option<NaiveDate>,
    pub history_loading: bool,
    /// Scroll del panel FINALIZADOS; `None` = pegado al fondo.
    pub finished_scroll: Option<usize>,
    pub finished_max_offset: usize,
}

impl App {
    pub fn new(emoji: bool) -> Self {
        Self {
            emoji,
            needs_clear: false,
            timeline_mode: TimelineMode::Key,
            timeline_scroll: None,
            timeline_max_offset: 0,
            earliest_loaded: None,
            history_loading: false,
            finished_scroll: None,
            finished_max_offset: 0,
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
    /// Finalizados: kickoff descendente (más reciente primero), igual que el panel.
    pub fn display_order(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.matches.len()).collect();
        idx.sort_by(|&a, &b| {
            let ma = &self.matches[a];
            let mb = &self.matches[b];
            let ra = status_rank(ma);
            let rb = status_rank(mb);
            ra.cmp(&rb).then_with(|| kickoff_display_order(ma, mb, ra))
        });
        idx
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.finished_scroll = None;
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.matches.len() {
            self.selected += 1;
        }
        self.finished_scroll = None;
    }

    pub fn can_load_previous(&self) -> bool {
        !self.history_loading
            && self
                .earliest_loaded
                .is_some_and(|d| previous_jornada_target(d).is_some())
    }

    pub fn try_start_history_load(&mut self) -> Option<NaiveDate> {
        if !self.can_load_previous() {
            return None;
        }
        let earliest = self.earliest_loaded.unwrap();
        let target = previous_jornada_target(earliest).unwrap();
        self.history_loading = true;
        Some(target)
    }

    /// Líneas del panel FINALIZADOS: separadores por jornada, más reciente arriba.
    pub fn finished_lines(&self) -> Vec<FinishedLine> {
        let mut indices: Vec<usize> = self
            .matches
            .iter()
            .enumerate()
            .filter(|(_, m)| m.status == MatchStatus::Finished)
            .map(|(i, _)| i)
            .collect();
        indices.sort_by(|&a, &b| self.matches[b].kickoff.cmp(&self.matches[a].kickoff));

        let mut lines = Vec::new();
        let mut last_date: Option<NaiveDate> = None;
        for i in indices {
            let date = self.matches[i]
                .kickoff
                .map(|k| k.with_timezone(&Local).date_naive());
            if date != last_date
                && let Some(d) = date
            {
                lines.push(FinishedLine::Separator(d));
                last_date = Some(d);
            }
            lines.push(FinishedLine::Match(i));
        }
        lines
    }

    /// Ajusta `finished_scroll` para que la línea `line_idx` quede visible.
    pub fn ensure_finished_line_visible(&mut self, line_idx: usize, visible: usize) {
        if visible == 0 {
            return;
        }
        let total = self.finished_lines().len();
        let max_offset = total.saturating_sub(visible);
        let cur = self.finished_scroll.unwrap_or(max_offset);
        if line_idx < cur {
            self.finished_scroll = Some(line_idx);
        } else if line_idx >= cur + visible {
            self.finished_scroll = Some(line_idx.saturating_sub(visible - 1).min(max_offset));
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
                let shape = |ms: &[Match]| -> Vec<(String, MatchStatus)> {
                    ms.iter().map(|m| (m.id.clone(), m.status)).collect()
                };
                let before = shape(&self.matches);
                merge_poll_matches(&mut self.matches, matches);
                if shape(&self.matches) != before {
                    self.needs_clear = true;
                }
                if self.earliest_loaded.is_none() {
                    let today = Local::now().date_naive();
                    self.earliest_loaded = today.checked_sub_days(Days::new(1));
                }
                if !self.matches.is_empty() && self.selected >= self.matches.len() {
                    self.selected = self.matches.len() - 1;
                }
                self.last_update = Some(Local::now());
                self.error = None;
            }
            DataMsg::HistoryMatches { date, matches } => {
                merge_history_matches(&mut self.matches, matches);
                self.earliest_loaded = Some(date);
                self.history_loading = false;
                self.finished_scroll = None;
                self.needs_clear = true;
                self.last_update = Some(Local::now());
                self.error = None;
            }
            DataMsg::HistoryLoadFailed => {
                self.history_loading = false;
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

/// Fusiona el resultado del poll: actualiza la ventana hoy/ayer sin borrar historial.
pub fn merge_poll_matches(existing: &mut Vec<Match>, polled: Vec<Match>) {
    let today = Local::now().date_naive();
    existing.retain(|m| !is_in_poll_window(m, today));
    for m in polled {
        if let Some(pos) = existing.iter().position(|x| x.id == m.id) {
            existing[pos] = m;
        } else {
            existing.push(m);
        }
    }
}

/// Anexa partidos históricos sin duplicar ids.
pub fn merge_history_matches(existing: &mut Vec<Match>, new_matches: Vec<Match>) {
    for m in new_matches {
        if !existing.iter().any(|x| x.id == m.id) {
            existing.push(m);
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

fn kickoff_display_order(a: &Match, b: &Match, rank: u8) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a.kickoff, b.kickoff) {
        (Some(ka), Some(kb)) if rank == 2 => kb.cmp(&ka),
        (Some(ka), Some(kb)) => ka.cmp(&kb),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
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

    #[test]
    fn merge_poll_keeps_history_outside_window() {
        use chrono::{TimeZone, Utc};
        let today = Local::now().date_naive();
        let old_day = today.checked_sub_days(Days::new(5)).unwrap();
        let kickoff_old = Local
            .from_local_datetime(&old_day.and_hms_opt(12, 0, 0).unwrap())
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let mut existing = vec![Match {
            id: "old".into(),
            kickoff: Some(kickoff_old),
            home: Team {
                name: "A".into(),
                abbrev: "AAA".into(),
                score: Some(1),
            },
            away: Team {
                name: "B".into(),
                abbrev: "BBB".into(),
                score: Some(0),
            },
            status: MatchStatus::Finished,
            clock: None,
            status_detail: "FT".into(),
            venue: None,
            city: None,
        }];
        let polled = vec![mk_match()];
        merge_poll_matches(&mut existing, polled);
        assert_eq!(existing.len(), 2);
        assert!(existing.iter().any(|m| m.id == "old"));
    }

    #[test]
    fn merge_history_dedupes_by_id() {
        let m = mk_match();
        let mut existing = vec![m.clone()];
        merge_history_matches(&mut existing, vec![m]);
        assert_eq!(existing.len(), 1);
    }

    #[test]
    fn previous_jornada_respects_tournament_start() {
        use world_cup_tui::espn::{previous_jornada_target, tournament_start};
        assert_eq!(previous_jornada_target(tournament_start()), None);
    }
}
