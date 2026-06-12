## MODIFIED Requirements

### Requirement: Línea de tiempo de eventos clave
La vista de detalle SHALL mostrar los eventos del partido en dos columnas por equipo (local izquierda, visitante derecha) con el minuto al centro, en orden cronológico, bajo un encabezado fijo con la identidad de cada equipo. La vista por defecto SHALL incluir únicamente goles (con autor y tipo) y tarjetas (amarillas/rojas), con iconografía diferenciada. Los eventos sin equipo asignable SHALL omitirse en la vista por defecto y mostrarse centrados en la vista completa.

#### Scenario: Gol con autor y minuto
- **WHEN** el partido tiene goles registrados
- **THEN** cada gol aparece en la columna de su equipo con minuto al centro, autor y tipo (p. ej. de cabeza, penal), visualmente distinguible del resto de eventos

#### Scenario: Tarjetas
- **WHEN** el partido tiene tarjetas registradas
- **THEN** cada tarjeta aparece en la columna de su equipo con minuto, jugador y color distinguible (amarilla/roja)

#### Scenario: Cronología compartida
- **WHEN** hay eventos de ambos equipos
- **THEN** las filas se ordenan cronológicamente de forma global (no por columna), de modo que la narrativa temporal del partido se preserva

## ADDED Requirements

### Requirement: Toggle de vista completa del timeline
La vista de detalle SHALL alternar con la tecla t entre la vista por defecto (goles y tarjetas) y la vista completa (todos los eventos: cambios, pausas, medio tiempo, etc.). El título del panel SHALL indicar el modo activo y la tecla de alternancia. Al cambiar de modo, el scroll SHALL volver al fondo.

#### Scenario: Alternar a vista completa
- **WHEN** el usuario presiona t en la vista de detalle
- **THEN** el timeline muestra todos los eventos del partido y el título del panel refleja el modo TODO

#### Scenario: Regresar a vista clave
- **WHEN** el usuario presiona t estando en vista completa
- **THEN** el timeline vuelve a mostrar solo goles y tarjetas

### Requirement: Scroll del timeline con sticky bottom
El timeline SHALL ser scrolleable con ↑↓ (y j/k) cuando el contenido excede el panel. Por defecto la vista está "pegada al fondo": los eventos nuevos mantienen la vista en el último evento. Si el usuario scrollea hacia arriba, la posición SHALL mantenerse fija ante eventos nuevos. El panel SHALL indicar overflow con marcadores de filas ocultas arriba (▲ n) y abajo (▼ n).

#### Scenario: Ver historial completo
- **WHEN** el timeline tiene más filas de las que caben y el usuario scrollea hacia arriba
- **THEN** puede alcanzar el primer evento del partido, con el marcador ▼ indicando cuántas filas quedan abajo

#### Scenario: Seguimiento en vivo sin arrastre
- **WHEN** el usuario está scrolleado hacia arriba y llegan eventos nuevos
- **THEN** la vista no se mueve y el marcador ▼ refleja las filas nuevas ocultas

#### Scenario: Pegado al fondo
- **WHEN** el usuario está al fondo del timeline y llega un evento nuevo
- **THEN** la vista avanza automáticamente para mostrarlo
