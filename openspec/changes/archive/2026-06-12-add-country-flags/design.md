## Context

La TUI se publicará como repo público; el renderizado de emoji varía por emulador, fuente y multiplexor (tmux), y no existe detección por software confiable del ancho real que un terminal pinta para un emoji de bandera (regional indicators). La pantalla de Stitch (referencia visual) muestra banderas junto a las selecciones. ESPN entrega `abbreviation` (trigrama FIFA) por equipo, no código ISO-2.

## Goals / Non-Goals

**Goals:**
- Banderas emoji donde se vean bien; trigrama FIFA en todos lados como base universal.
- Cero columnas rotas en cualquier terminal con la configuración por defecto.
- Política de emoji única y centralizada que gobierne banderas e iconos del timeline.

**Non-Goals:**
- Logos PNG vía protocolos gráficos de terminal (kitty graphics, sixel) — complejidad desproporcionada.
- Detección por sondeo de cursor (`CSI 6n`) — anotada como mejora futura, no v1.
- Banderas de tag sequences (Inglaterra/Escocia/Gales) — siempre trigrama.

## Decisions

### D1: Progressive enhancement con dos capas
Capa 0: trigrama FIFA (3 chars, ancho fijo, ASCII) siempre presente junto al nombre. Capa 1: bandera emoji en lugar del trigrama solo si la política de emoji lo permite. El formato de columnas reserva ancho fijo para el slot (3 chars trigrama o 2 celdas emoji + padding), de modo que activar/desactivar banderas no mueve las demás columnas.

### D2: Mapa estático FIFA→ISO de los 48 clasificados
`const` en código: `("MEX","MX"), ("USA","US"), ...`. Equipos sin entrada o sin ISO-2 (ENG/SCO/WAL) → trigrama. Sin dependencia externa ni red. Racional: el universo es cerrado y conocido (48 equipos, torneo de 6 semanas).

### D3: Política de activación — allowlist + override, centralizada
Un solo punto de verdad (`emoji_enabled()`):
1. `WCTUI_FLAGS=1|0` o `--flags`/`--no-flags` → gana siempre.
2. Si no hay override: ON si `TERM_PROGRAM` ∈ {iTerm.app, ghostty, WezTerm, Apple_Terminal, vscode?*} o `TERM` contiene `kitty`. OFF en caso contrario (incluye tmux sin `TERM_PROGRAM` propagado, Windows Terminal, consolas desconocidas).
3. La misma política decide los iconos del timeline: emoji (⚽🟨🟥🔁) vs texto (`G`/`A`/`R`/`<>`).

*La allowlist exacta se ajusta empíricamente; VS Code queda fuera por defecto en v1 (renderizado dependiente de fuente).

*Alternativa considerada*: sondeo de ancho real imprimiendo un emoji y consultando posición de cursor al arrancar (técnica de yazi). Más preciso, pero agrega protocolo de terminal al arranque; queda como Open Question/mejora futura.

### D4: Sin nueva dependencia de CLI
Dos flags booleanos no justifican `clap`: parseo manual de `std::env::args` + `std::env::var`. Si el CLI crece (elegir liga, fecha), se reevalúa.

## Risks / Trade-offs

- **[Allowlist incompleta: terminal capaz sin banderas por default]** → Override documentado en README (`--flags`); costo de un falso negativo = estética, costo de un falso positivo = layout roto. Sesgo deliberado a falsos negativos.
- **[Emoji de bandera con ancho distinto a 2 en algún terminal de la allowlist]** → La allowlist solo incluye emuladores verificados; el slot de ancho fijo acota el daño a esa celda. Ajustable por issue de GitHub sin tocar diseño.
- **[tmux degrada a trigrama aunque el terminal exterior soporte emoji]** → Aceptado: dentro de tmux la detección es poco fiable; el usuario puede forzar `--flags`.

## Open Questions

- Sondeo de ancho por cursor (`CSI 6n`) como detección v2: ¿vale la pena tras feedback real del repo público?
- Marcadores de texto exactos para el timeline en Capa 0 (`G`/`A`/`R`/`<>` vs `[G]`/`[Y]`/`[R]`): definir al implementar viendo alineación.
