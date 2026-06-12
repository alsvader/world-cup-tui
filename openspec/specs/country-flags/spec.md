# country-flags Specification

## Purpose

Identidad visual de las selecciones en la TUI: trigrama FIFA como base universal y bandera emoji como mejora condicional, con una política de emoji centralizada apta para terminales heterogéneos (repo público).

## Requirements

### Requirement: Trigrama FIFA como identidad base universal
Cada selección SHALL mostrarse con su trigrama FIFA (p. ej. `MEX`, `RSA`) junto al nombre en TODO lugar donde aparezca un partido: filas con marcador (paneles EN VIVO y FINALIZADOS), tabla de PRÓXIMOS y cabecera del detalle. El trigrama SHALL renderizarse con caracteres ASCII de ancho fijo, garantizando columnas alineadas en cualquier terminal.

#### Scenario: Terminal desconocido
- **WHEN** la TUI corre en un terminal fuera de la allowlist y sin override
- **THEN** todas las selecciones muestran trigrama y ningún emoji aparece en pantalla

### Requirement: Bandera emoji como mejora condicional
Cuando la política de emoji esté activa, el slot del trigrama SHALL mostrar la bandera emoji de la selección (regional indicators derivados de ISO 3166-1 alpha-2) usando un mapa estático FIFA→ISO de los 48 clasificados. El slot SHALL tener ancho fijo reservado de modo que alternar entre bandera y trigrama no desplace las demás columnas.

#### Scenario: Terminal de la allowlist
- **WHEN** la TUI corre en un terminal de la allowlist (p. ej. iTerm2, kitty, Ghostty)
- **THEN** las selecciones con ISO-2 muestran bandera emoji y las columnas permanecen alineadas

#### Scenario: Selección sin código ISO-2
- **WHEN** una selección no tiene ISO-2 (Inglaterra, Escocia, Gales) o no está en el mapa
- **THEN** muestra su trigrama incluso con la política de emoji activa (nunca tag sequences)

### Requirement: Política de activación de emoji centralizada
El sistema SHALL decidir el uso de emoji en un único punto, con esta precedencia: (1) override explícito por flag CLI `--flags`/`--no-flags` o variable `WCTUI_FLAGS=1|0`; (2) sin override, allowlist de terminales conocidos vía `TERM_PROGRAM`/`TERM`; (3) por defecto, emoji desactivado. La misma política SHALL gobernar los iconos del timeline de eventos (⚽🟨🟥🔁 vs marcadores de texto).

#### Scenario: Override manual gana a la detección
- **WHEN** el usuario pasa `--flags` en un terminal fuera de la allowlist
- **THEN** las banderas y los iconos emoji se activan

#### Scenario: Desactivación manual
- **WHEN** el usuario pasa `--no-flags` (o `WCTUI_FLAGS=0`) en un terminal de la allowlist
- **THEN** toda la UI usa trigramas y marcadores de texto, sin un solo emoji

#### Scenario: Timeline sin emoji
- **WHEN** la política de emoji está inactiva y se abre el detalle de un partido con goles y tarjetas
- **THEN** los eventos usan marcadores de texto distinguibles por tipo y color, con las columnas alineadas
