## Why

La pantalla de referencia de Stitch muestra banderas junto a cada selección y hoy la TUI solo muestra nombres. Además, el proyecto se publicará como repo público en GitHub: usuarios con terminales heterogéneos (iTerm2, Windows Terminal, tmux, VS Code, consolas sin fuente emoji) lo usarán, así que las banderas no pueden romper el layout en ningún entorno — la estrategia es progressive enhancement, no emoji a secas.

## What Changes

- **Capa 0 (universal, siempre activa)**: cada equipo muestra su trigrama FIFA (`MEX`, `RSA`) junto al nombre en las filas con marcador y en la cabecera del detalle. Funciona en cualquier terminal, columnas perfectas garantizadas.
- **Capa 1 (banderas emoji, condicional)**: bandera emoji (🇲🇽) en lugar del trigrama cuando se sabe que el terminal las renderiza bien:
  - Mapa estático FIFA→ISO 3166-1 alpha-2 de los 48 clasificados al Mundial 2026.
  - Activación por allowlist de terminales conocidos (`TERM_PROGRAM`/`TERM`: iTerm2, kitty, Ghostty, WezTerm, Apple Terminal).
  - Override manual en ambas direcciones: flags CLI `--flags`/`--no-flags` y variable `WCTUI_FLAGS=0|1`, que ganan sobre la detección.
- Inglaterra/Escocia/Gales (sin ISO-2; sus emoji usan tag sequences mal soportadas) SIEMPRE usan trigrama, incluso con banderas activas.
- Los iconos emoji existentes del timeline (⚽🟨🟥🔁) se acogen a la misma política: con emoji desactivado caen a marcadores de texto (`G`, `A`, `R`, `↔` o similares).

## Capabilities

### New Capabilities
- `country-flags`: identidad visual de selecciones (trigrama base + bandera emoji condicional), mapa FIFA→ISO de los 48 clasificados, y política de activación de emoji (allowlist + overrides) que también gobierna los iconos del timeline.

### Modified Capabilities

(Ninguna a nivel de specs principales — `match-list-ui` y `match-detail-ui` aún viven en el cambio `world-cup-tui` sin archivar; este cambio añade la capacidad transversal nueva sin alterar sus requisitos ya escritos.)

## Impact

- **Código**: nuevo módulo (p. ej. `src/flags.rs` o `src/ui/flags.rs`) con el mapa y la política de activación; toques de render en `ui/list.rs` y `ui/detail.rs`; parsing de args/env en `main.rs`.
- **Dependencias**: ninguna nueva (args simples a mano o `std::env`).
- **Compatibilidad**: el contrato público del repo es que la Capa 0 se ve perfecta en cualquier terminal; las banderas son opt-in automático solo en terminales verificados.
