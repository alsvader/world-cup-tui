# world-cup-tui

[![CI](https://github.com/OWNER/world-cup-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/OWNER/world-cup-tui/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

A terminal UI for following the **FIFA World Cup 2026** live: scores, match clock, goal scorers, yellow/red cards and substitutions — auto-refreshing while you work, in a Bloomberg-terminal-inspired dashboard.

![demo](assets/demo.gif?v=2)

*El demo muestra el dashboard, la carga de jornadas anteriores en FINALIZADOS con `p`, navegación en la lista ampliada y la vista de detalle.*

## Features

- **Live match tracking** — score and match clock update automatically (~30s for the dashboard, ~15s for the match you have open).
- **Three-panel dashboard** — LIVE (with active-match counter), UPCOMING (kickoff times in *your* timezone, venue and city) and FINISHED (today and yesterday by default; press `p` to load earlier matchdays incrementally).
- **Match centre detail view** — goals and cards in two columns, one per team, minute in the middle, in shared chronological order. Press `t` to see the full event feed (substitutions, delays, halftime).
- **Sticky-bottom scrolling** — follow a live match pinned to the latest event, or scroll up through history without being dragged down when new events arrive.
- **Country flags with progressive enhancement** — FIFA trigrams (`MEX`, `BRA`) everywhere as the universal base; flag emoji 🇲🇽 and event icons ⚽🟨🟥 automatically enabled on terminals known to render them well.
- **Resilient by design** — network errors never crash or blank the screen: the last good data stays visible with its timestamp, and the app reconnects by itself.

| Dashboard | Match detail |
|---|---|
| ![dashboard](assets/dashboard.png?v=2) | ![detail](assets/detail.png?v=2) |

*Screenshots show the universal text mode (works on any terminal). On iTerm2, kitty, Ghostty, WezTerm or Apple Terminal you get flag emoji and event icons automatically.*

## Installation

Requires [Rust](https://rustup.rs) (stable). A truecolor terminal is recommended for the exact design-system palette; any terminal works.

```bash
git clone https://github.com/OWNER/world-cup-tui.git
cd world-cup-tui
cargo build --release
./target/release/world-cup-tui
```

## Usage

| Key | Action |
|---|---|
| `↑↓` / `j k` | Move selection (list) / scroll timeline (detail) |
| `Enter` | Open match detail |
| `p` | Load previous matchday into FINISHED (one calendar day back) |
| `t` | Toggle timeline: goals & cards ⇄ full event feed |
| `Esc` | Back to the dashboard |
| `r` | Refresh now |
| `q` | Quit |

The interface language is Spanish — fitting, for a World Cup hosted in Mexico, the US and Canada.

### Flags & emoji

The base layout uses FIFA trigrams and colored text markers, guaranteed to align on **any** terminal. Flag emoji and event icons are enabled automatically when `TERM_PROGRAM`/`TERM` identifies a terminal that renders them correctly. Manual override always wins:

```bash
world-cup-tui --flags      # force emoji on
world-cup-tui --no-flags   # force universal text mode
WCTUI_FLAGS=1|0            # same, via environment
```

England, Scotland and Wales always use trigrams: their emoji flags ("tag sequences") render poorly on too many terminals.

## Data

Data comes from ESPN's public **undocumented** JSON API (the same backend espn.com uses). No API key required. Being unofficial, it may change without notice — the app degrades gracefully and the data layer is isolated in a single module ([`src/espn.rs`](src/espn.rs)) to make a source swap cheap.

Polling is deliberately gentle: ~30s for the scoreboard, ~15s for the open match, relaxed to ~120s when nothing is live.

This project is not affiliated with ESPN or FIFA.

## Architecture

```
src/
├── main.rs        # terminal setup, event loop, tokio poller (mpsc channel)
├── app.rs         # app state + pure logic (sorting, columns, scroll window)
├── espn.rs        # anti-corruption layer over the ESPN API
├── flags.rs       # FIFA→ISO map + emoji activation policy
├── model.rs       # guaranteed domain types (Match, KeyEvent, ...)
└── ui/            # ratatui views; all colors live in theme.rs (DESIGN.md)
```

Two design decisions worth knowing:

- **The UI thread never does I/O.** A tokio task owns all networking and sends normalized data through a channel; slow networks can't freeze the keyboard.
- **The ESPN JSON is treated as hostile.** Deserialization structs are fully permissive (`Option` + defaults); a guaranteed internal model is built immediately, and tests run against real fixtures captured from the tournament ([`tests/fixtures/`](tests/fixtures/)).

The functional requirements live as living documentation in [`openspec/specs/`](openspec/specs/), maintained with [OpenSpec](https://github.com/Fission-AI/OpenSpec): every capability (match data, dashboard, detail view, live refresh, country flags) has its requirements and acceptance scenarios written down, and every change that shaped the project is archived under `openspec/changes/archive/`.

## Development

```bash
cargo test                                # unit + fixture-based tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

To regenerate the README captures (ideally during a live match), install [vhs](https://github.com/charmbracelet/vhs) and run:

```bash
vhs assets/demo.tape
```

## License

[MIT](LICENSE) © 2026 Aarón López Sosa
