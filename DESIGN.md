---
name: Pitch Deck TUI
colors:
  surface: "#141313"
  surface-dim: "#141313"
  surface-bright: "#3a3939"
  surface-container-lowest: "#0e0e0e"
  surface-container-low: "#1c1b1b"
  surface-container: "#201f1f"
  surface-container-high: "#2b2a2a"
  surface-container-highest: "#353434"
  on-surface: "#e5e2e1"
  on-surface-variant: "#c8c5ca"
  inverse-surface: "#e5e2e1"
  inverse-on-surface: "#313030"
  outline: "#919095"
  outline-variant: "#47464a"
  surface-tint: "#c8c6c8"
  primary: "#c8c6c8"
  on-primary: "#313032"
  primary-container: "#09090b"
  on-primary-container: "#7a787b"
  inverse-primary: "#5f5e60"
  secondary: "#c8c6c9"
  on-secondary: "#303033"
  secondary-container: "#47464a"
  on-secondary-container: "#b6b4b8"
  tertiary: "#cec4c4"
  on-tertiary: "#352f2f"
  tertiary-container: "#0c0808"
  on-tertiary-container: "#7f7777"
  error: "#ffb4ab"
  on-error: "#690005"
  error-container: "#93000a"
  on-error-container: "#ffdad6"
  primary-fixed: "#e5e1e4"
  primary-fixed-dim: "#c8c6c8"
  on-primary-fixed: "#1c1b1d"
  on-primary-fixed-variant: "#474649"
  secondary-fixed: "#e4e1e5"
  secondary-fixed-dim: "#c8c6c9"
  on-secondary-fixed: "#1b1b1e"
  on-secondary-fixed-variant: "#47464a"
  tertiary-fixed: "#ebe0df"
  tertiary-fixed-dim: "#cec4c4"
  on-tertiary-fixed: "#1f1a1a"
  on-tertiary-fixed-variant: "#4c4545"
  background: "#141313"
  on-background: "#e5e2e1"
  surface-variant: "#353434"
typography:
  display-lg:
    fontFamily: JetBrains Mono
    fontSize: 32px
    fontWeight: "700"
    lineHeight: 40px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: JetBrains Mono
    fontSize: 20px
    fontWeight: "600"
    lineHeight: 28px
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: "400"
    lineHeight: 20px
  body-sm:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: "400"
    lineHeight: 18px
  label-caps:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: "700"
    lineHeight: 16px
    letterSpacing: 0.05em
  mono-data:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: "500"
    lineHeight: 16px
spacing:
  unit: 4px
  container-padding: 16px
  gutter: 1px
  panel-gap: 8px
  table-cell-padding: 12px
---

## Brand & Style

The design system is a high-density, performance-oriented interface designed for sports data professionals and power users. The aesthetic is a hybrid of a classic terminal interface and a modern SaaS dashboard—merging the authoritative, data-heavy layout of a Bloomberg terminal with the refined, minimalist execution seen in developer tools.

The brand personality is **Precise, Authoritative, and Live.** It avoids all decorative flourishes, focusing entirely on the speed of information delivery and clarity of state. There are no gradients, shadows, or rounded corners. The emotional response should be one of being "at the console"—full control, zero latency, and absolute focus.

## Colors

The color palette is strictly functional. The background is a deep, neutral black to ensure maximum contrast for data points. Borders and dividers use a subtle zinc tone to define structure without creating visual noise.

Semantic colors are used sparingly but with high impact:

- **Live:** A vibrant green indicates an active match or "On-Air" status.
- **Upcoming:** A clear blue for future events.
- **Finished:** A desaturated gray for historical data.
- **Updates:** An emerald tint specifically for goals or score changes to differentiate from general "Live" status.
- **Refresh:** A technical cyan used for data polling indicators or system pings.

## Typography

The system utilizes a dual-font strategy. **JetBrains Mono** is used for all headers, labels, and numerical data to reinforce the "Terminal" aesthetic and ensure perfect character alignment in tables. **Inter** is used for body text and descriptions to maintain high readability during long sessions.

Data density is prioritized; font sizes are generally smaller than consumer apps but balanced by generous line-heights and clear character distinction. All labels should be uppercase to evoke a command-line feel.

## Layout & Spacing

The layout follows a **Fixed Grid** model inspired by tiled window managers. Content is organized into "Panels" that occupy specific areas of the screen.

Instead of wide gutters, this design system uses 1px borders as the primary separator between elements, maximizing the "information per square inch."

- **Desktop:** A 12-column grid where panels typically span 3, 6, or 9 columns.
- **Mobile:** Panels stack vertically. Heavy use of horizontal scrolling for data tables.
- **Rhythm:** All spacing is based on a 4px baseline. Components are tightly packed with minimal internal padding to maintain the TUI (Text User Interface) feel.

## Elevation & Depth

Depth is communicated through **Tonal Layers** and **Borders** rather than shadows.

- **Level 0 (Background):** #09090b.
- **Level 1 (Panels):** #18181b with a 1px border of #27272a.
- **Active State:** Elements in focus or selected receive a 1px solid border of the Primary text color or a semantic color, with no change in elevation.

There are no blurs or transparency effects. The interface is purposefully flat and rigid, ensuring that the user’s eye is drawn to color changes (status) rather than physical depth.

## Shapes

The shape language is strictly **Sharp**. All corners have a 0px radius. This reinforces the technical, grid-based nature of the terminal aesthetic. Buttons, input fields, and panels must all be perfectly rectangular.

## Components

### Buttons

Buttons are rectangular with a 1px border. The "Primary" button uses a solid fill of the text color with inverted text. "Secondary" buttons use a ghost style (border only). On hover, buttons should invert their colors immediately with no transition timing.

### Data Tables

Tables are the core of the system. Use `1px` dividers between rows and columns. Header cells use `label-caps` typography with a subtle background tint (#18181b). Score columns must be monospaced and center-aligned for rapid scanning.

### Status Badges

Badges are small, rectangular tags. Use a "Live" badge with a blinking 8px dot for active matches. Use the semantic color for the text and a 10% opacity background of that same color.

### Input Fields

Inputs are simple 1px boxes. The cursor should be a solid block (mimicking a terminal cursor) rather than a thin line. Use `JetBrains Mono` for all input text.

### Panels

Every major section (Scores, Standings, Player Stats) is housed in a Panel. Panels must have a header bar containing the title in `label-caps` and any relevant utility actions (e.g., [REFRESH], [EXPAND]).
