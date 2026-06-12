## 1. Setup del proyecto

- [x] 1.1 `cargo init` del binario `world-cup-tui` con dependencias: ratatui, crossterm, tokio (rt-multi-thread, macros, sync, time), reqwest (json), serde, serde_json, chrono, anyhow
- [x] 1.2 Capturar fixtures JSON reales de la API (scoreboard y summary de un partido con goles y tarjetas) en `tests/fixtures/`

## 2. Capa de datos (match-data)

- [x] 2.1 `model.rs`: tipos propios garantizados — `Match`, `Team`, `MatchStatus` (Scheduled/Live/HalfTime/Finished), `KeyEvent` (Goal/Card/Substitution/Other)
- [x] 2.2 `espn.rs`: structs raw permisivos (`Option` + `serde(default)`) para scoreboard y summary
- [x] 2.3 `espn.rs`: cliente HTTP con timeout + funciones `fetch_scoreboard()` y `fetch_summary(event_id)` que devuelven `Result` (sin panics)
- [x] 2.4 `espn.rs`: normalización raw → modelo propio, mapeando tipos de `keyEvents` por `type.text` con fallback a `Other`
- [x] 2.5 Tests de normalización contra los fixtures: estados, marcador, minuto, goles con autor/minuto/tipo, tarjetas con color, campos ausentes

## 3. Esqueleto de la app y refresh (live-refresh)

- [x] 3.1 `main.rs`: setup/teardown de terminal (raw mode, pantalla alternativa) + panic hook que restaura la terminal
- [x] 3.2 `app.rs`: estado de la app — vista actual, partidos, selección, último summary, timestamp de última actualización, mensaje de error discreto
- [x] 3.3 Poller en tokio con canal mpsc: scoreboard cada 30s, summary del partido abierto cada 15s, cadencia 120s si no hay partidos en vivo; comandos UI→poller (abrir/cerrar detalle, refresh manual)
- [x] 3.4 Loop principal: tick ~250ms, drenar canal, manejar input, render; errores de fetch conservan datos previos y setean aviso de reconexión

## 4. Vista de lista (match-list-ui)

- [x] 4.1 `ui/list.rs`: render de secciones EN VIVO / PRÓXIMOS / FINALIZADOS con marcador+minuto, hora local (chrono) o marcador final según estado
- [x] 4.2 Navegación: ↑↓/jk para selección, Enter abre detalle, q sale; mensaje claro cuando no hay partidos hoy
- [x] 4.3 Finalizados del día anterior visibles con marcador "AYER" (fetch con rango de fechas + filtro client-side por fecha local; alcance ampliado tras prueba del usuario)

## 5. Vista de detalle (match-detail-ui)

- [x] 5.1 `ui/detail.rs`: cabecera con equipos, marcador, minuto (incl. tiempo añadido), estado, sede; hora de inicio si está programado
- [x] 5.2 Timeline de eventos en orden cronológico con iconografía: ⚽ goles (autor, tipo), 🟨/🟥 tarjetas, 🔁 cambios, texto plano para `Other`
- [x] 5.3 Indicador "actualizado hace Xs" + Esc vuelve a la lista conservando selección

## 6. Verificación end-to-end

- [x] 6.1 `cargo test` + `cargo clippy` limpios
- [x] 6.2 Prueba manual contra el torneo en vivo: lista del día correcta en hora local, detalle de un partido en vivo refrescando solo, recuperación tras cortar la red (validado por el usuario el 12 jun 2026 durante la jornada 2)
