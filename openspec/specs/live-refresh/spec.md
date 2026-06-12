# live-refresh Specification

## Purpose

Actualización automática de datos en segundo plano sin bloquear la UI, con cadencia diferenciada por vista y degradación elegante ante errores de red.

## Requirements

### Requirement: Actualización automática en segundo plano
El sistema SHALL actualizar los datos automáticamente sin bloquear la UI: el scoreboard cada ~30 segundos, y el summary del partido abierto en detalle cada ~15 segundos. El polling del summary SHALL detenerse al salir de la vista de detalle.

#### Scenario: UI fluida durante fetch
- **WHEN** una petición de red está en curso (incluso lenta)
- **THEN** la UI sigue respondiendo a teclado y repintando sin congelarse

#### Scenario: Detalle abierto
- **WHEN** el usuario tiene abierta la vista de detalle de un partido en vivo
- **THEN** los eventos y el marcador de ese partido se refrescan a la cadencia rápida (~15s) sin acción del usuario

### Requirement: Cadencia relajada sin partidos en vivo
El sistema SHALL reducir la frecuencia de polling (a ~120 segundos) cuando ningún partido del día esté en curso.

#### Scenario: Sin partidos en vivo
- **WHEN** todos los partidos del día están finalizados o por comenzar
- **THEN** el polling del scoreboard pasa a la cadencia relajada hasta que un partido entre en vivo

### Requirement: Degradación elegante ante errores
Ante un fallo de actualización, el sistema SHALL conservar y seguir mostrando los últimos datos válidos, indicar el problema de forma no intrusiva, y reintentar en el siguiente ciclo. La aplicación NUNCA SHALL terminar ni vaciar la pantalla por un error de red.

#### Scenario: Red caída temporalmente
- **WHEN** varios ciclos de polling fallan consecutivamente
- **THEN** la UI sigue mostrando los últimos datos con su indicador de frescura creciendo y un aviso discreto de reconexión, y se recupera sola cuando vuelve la red

### Requirement: Refresh manual
El sistema SHALL permitir forzar una actualización inmediata con la tecla r en cualquier vista.

#### Scenario: Usuario impaciente
- **WHEN** el usuario presiona r
- **THEN** se dispara un fetch inmediato de los datos de la vista actual sin esperar al siguiente ciclo
