use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchStatus {
    Scheduled,
    Live,
    HalfTime,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub name: String,
    pub abbrev: String,
    pub score: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub id: String,
    pub kickoff: Option<DateTime<Utc>>,
    pub home: Team,
    pub away: Team,
    pub status: MatchStatus,
    /// Reloj del partido tal como lo reporta la fuente, p. ej. "67'" o "45'+4'".
    pub clock: Option<String>,
    /// Detalle textual del estado, p. ej. "FT", "HT".
    pub status_detail: String,
    pub venue: Option<String>,
    pub city: Option<String>,
}

impl Match {
    pub fn is_live(&self) -> bool {
        matches!(self.status, MatchStatus::Live | MatchStatus::HalfTime)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardColor {
    Yellow,
    Red,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyEventKind {
    /// `detail` conserva la variante reportada por la fuente, p. ej. "Header".
    Goal {
        detail: Option<String>,
    },
    Card(CardColor),
    Substitution,
    /// Cualquier tipo no reconocido se conserva como texto, nunca se descarta.
    Other,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub minute: String,
    pub period: u32,
    pub kind: KeyEventKind,
    pub player: Option<String>,
    pub team: Option<String>,
    pub text: String,
}
