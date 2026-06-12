# match-detail-ui Specification

## Purpose

Vista de detalle de un partido: cabecera con marcador, minuto, estado y sede; línea de tiempo de eventos clave; indicador de frescura de datos.

## Requirements

### Requirement: Cabecera del partido con marcador y minuto
La vista de detalle SHALL mostrar de forma prominente: ambos equipos, el marcador actual, el minuto en curso (incluyendo tiempo añadido, p. ej. "45'+4'"), el estado del partido (en vivo / medio tiempo / finalizado / programado) y la sede con su ciudad.

#### Scenario: Partido en vivo
- **WHEN** se abre el detalle de un partido en curso
- **THEN** se muestran marcador, minuto en curso, indicador de EN VIVO y sede con ciudad

#### Scenario: Partido programado
- **WHEN** se abre el detalle de un partido que aún no comienza
- **THEN** se muestra la hora de inicio local y los equipos, sin marcador inventado

### Requirement: Línea de tiempo de eventos clave
La vista de detalle SHALL mostrar los eventos del partido en orden cronológico, con minuto e iconografía diferenciada: goles (con autor y tipo), tarjetas amarillas, tarjetas rojas y cambios.

#### Scenario: Gol con autor y minuto
- **WHEN** el partido tiene goles registrados
- **THEN** cada gol aparece con minuto, autor, equipo y tipo (p. ej. de cabeza, penal), visualmente distinguible del resto de eventos

#### Scenario: Tarjetas
- **WHEN** el partido tiene tarjetas registradas
- **THEN** cada tarjeta aparece con minuto, jugador y color distinguible (amarilla/roja)

### Requirement: Indicador de frescura de datos
La TUI SHALL mostrar la hora local de la última actualización exitosa de datos ("ACTUALIZADO HH:MM:SS"). El indicador SHALL ser estático entre actualizaciones (sin contadores que cambien cada segundo, para no generar ruido visual).

#### Scenario: Datos recién actualizados
- **WHEN** llega una actualización exitosa de datos
- **THEN** el indicador muestra la hora de esa actualización y no cambia hasta la siguiente

### Requirement: Volver a la lista
La vista de detalle SHALL permitir volver a la lista con Esc y salir de la aplicación con q.

#### Scenario: Volver
- **WHEN** el usuario presiona Esc en la vista de detalle
- **THEN** la TUI regresa a la lista de partidos conservando la selección previa
