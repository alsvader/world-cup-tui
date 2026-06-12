# match-list-ui Specification

## Purpose

Vista principal de la TUI: dashboard de tres paneles (EN VIVO / PRÓXIMOS / FINALIZADOS) según la pantalla "World Cup TUI Dashboard" de Stitch, con navegación por teclado.

## Requirements

### Requirement: Dashboard de partidos del día en tres paneles
La TUI SHALL mostrar los partidos del día como un dashboard de tres paneles separados (según la pantalla "World Cup TUI Dashboard" de Stitch): EN VIVO (con contador "[N ACTIVOS]" en el título), PRÓXIMOS (tabla con columnas HORA / PARTIDO / SEDE y zona horaria en el título) y FINALIZADOS. Las filas con marcador SHALL mostrar el minuto a la izquierda y el marcador en caja "[ h - a ]". La pantalla SHALL incluir una barra superior con la marca y la fecha/hora local, y una barra inferior con los atajos de teclado en formato "[TECLA] ACCIÓN".

#### Scenario: Partidos en los tres estados
- **WHEN** hay partidos en vivo, próximos y finalizados en el día
- **THEN** cada partido aparece en su panel correspondiente con el dato relevante a su estado (minuto y marcador, hora local y sede, o marcador final)

#### Scenario: Panel sin partidos
- **WHEN** un panel no tiene partidos (p. ej. ninguno en vivo)
- **THEN** el panel muestra un mensaje claro ("SIN PARTIDOS EN VIVO AHORA", "NO QUEDAN PARTIDOS HOY", "AÚN NINGUNO") en lugar de quedar vacío

#### Scenario: Finalizados de ayer
- **WHEN** hay partidos finalizados del día anterior
- **THEN** aparecen en el panel FINALIZADOS marcados "AYER" (los de hoy muestran "FT"), con su detalle accesible con Enter

### Requirement: Navegación por teclado y selección
La TUI SHALL permitir mover la selección entre partidos con las flechas (y j/k), abrir el detalle del partido seleccionado con Enter, y salir de la aplicación con q.

#### Scenario: Abrir detalle
- **WHEN** el usuario presiona Enter sobre un partido de la lista
- **THEN** la TUI cambia a la vista de detalle de ese partido

#### Scenario: Salir
- **WHEN** el usuario presiona q en la vista de lista
- **THEN** la aplicación termina restaurando la terminal a su estado normal
