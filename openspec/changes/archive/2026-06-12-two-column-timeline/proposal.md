## Why

El timeline del detalle mezcla lo esencial (goles, tarjetas) con ruido (cambios, pausas de hidratación, medio tiempo) en una sola lista cronológica, y cuando no cabe en pantalla se trunca por arriba sin forma de ver el resto. El usuario quiere leer el partido de un vistazo por equipo, como en un "match centre" clásico.

## What Changes

- **Timeline en dos columnas (estilo match centre)**: una fila por evento en orden cronológico, minuto al centro, cada evento en la columna de su equipo (local izquierda, visitante derecha), con encabezado fijo de equipos.
- **Vista por defecto filtrada**: solo goles y tarjetas (amarillas/rojas). Tecla `t` alterna a la vista TODO (lista completa actual con cambios y demás eventos), y de regreso.
- **Scroll con sticky bottom**: `↑↓`/`j k` scrollean el timeline; si el usuario está al fondo, los eventos nuevos lo mantienen al fondo (modo seguir-en-vivo); si scrolleó hacia arriba, no se le arrastra y un indicador discreto avisa que hay eventos nuevos abajo. Indicadores de overflow arriba/abajo.
- El encabezado del partido, frescura y navegación (Esc/q/r) no cambian.

## Capabilities

### New Capabilities

(Ninguna.)

### Modified Capabilities

- `match-detail-ui`: el requisito "Línea de tiempo de eventos clave" cambia a presentación en dos columnas por equipo con filtro por defecto a goles/tarjetas; se agregan requisitos de toggle de vista completa y de scroll con sticky bottom.

## Impact

- **Código**: `src/ui/detail.rs` (layout de dos columnas, render del filtro), `src/app.rs` (estado de scroll, modo de vista, asignación de eventos a equipo), `src/main.rs` (teclas ↑↓/jk/t en la vista de detalle).
- **Datos**: sin cambios en `espn.rs`/`model.rs` — los eventos ya traen `team`; la asignación a columna es por comparación con `home.name`/`away.name`.
- **Riesgo conocido**: en autogoles ESPN podría acreditar el evento al equipo beneficiado en vez del equipo del jugador; pendiente de verificar con fixture real cuando ocurra (degradación aceptable).
