# VIMURAI

> Domina Vim a través de memoria muscular.

Vimurai es un dojo interactivo para terminal construido en Rust con Ratatui.
Combina un mini‑Vim, retos sobre código realista, una academia por cinturones,
repetición espaciada SM‑2 y gamificación ligera. Kage, un gato hacker dibujado
en pixel art de terminal, acompaña la sesión y reacciona a tu forma de resolver
cada ejercicio.

## Qué incluye

- **Daily Drill:** una sesión finita de repaso adaptada a lo que estás por
  olvidar.
- **Academia:** recorrido progresivo desde `hjkl` hasta gramática de operadores,
  objetos de texto, búsqueda y navegación estructural, con 29 misiones en seis
  cinturones.
- **Sandbox:** buffer libre para practicar con snippets sin arriesgar tu
  progreso.
- **Mini‑Vim Unicode:** modos Normal, Insert, Visual, Command y Search; conteos,
  motions, operadores, registros, undo/redo y búsquedas.
- **Progreso local:** XP, nivel, racha, precisión, actividad, dominio por comando
  y logros persistidos en SQLite.
- **TUI responsiva:** layouts wide/compact, Gruvbox Dark/Light automático,
  fondo heredado de la terminal, alto contraste y restauración segura incluso
  ante errores.

La filosofía pedagógica y las decisiones de producto están documentadas en
[docs/VISION.md](docs/VISION.md).

## Ejecutar

Requisitos: Rust 1.88 o superior y una terminal Unicode. `NO_COLOR=1` y
`--ascii` ofrecen una presentación monocroma que también hereda los colores
predeterminados del terminal.

```bash
cd /home/stiff/vimurai
cargo run --release
```

Para instalar el binario en tu perfil:

```bash
cargo install --path .
vimurai
```

### Tema de terminal

Vimurai no pinta un fondo propio: conserva el fondo, transparencia y wallpaper
configurados en tu terminal. Al arrancar consulta el color predeterminado con
OSC 11 y selecciona **Gruvbox Dark** o **Gruvbox Light** según su luminancia. La
consulta espera como máximo 50 ms y conserva cualquier tecla recibida durante
ese intervalo.

Si el emulador no responde, usa `COLORFGBG`; el fallback final es Gruvbox Dark.
Puedes forzar una variante o desactivar la consulta sin cambiar tu progreso:

```bash
VIMURAI_THEME=light vimurai   # también acepta dark
VIMURAI_NO_OSC=1 vimurai      # sólo heurísticas de entorno
```

La detección se repite en cada inicio, así que alternar el perfil claro/oscuro
de la terminal y volver a abrir Vimurai elige la paleta correspondiente.

## Controles

Fuera del editor:

| Tecla | Acción |
|---|---|
| `j` / `k` o flechas | Mover selección |
| `h` / `l` | Cambiar panel |
| `Enter` | Abrir / confirmar |
| `Esc` | Volver / cerrar overlay |
| `F1` | Ayuda contextual |
| `q` | Solicitar salida |

Durante una práctica, las teclas imprimibles se reservan para Vim. Los atajos
de la aplicación evitan robar motions:

| Tecla | Acción |
|---|---|
| `F1` | Mostrar una pista contextual |
| `F2` | Salir de la práctica / volver al mapa |
| `F3` | Pausar la sesión y consultar progreso |
| `F5` | Reiniciar el reto (registra el intento) |
| `F6` | Cambiar snippet en Sandbox |
| `Ctrl-Q` | Solicitar salida de Vimurai |
| `:q` | Salir del buffer como en Vim |

El pegado bracketed se admite sólo en Sandbox: una solución copiada nunca puede
otorgar XP ni alterar la repetición espaciada.

## Datos y privacidad

Vimurai funciona sin cuentas ni red. El progreso se guarda en el directorio de
datos local del sistema, normalmente:

```text
~/.local/share/vimurai/progress.db
```

Para usar otra ubicación, define `VIMURAI_DATA_DIR`. Esto también facilita
perfiles portátiles y pruebas herméticas.

Las bases del prototipo se migran de forma transaccional. Las equivalencias
seguras pasan al currículo actual y los registros antiguos restantes quedan
archivados dentro de la propia SQLite, sin inventar dominio sobre motions no
practicados.

## Desarrollo

```bash
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test --all-targets
```

El editor es independiente de Ratatui, el currículo es declarativo y los tests
de persistencia usan SQLite `:memory:`. El guard RAII de terminal restaura raw
mode, alternate screen, cursor y bracketed paste aun durante un panic.

## Licencia

[GPL‑2.0‑only](LICENSE), en continuidad con el proyecto Vimurai original.
