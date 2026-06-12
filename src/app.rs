use chrono::{DateTime, Local};
use world_cup_tui::model::{KeyEvent, Match, MatchStatus};

use crate::DataMsg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
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
}

impl App {
    pub fn new(emoji: bool) -> Self {
        Self {
            emoji,
            needs_clear: false,
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
        Some(id)
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
