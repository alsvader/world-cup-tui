## Why

El panel FINALIZADOS solo muestra partidos de hoy y ayer por diseño (dashboard del día). Durante el torneo el usuario quiere consultar resultados de jornadas anteriores sin salir de la TUI, pero cargándolos bajo demanda para no inflar el polling ni el payload habitual.

## What Changes

- Mantener el comportamiento por defecto: hoy (todos los estados) + finalizados de ayer en el poll automático.
- Nueva acción **jornada anterior** con tecla `p`: carga un día calendario más atrás desde ESPN, anexa solo partidos finalizados a la lista y deduplica por id.
- Panel FINALIZADOS con **altura fija**, scroll interno (mismo patrón que el timeline del detalle) e indicador de filas ocultas.
- **Separadores por jornada** dentro del panel (p. ej. `─── VIE 13 JUN ───`) agrupando filas por fecha local de kickoff.
- Etiquetas de fila: `FT` (hoy), `AYER` (ayer), fecha abreviada para jornadas más antiguas.
- Deshabilitar `p` al llegar al inicio del torneo (11 jun 2026) o mientras una carga histórica está en curso.
- Footer y README: documentar `[p] JORNADA ANTERIOR`.

## Capabilities

### New Capabilities

(Ninguna — el alcance extiende capacidades existentes.)

### Modified Capabilities

- `match-data`: fetch puntual por fecha para historial; merge de partidos finalizados sin duplicados; sin ampliar el rango del poll periódico.
- `match-list-ui`: scroll en FINALIZADOS, separadores por jornada, tecla `p`, etiquetas de fecha ampliadas.

## Impact

- **Código**: `espn.rs` (fetch por día), `app.rs` (estado de historial, scroll del panel), `ui/list.rs` (layout fijo + separadores), `main.rs` (tecla `p`, comando al poller).
- **Specs**: deltas en `match-data` y `match-list-ui`; `live-refresh` sin cambio de requisitos.
- **API externa**: una petición extra scoreboard por cada pulsación de `p` (datos estáticos, sin re-poll).
- **No breaking**: el dashboard por defecto y las teclas existentes se mantienen; `j`/`k` en lista siguen moviendo la selección global.
