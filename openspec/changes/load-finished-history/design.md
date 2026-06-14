## Context

El dashboard muestra EN VIVO / PRÓXIMOS / FINALIZADOS con datos filtrados a hoy + finalizados de ayer (`filter_relevant` en `espn.rs`). El layout de lista asigna altura dinámica a cada panel según número de filas; el detalle ya implementa scroll con `scroll_window` y sticky bottom. El poller tokio es el único punto de I/O de red.

## Goals / Non-Goals

**Goals:**

- Cargar bajo demanda una jornada calendario anterior (día local) de partidos finalizados y anexarlos al estado existente.
- Panel FINALIZADOS con altura acotada, scroll interno y separadores visuales por jornada.
- Tecla `p` (previous) en vista lista; sin interferir con `j`/`k` de selección global.
- Reutilizar `scroll_window` y el patrón de indicadores del timeline del detalle.

**Non-Goals:**

- Re-polling de jornadas históricas (datos estáticos tras la carga).
- Historial completo automático al arranque.
- Scroll independiente en EN VIVO o PRÓXIMOS.
- Cambiar el rango del fetch periódico del scoreboard.

## Decisions

### D1: Estado de historial en `App`

- `earliest_loaded: NaiveDate` — día calendario local más antiguo ya incluido en `matches` (inicialmente ayer).
- `history_loading: bool` — bloquea `p` mientras hay fetch en curso.
- `finished_scroll: Option<usize>` y `finished_max_offset: usize` — paralelos a `timeline_scroll`.

Rationale: separar “ventana de poll” (hoy+ayer) de “historial acumulado” evita re-fetch innecesario.

### D2: Fetch puntual por día en `espn::Client`

Nuevo método `fetch_scoreboard_day(date: NaiveDate) -> Result<Vec<Match>>`:

- URL: `scoreboard?dates=YYYYMMDD-YYYYMMDD` (un solo día).
- Tras parse: filtrar solo `MatchStatus::Finished` con kickoff en ese día local.
- No pasar por `filter_relevant` (esa función sigue siendo la del poll habitual).

Alternativa descartada: ampliar el rango del poll → infla cada 30s con datos que no cambian.

### D3: Merge en el poller vía nuevo `Cmd::LoadPreviousJornada`

Flujo:

1. UI envía `Cmd::LoadPreviousJornada` al poller (tecla `p`).
2. Poller calcula `target = earliest_loaded - 1 día`; si `target < 2026-06-11`, ignorar.
3. Fetch del día, merge en `DataMsg::HistoryMatches(Vec<Match>)` o extensión de `DataMsg::Matches` con flag.
4. `App::apply` deduplica por `match.id`, inserta nuevos, actualiza `earliest_loaded`.

Si el día no tiene partidos finalizados: avanzar `earliest_loaded` igual (día de descanso) sin filas nuevas; opcional mensaje breve en barra de error/info.

### D4: Layout del panel FINALIZADOS

- EN VIVO y PRÓXIMOS: altura dinámica (como hoy).
- FINALIZADOS: `Constraint::Min(4)` o altura fija razonable (p. ej. 8–12 líneas de contenido) con el resto del espacio vertical asignado ahí cuando hay historial.

Render:

1. Construir lista de líneas: separador de jornada + filas de partido, orden descendente por fecha (hoy arriba, más antiguo abajo) o ascendente con scroll — **decisión: más reciente arriba** (coherente con dashboard).
2. Aplicar `scroll_window` sobre el total de líneas del panel.
3. Al mover selección con `j`/`k`, auto-desplazar scroll para mantener la fila seleccionada visible (`ensure_selection_visible`).

Separador: línea centrada `─── VIE 13 JUN ───` con `theme::muted()`, sin seleccionable.

### D5: Etiquetas en `score_row`

| Condición (fecha local vs hoy) | Etiqueta izquierda |
|---|---|
| Hoy | `FT` (o `status_detail`) |
| Ayer | `AYER` |
| Antes de ayer | `VIE 13/06` (día abreviado español, `%a %d/%m`) |

### D6: Teclas y footer

- Vista lista: `p` → `LoadPreviousJornada`; ignorar si `history_loading` o en límite del torneo.
- Footer lista: añadir `[P] JORNADA ANTERIOR`.
- README: documentar `p`.

## Risks / Trade-offs

- **[Riesgo] Selección global con muchas filas finished** → Mitigation: scroll del panel sigue la selección; separadores no son ítems seleccionables.
- **[Riesgo] Día sin partidos** → Mitigation: avanzar `earliest_loaded` sin UI bloqueada; usuario puede pulsar `p` de nuevo.
- **[Riesgo] Duplicados tras merge** → Mitigation: dedupe estricto por `id` antes de insertar.
- **[Trade-off] Altura fija del panel** → En terminales muy pequeñas el panel puede ser corto; `Min(4)` garantiza usabilidad mínima.

## Migration Plan

Cambio incremental en rama `load-finished-history`; sin migración de datos. Rollback: revertir branch.

## Open Questions

(Ninguna — tecla `p`, separadores por jornada y carga incremental acordados.)
