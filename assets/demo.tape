# Demo principal: dashboard → detalle (dos columnas) → toggle TODO → scroll.
# Regenerar con: vhs assets/demo.tape  (idealmente durante un partido en vivo)
# Nota: se graba con --no-flags porque el renderer de VHS (xterm.js/Chromium)
# no liga los pares regional-indicator de las banderas emoji.
Output assets/demo.gif

Set FontSize 15
Set Width 1200
Set Height 720
Set Padding 12
Set Theme { "background": "#141313", "foreground": "#e5e2e1", "cursor": "#c8c6c8" }

Hide
Type "./target/release/world-cup-tui --no-flags"
Enter
Sleep 6s
Show

Sleep 3s
Screenshot assets/dashboard.png

# Bajar al partido inaugural (finalizados) y abrir su detalle
Down
Sleep 800ms
Enter
Sleep 5s
Screenshot assets/detail.png
Sleep 2s

# Alternar a vista completa y scrollear el historial
Type "t"
Sleep 2.5s
Type "kkkkkk"
Sleep 2.5s
Type "jjj"
Sleep 1.5s
Type "t"
Sleep 2s

# Volver al dashboard
Escape
Sleep 2.5s

Hide
Type "q"
