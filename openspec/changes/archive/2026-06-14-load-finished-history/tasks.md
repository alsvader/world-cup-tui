## 1. Capa de datos (espn)

- [x] 1.1 Añadir `fetch_scoreboard_day(date: NaiveDate)` con rango de un día y filtro a finalizados (fecha local)
- [x] 1.2 Definir constante o helper para límite del torneo (2026-06-11)
- [x] 1.3 Tests: día con partidos, día vacío, rechazo antes del inicio del torneo

## 2. Estado y merge (app)

- [x] 2.1 Campos `earliest_loaded`, `history_loading`, `finished_scroll`, `finished_max_offset`
- [x] 2.2 `merge_matches`: dedupe por id; poll actualiza entradas existentes
- [x] 2.3 `apply_history_matches` y actualización de `earliest_loaded` (incl. día sin partidos)
- [x] 2.4 `finished_scroll_up/down` y `ensure_finished_visible` para la selección global
- [x] 2.5 Tests unitarios de merge y límites de historial

## 3. Poller y teclas (main)

- [x] 3.1 `Cmd::LoadPreviousJornada` y manejo en poller (fetch + `DataMsg` de historial)
- [x] 3.2 Tecla `p` en vista lista → enviar comando; ignorar si loading o en límite
- [x] 3.3 Inicializar `earliest_loaded` a ayer al primer scoreboard exitoso

## 4. UI lista (FINALIZADOS)

- [x] 4.1 Layout: altura acotada para FINALIZADOS (`Min`/`Length`); EN VIVO y PRÓXIMOS dinámicos
- [x] 4.2 Construir líneas con separadores por jornada (fecha abreviada español, orden descendente)
- [x] 4.3 Scroll interno con `scroll_window` e indicadores de filas ocultas
- [x] 4.4 Etiquetas en `score_row`: FT / AYER / fecha abreviada para jornadas antiguas
- [x] 4.5 Footer: `[P] JORNADA ANTERIOR`

## 5. Documentación y calidad

- [x] 5.1 Actualizar README (tecla `p`, comportamiento del historial)
- [x] 5.2 `cargo test`, `cargo clippy`, `cargo fmt --check`
