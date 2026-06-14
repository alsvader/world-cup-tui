# match-data Specification

## Purpose

Obtención y normalización de datos del Mundial 2026 desde la API no documentada de ESPN: partidos relevantes (hoy + finalizados de ayer), carga bajo demanda de jornadas anteriores, marcador, minuto, estado y eventos clave, con tolerancia a fallos de red y campos irregulares.

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

### Requirement: Cargar partidos finalizados de una jornada anterior bajo demanda
El sistema SHALL permitir obtener desde la API de ESPN los partidos FINALIZADOS de un día calendario específico (hora local), mediante una petición puntual al scoreboard con rango de un solo día (`dates=YYYYMMDD-YYYYMMDD`). Esta carga SHALL ser independiente del poll periódico del scoreboard (hoy + ayer) y SHALL devolver solo partidos con estado finalizado cuya fecha de inicio, en hora local, coincide con el día solicitado.

#### Scenario: Carga exitosa de jornada anterior
- **WHEN** se solicita el scoreboard para un día con partidos finalizados
- **THEN** el sistema devuelve esos partidos normalizados al modelo propio, todos con estado finalizado

#### Scenario: Día sin partidos finalizados
- **WHEN** se solicita un día sin partidos finalizados (día de descanso o sin fixture)
- **THEN** el sistema devuelve una lista vacía sin error

#### Scenario: Límite del torneo
- **WHEN** se solicita un día anterior al 11 de junio de 2026
- **THEN** la solicitud no se realiza (el llamador rechaza la carga antes del fetch)

### Requirement: Integrar historial sin duplicados
Al anexar partidos de jornadas anteriores al conjunto en memoria, el sistema SHALL fusionar por `id` de partido: los existentes se conservan, los nuevos se añaden. El poll periódico SHALL seguir aplicando `filter_relevant` solo a su propio fetch y SHALL actualizar partidos ya presentes (incluidos los cargados por historial) cuando el mismo `id` aparece en el scoreboard actual.

#### Scenario: Partido ya presente
- **WHEN** un partido del historial también aparece en el poll de hoy/ayer
- **THEN** una sola entrada permanece en memoria y el poll actualiza marcador/estado

#### Scenario: Partidos nuevos del historial
- **WHEN** se cargan partidos finalizados de un día no presentes en memoria
- **THEN** se añaden al conjunto sin duplicar ids existentes
