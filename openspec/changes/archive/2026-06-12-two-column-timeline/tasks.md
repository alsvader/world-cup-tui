## 1. Estado y teclas

- [x] 1.1 `app.rs`: estado `TimelineMode { Key, All }` (default Key) y `timeline_scroll: Option<usize>` (None = pegado al fondo); reset de ambos al abrir/cerrar detalle
- [x] 1.2 `app.rs`: asignación de evento a columna (`Home`/`Away`/`Neutral`) comparando `event.team` con los nombres del match, y filtro de eventos según el modo
- [x] 1.3 `main.rs`: en vista detalle, `t` alterna modo (resetea scroll), `↑↓`/`j k` mueven el scroll con semántica sticky-bottom

## 2. Render de dos columnas

- [x] 2.1 `ui/detail.rs`: encabezado fijo de columnas con identidad de equipos (slots de country-flags), título del panel con modo y tecla (`EVENTOS — GOLES Y TARJETAS [T: TODO]`)
- [x] 2.2 `ui/detail.rs`: filas cronológicas de tres zonas (columna local | minuto | columna visitante), truncado con elipsis al ancho de columna, eventos neutrales centrados solo en vista TODO
- [x] 2.3 `ui/detail.rs`: ventana de scroll con clamp del offset en render e indicadores `▲ n` / `▼ n` de overflow

## 3. Verificación

- [x] 3.1 Tests unitarios de la lógica pura: asignación a columna (home/away/neutral), filtro por modo, y ventana de scroll (sticky bottom, clamp, indicadores)
- [x] 3.2 `cargo test` + `cargo clippy` limpios
- [x] 3.3 Corrida en pty contra un partido real: vista por defecto con goles/tarjetas en su columna, toggle `t`, scroll hasta el primer evento y regreso al fondo
