//! Identidad visual de selecciones: trigrama FIFA universal + bandera emoji
//! condicional (progressive enhancement, ver design del cambio
//! `add-country-flags`).

/// Mapa trigrama FIFA → ISO 3166-1 alpha-2. Superconjunto generoso de los
/// clasificados al Mundial 2026: una entrada de más es inocua (se indexa por
/// lo que manda ESPN), una de menos solo degrada a trigrama.
/// ENG/SCO/WAL/NIR quedan fuera a propósito: sin ISO-2, sus banderas emoji
/// usan tag sequences mal soportadas entre terminales.
const FIFA_TO_ISO: &[(&str, &str)] = &[
    // CONCACAF
    ("CAN", "CA"),
    ("CRC", "CR"),
    ("CUW", "CW"),
    ("HAI", "HT"),
    ("HON", "HN"),
    ("JAM", "JM"),
    ("MEX", "MX"),
    ("PAN", "PA"),
    ("USA", "US"),
    // CONMEBOL
    ("ARG", "AR"),
    ("BOL", "BO"),
    ("BRA", "BR"),
    ("CHI", "CL"),
    ("COL", "CO"),
    ("ECU", "EC"),
    ("PAR", "PY"),
    ("PER", "PE"),
    ("URU", "UY"),
    ("VEN", "VE"),
    // UEFA
    ("ALB", "AL"),
    ("AUT", "AT"),
    ("BEL", "BE"),
    ("BIH", "BA"),
    ("CRO", "HR"),
    ("CZE", "CZ"),
    ("DEN", "DK"),
    ("ESP", "ES"),
    ("FRA", "FR"),
    ("GER", "DE"),
    ("GRE", "GR"),
    ("HUN", "HU"),
    ("IRL", "IE"),
    ("ISL", "IS"),
    ("ITA", "IT"),
    ("NED", "NL"),
    ("NOR", "NO"),
    ("POL", "PL"),
    ("POR", "PT"),
    ("ROU", "RO"),
    ("SRB", "RS"),
    ("SUI", "CH"),
    ("SVK", "SK"),
    ("SVN", "SI"),
    ("SWE", "SE"),
    ("TUR", "TR"),
    ("UKR", "UA"),
    // AFC
    ("AUS", "AU"),
    ("IDN", "ID"),
    ("IRN", "IR"),
    ("IRQ", "IQ"),
    ("JOR", "JO"),
    ("JPN", "JP"),
    ("KOR", "KR"),
    ("KSA", "SA"),
    ("QAT", "QA"),
    ("UAE", "AE"),
    ("UZB", "UZ"),
    // CAF
    ("ALG", "DZ"),
    ("CIV", "CI"),
    ("CMR", "CM"),
    ("COD", "CD"),
    ("CPV", "CV"),
    ("EGY", "EG"),
    ("GHA", "GH"),
    ("MAR", "MA"),
    ("NGA", "NG"),
    ("RSA", "ZA"),
    ("SEN", "SN"),
    ("TUN", "TN"),
    // OFC
    ("FIJ", "FJ"),
    ("NZL", "NZ"),
];

/// Bandera emoji (regional indicators) para un trigrama FIFA, si tiene ISO-2.
pub fn flag_emoji(fifa: &str) -> Option<String> {
    let iso = FIFA_TO_ISO
        .iter()
        .find(|(f, _)| *f == fifa)
        .map(|(_, iso)| *iso)?;
    Some(
        iso.chars()
            .map(|c| char::from_u32(0x1F1E6 + (c as u32 - 'A' as u32)).unwrap_or(c))
            .collect(),
    )
}

/// Política de emoji centralizada. Precedencia: override CLI > env
/// `WCTUI_FLAGS` > allowlist de terminales > off.
/// Función pura para poder testearla; `emoji_enabled` lee el entorno real.
pub fn decide_emoji(
    cli_override: Option<bool>,
    env_flag: Option<&str>,
    term_program: Option<&str>,
    term: Option<&str>,
) -> bool {
    if let Some(v) = cli_override {
        return v;
    }
    match env_flag {
        Some("1") => return true,
        Some("0") => return false,
        _ => {}
    }
    const ALLOWLIST: &[&str] = &["iTerm.app", "WezTerm", "ghostty", "Apple_Terminal"];
    if term_program.is_some_and(|tp| ALLOWLIST.contains(&tp)) {
        return true;
    }
    term.is_some_and(|t| t.contains("kitty"))
}

pub fn emoji_enabled(cli_override: Option<bool>) -> bool {
    decide_emoji(
        cli_override,
        std::env::var("WCTUI_FLAGS").ok().as_deref(),
        std::env::var("TERM_PROGRAM").ok().as_deref(),
        std::env::var("TERM").ok().as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_known_and_excluded_teams() {
        assert_eq!(flag_emoji("MEX").as_deref(), Some("🇲🇽"));
        assert_eq!(flag_emoji("RSA").as_deref(), Some("🇿🇦"));
        assert_eq!(flag_emoji("ENG"), None, "tag sequences excluidas");
        assert_eq!(flag_emoji("SCO"), None);
        assert_eq!(flag_emoji("WAL"), None);
        assert_eq!(flag_emoji("XYZ"), None);
    }

    #[test]
    fn policy_precedence() {
        // Override CLI gana a todo.
        assert!(decide_emoji(Some(true), Some("0"), None, None));
        assert!(!decide_emoji(
            Some(false),
            Some("1"),
            Some("iTerm.app"),
            None
        ));
        // Env gana a la allowlist.
        assert!(decide_emoji(None, Some("1"), None, None));
        assert!(!decide_emoji(None, Some("0"), Some("iTerm.app"), None));
        // Allowlist por TERM_PROGRAM o TERM.
        assert!(decide_emoji(None, None, Some("ghostty"), None));
        assert!(decide_emoji(None, None, None, Some("xterm-kitty")));
        // Default conservador: off.
        assert!(!decide_emoji(
            None,
            None,
            Some("vscode"),
            Some("xterm-256color")
        ));
        assert!(!decide_emoji(None, None, None, None));
    }
}
