# match-data Specification

## Purpose

Obtención y normalización de datos del Mundial 2026 desde la API no documentada de ESPN: partidos relevantes (hoy + finalizados de ayer), marcador, minuto, estado y eventos clave, con tolerancia a fallos de red y campos irregulares.

## Requirements

### Requirement: Obtener los partidos del día
El sistema SHALL obtener desde la API de ESPN (`fifa.world/scoreboard`, con rango de fechas) los partidos del Mundial 2026 relevantes: todos los del día actual (hora local) y los FINALIZADOS del día anterior. Para cada partido: equipos, marcador, estado (programado / en vivo / medio tiempo / finalizado), minuto en curso (si aplica), hora de inicio, sede y ciudad.

#### Scenario: Resultados de ayer disponibles
- **WHEN** se consulta el scoreboard la mañana siguiente a una jornada
- **THEN** los partidos finalizados del día anterior están presentes, y los programados/en vivo de días distintos a hoy no

#### Scenario: Día con partidos en distintos estados
- **WHEN** se consulta el scoreboard en un día con partidos en vivo, finalizados y por jugar
- **THEN** el sistema devuelve todos los partidos del día con su estado, marcador y minuto correctamente clasificados en el modelo propio

#### Scenario: Día sin partidos
- **WHEN** se consulta el scoreboard en un día sin partidos programados
- **THEN** el sistema devuelve una lista vacía sin error

### Requirement: Obtener eventos clave de un partido
El sistema SHALL obtener desde la API de ESPN (`fifa.world/summary?event=<id>`) los eventos clave de un partido: goles (con autor, minuto y tipo), tarjetas amarillas, tarjetas rojas y cambios, en orden cronológico.

#### Scenario: Partido con goles y tarjetas
- **WHEN** se consulta el summary de un partido que tiene goles y tarjetas registrados
- **THEN** cada gol incluye autor, minuto y tipo, y cada tarjeta incluye jugador, minuto y color, en orden cronológico

#### Scenario: Tipo de evento no reconocido
- **WHEN** el feed incluye un tipo de evento no contemplado en el mapeo (p. ej. uno nuevo en prórroga)
- **THEN** el evento se conserva como evento genérico con su texto y minuto, sin descartarse ni causar error

### Requirement: Normalización a modelo propio tolerante a campos ausentes
El sistema SHALL deserializar el JSON de ESPN tratando como opcional todo campo no garantizado, y SHALL convertir inmediatamente a un modelo propio con tipos garantizados. Ningún campo ausente o con forma inesperada SHALL causar un fallo de parseo del partido completo.

#### Scenario: Campo ausente en el JSON
- **WHEN** un partido del feed carece de un campo no esencial (p. ej. asistencia, broadcast)
- **THEN** el partido se normaliza correctamente con valores por defecto y el resto de los datos intactos

#### Scenario: JSON real de fixture
- **WHEN** se deserializa un fixture JSON real capturado de la API (scoreboard y summary)
- **THEN** la normalización produce el modelo esperado (verificado en tests automatizados)

### Requirement: Tolerancia a fallos de red
Las funciones de obtención de datos SHALL devolver errores recuperables (nunca panic) ante fallos de red, timeouts o respuestas no-JSON.

#### Scenario: Sin conexión
- **WHEN** una petición falla por red caída o timeout
- **THEN** la función devuelve un error descriptivo que el llamador puede manejar, y el proceso no termina
