## Context

Proyecto greenfield: una TUI en Rust para seguir en vivo el Mundial 2026 desde la terminal. El torneo ya está en curso (11 jun – 19 jul 2026), por lo que la velocidad de entrega importa: cada día sin la herramienta es un día de torneo perdido.

La fuente de datos fue verificada durante la exploración contra partidos reales del torneo: la API no documentada de ESPN responde sin autenticación y contiene todo lo necesario.

- `GET https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/scoreboard` → partidos del día: equipos, marcador, estado (`pre`/`in`/`post`), minuto (`status.displayClock`), sede.
- `GET https://site.api.espn.com/apis/site/v2/sports/soccer/fifa.world/summary?event=<id>` → `keyEvents`: goles (con autor, minuto y tipo, p. ej. "Goal - Header"), tarjetas amarillas/rojas, cambios, pausas.

Decisiones ya tomadas con el usuario en exploración: Rust + Ratatui; alcance v1 = lista de partidos del día + detalle de partido; sin notificaciones del sistema.

## Goals / Non-Goals

**Goals:**
- TUI estable que se pueda dejar abierta en un pane de tmux durante un partido completo sin romperse.
- Datos en vivo: marcador, minuto, goles (quién y cuándo), tarjetas amarillas/rojas, datos básicos (sede, estado, hora de inicio).
- Refresh automático sin parpadeos ni bloqueos de UI.
- Resiliencia: errores de red o JSON inesperado nunca tiran la app; se conserva el último estado válido.

**Non-Goals:**
- Notificaciones del sistema (explícitamente fuera de v1).
- Tabla de grupos, bracket, calendario completo del torneo, estadísticas avanzadas (xG, posesión), alineaciones.
- Soporte de otras ligas/torneos (aunque la liga `fifa.world` es un parámetro trivial de generalizar después).
- Empaquetado/distribución (Homebrew, crates.io); con `cargo build --release` basta para uso personal.

## Decisions

### D1: Rust + Ratatui + Crossterm
Elección del usuario. Ratatui es el framework TUI estándar en Rust (sucesor de tui-rs), con render por frames inmediatos que encaja bien con datos que cambian cada pocos segundos. Crossterm como backend por ser multiplataforma y el default de facto.

*Alternativas consideradas*: Go + Bubble Tea (más rápido de desarrollar), Python + Textual (más vistoso out-of-the-box). Descartadas por preferencia del usuario; Rust además da binario único sin runtime.

### D2: Arquitectura — UI síncrona + poller async con canal mpsc
Dos mundos separados:
- **Hilo de UI**: loop de render + input con `crossterm::event::poll` (tick ~250ms). Nunca hace I/O de red.
- **Tarea de datos (tokio)**: hace los fetches en su propio runtime y envía resultados normalizados por un `tokio::sync::mpsc` hacia el estado de la app.

La UI drena el canal en cada tick (`try_recv`) y repinta. Esto garantiza que la latencia de red jamás congela el teclado ni el render.

*Alternativa considerada*: todo async (UI incluida) con `EventStream`. Funciona, pero mezcla concerns y complica el manejo de input; el patrón canal es el idiomático en apps Ratatui con datos remotos.

### D3: Capa anticorrupción — módulo `espn` aislado del modelo propio
El JSON de ESPN es grande e irregular: campos que aparecen/desaparecen según el estado del partido. Por eso:
- Structs de deserialización (`espn::raw`) totalmente permisivos: `Option<T>` y `#[serde(default)]` en todo campo no garantizado.
- Conversión inmediata a un modelo propio y garantizado: `Match`, `MatchStatus` (enum `Scheduled`/`Live`/`HalfTime`/`Finished`), `KeyEvent` (enum `Goal { player, minute, kind }`, `Card { player, minute, color }`, `Substitution`, `Other`).
- El resto de la app solo conoce el modelo propio. Si ESPN cambia su JSON, solo se toca `espn.rs`.

Los tipos de evento se mapean por el campo `type.text` de `keyEvents` ("Goal", "Goal - Header", "Penalty - Scored", "Yellow Card", "Red Card", "Substitution"...); cualquier tipo no reconocido cae en `Other` y se muestra como texto plano en lugar de descartarse.

### D4: Polling escalonado por vista
- Scoreboard: cada 30s siempre (alimenta la lista y los marcadores).
- Summary: solo del partido actualmente abierto en detalle, cada 15s; se cancela al salir de la vista.
- Si el scoreboard no reporta ningún partido `in` (en vivo), ambos ritmos se relajan a 120s.
- Refresh manual con tecla `r` para impaciencia humana.

*Racional*: minimiza llamadas a una API ajena y gratuita (buen ciudadano) sin sacrificar frescura donde importa. No hay rate limit documentado, pero ESPN sirve esto a su propio sitio masivamente; a estas cadencias el riesgo es nulo.

### D5: Manejo de errores — degradar, nunca morir
- Todo fetch devuelve `Result`; un error se convierte en un mensaje de estado ("sin conexión — reintentando"), nunca en panic ni en pantalla vacía.
- El estado de la app conserva los últimos datos válidos con su timestamp; la UI muestra "actualizado hace Xs" para que la frescura sea visible.
- `color_eyre`/`anyhow` + un panic hook que restaura la terminal (raw mode off, pantalla alternativa) antes de morir, para no dejar la terminal rota en el peor caso.

### D7: Sistema de diseño — `DESIGN.md` en la raíz es la fuente de verdad visual
Todas las decisiones visuales siguen el sistema de diseño del archivo `DESIGN.md` (generado con Google Stitch), traducido al terminal:
- **Paleta**: fondo `surface #141313`, texto `on-surface #e5e2e1`, bordes `outline-variant #47464a`, texto secundario `outline #919095`, vía `Color::Rgb` (requiere terminal truecolor).
- **Semántica**: EN VIVO verde vibrante; PRÓXIMOS azul; FINALIZADOS gris desaturado; goles esmeralda (distinto del verde live); indicador de polling/frescura cyan técnico; errores `#ffb4ab`.
- **Estética**: sin adornos, bordes rectos (0 radius), labels y títulos de panel en MAYÚSCULAS, badge "● EN VIVO" con punto parpadeante, densidad alta.
- Centralizado en `src/ui/theme.rs`; ningún color hardcodeado fuera de ese módulo.

### D6: Estructura del crate
```
src/
├── main.rs        # setup terminal, spawn poller, loop principal
├── app.rs         # estado: vista actual, partidos, selección, frescura
├── espn.rs        # cliente HTTP + structs raw + normalización → modelo
├── model.rs       # Match, MatchStatus, KeyEvent, Team — tipos garantizados
└── ui/
    ├── mod.rs     # dispatch por vista
    ├── list.rs    # lista de partidos del día
    └── detail.rs  # detalle: marcador, minuto, timeline de eventos
```
Binario único, sin workspace. Tests de normalización con fixtures JSON reales capturados de la API (se guardan en `tests/fixtures/`).

## Risks / Trade-offs

- **[API no documentada cambia o desaparece]** → Capa anticorrupción (D3) concentra el impacto en un módulo; fixtures reales en tests detectan rupturas de parseo de inmediato. Plan B identificado en exploración: openfootball JSON (fixtures sin minuto a minuto) o APIs de pago.
- **[Campos del JSON ausentes en estados no observados aún (prórroga, penales, suspensión)]** → Todo opcional + variante `Other`/fallback en enums; cuando lleguen las eliminatorias (prórroga/penales) puede requerir un ajuste menor de mapeo. Riesgo aceptado y acotado.
- **[Reloj del partido solo avanza al ritmo del polling]** → El minuto mostrado puede ir hasta ~30s detrás. Aceptable para v1; el indicador "actualizado hace Xs" lo hace transparente.
- **[Hora local vs UTC]** → ESPN devuelve fechas en UTC ("partidos del día" puede partir mal a medianoche). Se convierte a hora local con `chrono` para agrupar y mostrar.
- **[Vida útil corta del proyecto]** → Se acepta deuda deliberada: sin configurabilidad, sin i18n, liga hardcodeada. Trade-off explícito a favor de entregar durante el torneo.

## Open Questions

- Comportamiento exacto del feed en prórroga y penales (no observable hasta la fase eliminatoria, a partir del 28 de junio aprox.): el mapeo de `keyEvents` podría necesitar variantes nuevas. Se resolverá con un fixture real cuando ocurra el primer partido con prórroga.
