# World Cup TUI

TUI en terminal para seguir el Mundial 2026 en vivo: marcador, minuto, goles, tarjetas y
sustituciones con auto-refresh. Toda la planeación vive en OpenSpec: requisitos en
`openspec/specs/` (documentación viva por capability) y cambios activos en
`openspec/changes/`; contexto de stack y decisiones en `openspec/config.yaml`.

## Flujo de trabajo OpenSpec

- **Proponer** un cambio (proposal, design, specs, tasks): `/opsx:propose`
- **Explorar** ideas sin implementar: `/opsx:explore`
- **Implementar** tareas de un cambio: `/opsx:apply`
- **Sincronizar** delta specs a main specs: `/opsx:sync`
- **Archivar** un cambio completado: `/opsx:archive`

Skills equivalentes en `.cursor/skills/openspec-*` y `.claude/skills/openspec-*`.

## Flujo de trabajo de UI (obligatorio)

- `DESIGN.md` (raíz, formato Google Stitch) es la fuente de verdad visual: paleta,
  tipografía, densidad, semántica de colores y componentes definidos ahí NO se reinventan.
- Todos los colores de la TUI viven en `src/ui/theme.rs`, derivados de DESIGN.md.
- Estética Bloomberg-terminal / TUI: sin sombras, sin bordes redondeados, densidad alta,
  separadores de 1px, labels en mayúsculas.
- Interfaz en español (torneo en México, EE.UU. y Canadá).
- Banderas: trigramas FIFA como base universal; emoji solo con progressive enhancement
  (`flags.rs`, flags `--flags` / `--no-flags` / `WCTUI_FLAGS`).

## Reglas técnicas

- Rust (edition 2024) + ratatui + crossterm + tokio + reqwest + serde.
- **La UI nunca hace I/O.** Un task tokio hace networking y envía datos normalizados por
  canal `mpsc`; redes lentas no bloquean el teclado.
- **El JSON de ESPN se trata como hostil.** Deserialización permisiva en `espn.rs`;
  modelo garantizado en `model.rs`; tests con fixtures reales en `tests/fixtures/`.
- Polling escalonado: scoreboard ~30s en vivo, detalle del partido abierto ~15s,
  ~120s cuando no hay partidos en vivos.
- Lógica pura de estado (ordenación, columnas, scroll) en `app.rs`; vistas en `src/ui/`.
- Fuente de datos: API no documentada de ESPN (`site.api.espn.com`, liga `fifa.world`);
  sin API key; puede cambiar sin aviso — degradación elegante (últimos datos válidos +
  timestamp).
- Calidad: `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.
