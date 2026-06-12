# world-cup-tui

TUI en Rust para seguir los partidos del Mundial 2026 en vivo desde la terminal: marcador, minuto, goles (autor y momento), tarjetas y datos del partido, con actualización automática.

```
 WORLD CUP TUI                                                12 JUN 2026 · 13:45
┌ ● EN VIVO ─────────────────────────────────────────────────── [1 ACTIVOS] ┐
│ 62'        CANADA 🇨🇦   [ 1 - 0 ]  🇧🇦 BOSNIA-HERZEGOVINA   BMO Field      │
└────────────────────────────────────────────────────────────────────────────┘
┌ ○ PRÓXIMOS ─────────────────────────────────────────────── [HOY · UTC-6] ┐
│ HORA     PARTIDO                              SEDE                        │
│ 19:00    UNITED STATES vs PARAGUAY            SoFi Stadium · Inglewood    │
└────────────────────────────────────────────────────────────────────────────┘
 [Q] SALIR · [R] REFRESCAR · [J/K] NAVEGAR · [ENTER] DETALLE
```

## Uso

```bash
cargo build --release
./target/release/world-cup-tui
```

| Tecla | Acción |
|---|---|
| `↑↓` / `j k` | Mover selección |
| `Enter` | Detalle del partido (goles, tarjetas, cambios en vivo) |
| `Esc` | Volver a la lista |
| `r` | Refrescar ahora |
| `q` | Salir |

## Banderas y emoji

El layout base usa trigramas FIFA (`MEX`, `BRA`) y marcadores de texto — se ve perfecto en **cualquier** terminal. Las banderas emoji (🇲🇽) y los iconos (⚽🟨🟥) se activan automáticamente solo en terminales que las renderizan bien (iTerm2, kitty, Ghostty, WezTerm, Apple Terminal).

Override manual, que siempre gana a la detección:

```bash
world-cup-tui --flags      # forzar banderas/emoji
world-cup-tui --no-flags   # forzar modo texto
WCTUI_FLAGS=1|0            # equivalente por variable de entorno
```

Inglaterra, Escocia y Gales siempre usan trigrama: sus banderas emoji ("tag sequences") se renderizan mal en demasiados terminales.

## Datos

Los datos provienen de la API JSON pública **no documentada** de ESPN (la misma que usa espn.com). No requiere API key. Al ser no oficial, podría cambiar sin aviso; la app degrada con elegancia ante errores (conserva los últimos datos y reintenta).

Polling: scoreboard cada ~30s, partido abierto cada ~15s, relajado a ~120s sin partidos en vivo.
