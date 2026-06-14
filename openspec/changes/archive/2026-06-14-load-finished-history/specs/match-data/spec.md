## ADDED Requirements

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
