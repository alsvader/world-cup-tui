## Why

El Mundial 2026 está en curso (11 de junio – 19 de julio de 2026) y el usuario pasa la mayor parte del tiempo en la terminal. Quiere seguir los partidos en vivo —marcador, minuto, goles y tarjetas— sin salir de su entorno de trabajo. No existe hoy una herramienta propia que lo haga; la ventana de utilidad es ahora, durante el torneo.

## What Changes

- Se crea desde cero una TUI en Rust + Ratatui (`world-cup-tui`), proyecto nuevo en este repositorio.
- Vista de **lista de partidos del día**: partidos en vivo, próximos y finalizados, con marcador y minuto, agrupados por estado.
- Vista de **detalle de partido**: marcador grande, minuto en curso, estado (en vivo / medio tiempo / final), sede, y línea de tiempo de eventos clave: goles (autor, minuto, tipo), tarjetas amarillas y rojas, cambios.
- **Capa de datos** sobre la API no documentada de ESPN (`site.api.espn.com`, liga `fifa.world`): gratuita, sin API key, verificada con datos reales del torneo. Se normaliza a un modelo propio para aislar a la app de cambios en la fuente.
- **Auto-refresh** por polling escalonado: scoreboard ~30s, detalle del partido abierto ~15s; ritmo relajado cuando no hay partidos en vivo.
- Sin notificaciones del sistema en v1 (decisión explícita; posible mejora futura).

## Capabilities

### New Capabilities
- `match-data`: obtención y normalización de datos del Mundial desde la API de ESPN — partidos del día, marcador, minuto, estado, y eventos clave (goles, tarjetas, cambios) — con tolerancia a fallos de red y a campos ausentes/irregulares del JSON.
- `match-list-ui`: vista TUI de los partidos del día agrupados por estado (en vivo / próximos / finalizados), con navegación por teclado y selección de partido.
- `match-detail-ui`: vista TUI de detalle de un partido con marcador, minuto, datos básicos y línea de tiempo de eventos; indicador de frescura de datos ("actualizado hace Xs").
- `live-refresh`: actualización automática de datos en segundo plano sin bloquear la UI, con cadencia diferenciada por vista y degradación elegante ante errores de red (se conservan los últimos datos válidos).

### Modified Capabilities

(Ninguna — proyecto nuevo, no hay specs existentes.)

## Impact

- **Código**: proyecto Rust nuevo (binario `world-cup-tui`), estructura `src/` con módulos de datos (espn), modelo, app/estado y UI.
- **Dependencias**: `ratatui`, `crossterm`, `tokio`, `reqwest`, `serde`/`serde_json`, `anyhow`.
- **Sistemas externos**: API no documentada de ESPN. Riesgo: puede cambiar sin aviso; mitigado aislando la deserialización en un solo módulo y manejando todos los campos como opcionales. Sin costo ni registro.
- **Vida útil**: herramienta de torneo (~6 semanas de uso intensivo); se prioriza velocidad de entrega y robustez de lectura sobre generalidad.
