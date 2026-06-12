## 1. Núcleo de banderas

- [x] 1.1 Módulo `flags` con el mapa estático FIFA→ISO de los 48 clasificados al Mundial 2026 (ENG/SCO/WAL deliberadamente sin entrada) y función trigrama→bandera emoji
- [x] 1.2 Política de emoji centralizada `emoji_enabled()`: precedencia override (`--flags`/`--no-flags`, `WCTUI_FLAGS`) > allowlist (`TERM_PROGRAM`/`TERM`) > off; parseo de args/env en `main.rs`
- [x] 1.3 Tests unitarios: mapa (MEX→🇲🇽, ENG→None), precedencia de la política con env simulado

## 2. Integración en la UI

- [x] 2.1 Slot de identidad de ancho fijo en filas con marcador (`ui/list.rs`): bandera o trigrama junto a cada equipo, columnas estables en ambos modos
- [x] 2.2 Cabecera del detalle (`ui/detail.rs`): bandera/trigrama junto a cada selección
- [x] 2.3 Timeline con la misma política: iconos emoji vs marcadores de texto coloreados por tipo de evento
- [x] 2.4 Tabla de PRÓXIMOS: bandera/trigrama junto a cada selección en la columna PARTIDO (alcance ampliado tras prueba del usuario: identidad en todo lugar donde aparezca un partido)

## 3. Verificación

- [x] 3.1 `cargo test` + `cargo clippy` limpios
- [x] 3.2 Corrida en pty con `WCTUI_FLAGS=1` y `WCTUI_FLAGS=0`: verificar alineación de columnas idéntica en ambos modos y ausencia total de emoji en modo off
- [x] 3.3 Documentar en README (cuando exista) la detección, los overrides y la garantía de Capa 0
