## MODIFIED Requirements

### Requirement: Dashboard de partidos del día en tres paneles
La TUI SHALL mostrar los partidos como un dashboard de tres paneles separados (según la pantalla "World Cup TUI Dashboard" de Stitch): EN VIVO (con contador "[N ACTIVOS]" en el título), PRÓXIMOS (tabla con columnas HORA / PARTIDO / SEDE y zona horaria en el título) y FINALIZADOS. Las filas con marcador SHALL mostrar el minuto o etiqueta de estado a la izquierda y el marcador en caja "[ h - a ]". El panel FINALIZADOS SHALL tener altura acotada con scroll interno cuando el contenido excede el área visible (mismo patrón de ventana que el timeline del detalle, con indicador de filas ocultas). Los partidos finalizados de jornadas anteriores cargadas bajo demanda SHALL agruparse con separadores por jornada (p. ej. `─── VIE 13 JUN ───`) ordenados por fecha local descendente (más reciente arriba). La pantalla SHALL incluir una barra superior con la marca y la fecha/hora local, y una barra inferior con los atajos de teclado en formato "[TECLA] ACCIÓN".

#### Scenario: Partidos en los tres estados
- **WHEN** hay partidos en vivo, próximos y finalizados en el día
- **THEN** cada partido aparece en su panel correspondiente con el dato relevante a su estado (minuto y marcador, hora local y sede, o marcador final)

#### Scenario: Panel sin partidos
- **WHEN** un panel no tiene partidos (p. ej. ninguno en vivo)
- **THEN** el panel muestra un mensaje claro ("SIN PARTIDOS EN VIVO AHORA", "NO QUEDAN PARTIDOS HOY", "AÚN NINGUNO") en lugar de quedar vacío

#### Scenario: Finalizados de ayer
- **WHEN** hay partidos finalizados del día anterior
- **THEN** aparecen en el panel FINALIZADOS marcados "AYER" (los de hoy muestran "FT"), con su detalle accesible con Enter

#### Scenario: Scroll en finalizados con historial
- **WHEN** el panel FINALIZADOS tiene más filas (incluyendo separadores) que su altura visible
- **THEN** el usuario puede desplazar la vista con j/k al mover la selección y el panel muestra cuántas filas quedan ocultas arriba o abajo

#### Scenario: Separadores por jornada
- **WHEN** hay partidos finalizados de varias fechas locales en el panel
- **THEN** cada jornada muestra un separador con la fecha abreviada en español antes de sus partidos

#### Scenario: Etiqueta de jornadas antiguas
- **WHEN** un partido finalizado es de una fecha local anterior a ayer
- **THEN** la columna izquierda muestra la fecha abreviada (p. ej. "VIE 13/06") en lugar de "AYER" o "FT"

## ADDED Requirements

### Requirement: Cargar jornada anterior con tecla p
En la vista de lista, la TUI SHALL cargar bajo demanda los partidos finalizados del día calendario local inmediatamente anterior al límite de historial ya cargado cuando el usuario presiona `p` (previous). La acción SHALL anexar esos partidos al panel FINALIZADOS. La tecla SHALL ignorarse mientras una carga histórica está en curso o cuando el límite de historial ya es el 11 de junio de 2026 (inicio del torneo). La barra inferior SHALL incluir `[P] JORNADA ANTERIOR`.

#### Scenario: Cargar viernes desde domingo
- **WHEN** el usuario está en lista con historial hasta sábado y presiona `p`
- **THEN** se solicitan y muestran los finalizados del viernes anexados al panel, con separador de jornada

#### Scenario: Límite del torneo
- **WHEN** el historial ya incluye el 11 de junio de 2026 y el usuario presiona `p`
- **THEN** no se realiza petición de red y la lista no cambia

#### Scenario: Carga en curso
- **WHEN** una carga histórica está en progreso y el usuario presiona `p`
- **THEN** la pulsación se ignora hasta completar la carga anterior
