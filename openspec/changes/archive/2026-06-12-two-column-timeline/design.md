## Context

El detalle hoy renderiza `app.events` completo como lista única (Paragraph) y trunca por arriba con `skip = len - visible`, sin scroll. Los eventos del modelo (`KeyEvent`) traen `team: Option<String>` (displayName) y minuto como string ("45'+4'"); los eventos sin equipo (kickoff, medio tiempo, pausas) son exactamente el ruido que la vista por defecto va a filtrar. Decisiones tomadas con el usuario en exploración: layout A1 (cronología compartida con minuto al centro), toggle `t`, scroll sticky-bottom.

## Goals / Non-Goals

**Goals:**
- Lectura por equipo de un vistazo: goles y tarjetas en la columna del equipo correspondiente, cronología preservada.
- Acceso a la información completa (cambios, pausas) a una tecla, sin perderla.
- Ver todo el historial con scroll, sin romper el seguimiento en vivo.

**Non-Goals:**
- Estadísticas adicionales (posesión, tiros, xG) — no existen en nuestro modelo.
- Cambios en la capa de datos o en el polling.
- Mouse/rueda — solo teclado, consistente con el resto de la TUI.

## Decisions

### D1: Layout A1 — cronología compartida con minuto al centro
Una fila por evento visible, ordenadas cronológicamente (orden del feed). Tres zonas por fila: columna local (alineada a la derecha contra el centro), minuto al centro (ancho fijo 7), columna visitante (alineada a la izquierda). El evento se asigna a columna comparando `event.team` con `match.home.name` / `match.away.name`; si no coincide con ninguno (o no hay equipo) en vista filtrada no aparece, y en vista TODO se renderiza centrado a lo ancho (caso kickoff/medio tiempo).

*Alternativa considerada*: pilas independientes por equipo (A2) — más densa pero pierde el interleaving temporal; descartada con el usuario.

### D2: Filtro por defecto + toggle con `t`
Estado `TimelineMode { Key, All }` en `App`, default `Key` (solo `Goal` y `Card`). `t` alterna; el título del panel indica el modo y la tecla: `" EVENTOS — GOLES Y TARJETAS [T: TODO] "` / `" EVENTOS — TODO [T: CLAVE] "`. El footer del detalle agrega `[T]`. Al cambiar de modo el scroll se resetea a "pegado al fondo".

### D3: Scroll sticky-bottom
Estado `scroll: Option<usize>` en `App`: `None` = pegado al fondo (default, modo seguir-en-vivo); `Some(offset)` = posición fija desde arriba. `↑`/`k` desde el fondo fija `Some(max_offset-1)`; `↓`/`j` en el fondo no hace nada; volver al fondo restablece `None`. Eventos nuevos con `Some(_)` no mueven la vista; con `None` la vista sigue al último. Indicadores en los bordes del panel: `▲ n` arriba cuando hay filas ocultas arriba, `▼ n` abajo cuando las hay abajo (cubre también el aviso de "eventos nuevos" sin estado extra).

El cálculo de `max_offset` depende de la altura del panel, que solo se conoce en render: el render *clampa* el offset efectivo y la corrección se persiste en el estado en el siguiente input (patrón estándar en ratatui; evita pasar dimensiones al manejo de teclas).

### D4: Encabezado de columnas fijo
Primera fila del panel (no scrolleable): `🇲🇽 MEXICO` a la izquierda, `SOUTH AFRICA 🇿🇦` a la derecha (mismos slots de identidad de `country-flags`). El cuerpo scrollea debajo.

### D5: Sin cambios de modelo
La asignación por nombre de equipo se hace en la capa de UI. Si en el futuro hiciera falta robustez (nombres distintos entre scoreboard y summary), se movería a normalización con el id de equipo de ESPN — no ahora (YAGNI; los displayName provienen de la misma fuente).

## Risks / Trade-offs

- **[Nombre de equipo del evento ≠ nombre en el match]** → Improbable (misma fuente); si ocurre, el evento solo aparece en vista TODO centrado. Verificable en partidos reales del torneo de inmediato.
- **[Autogol acreditado al equipo beneficiado]** → Pendiente de fixture real; en el peor caso el autogol aparece en la columna "equivocada" semánticamente pero consistente con el marcador (que es como lo muestran las apps de ESPN).
- **[Columnas estrechas en terminales angostos]** → Nombres de jugador se truncan con elipsis al ancho de columna; el minuto al centro nunca se trunca.

## Open Questions

- Comportamiento del feed en penales de eliminatorias (¿columna del pateador?): se resolverá con fixture real a partir del 28 de junio.
