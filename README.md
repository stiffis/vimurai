# Vimurai - Vim Learning Application

## Project Overview

Vimurai es una aplicacion CLI interactiva para aprender Vim mediante practica muscular. Esta escrita en Rust y usa ratatui para la interfaz grafica de terminal.

## Estado Actual del Proyecto

El proyecto compila correctamente y los tests pasan.

### Bugs Corregidos
- **Escape en modo practica**: Ahora solo limpia el buffer de comandos y asegura el modo Normal, sin salir al menú.
- **Modo Visual**: Se verificó que no permite insertar caracteres inválidos.
- **Compilacion**: Se corrigieron errores de tipos y warnings.
- **Tests**: Se arreglaron los tests unitarios de SM-2 y se añadieron tests para `app.rs`.

## Estructura del Proyecto

```
vimurai/
├── Cargo.toml              # Dependencias (ratatui, crossterm, rusqlite, etc.)
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library root
│   ├── app/
│   │   ├── mod.rs          # App module, run() function
│   │   ├── app.rs          # Main application logic (PRINCIPAL - tiene bugs)
│   │   ├── screens.rs      # Screen states (Screen, PracticeState, etc.)
│   │   └── ui/mod.rs       # TUI rendering con ratatui
│   ├── engine/
│   │   ├── mod.rs          # Engine module
│   │   ├── mode.rs         # VimMode enum (Normal, Insert, Visual, Command)
│   │   └── vim_buffer.rs   # VimBuffer struct (lines, cursor_row, cursor_col)
│   ├── commands/
│   │   └── command_db.rs   # Base de datos de 25 comandos Vim
│   ├── spaced_rep/
│   │   └── sm2.rs          # Algoritmo SM-2 spaced repetition
│   └── database/
│       └── user_progress.rs # Persistencia SQLite
└── vim-trainer-features.md  # Especificacion de features
```

## Bugs Conocidos en app.rs

1. **Escape en modo practica sale al menu**: El Escape deberia cambiar al modo Normal, no salir de la pantalla de practica
2. **Modo Visual permite escribir**: En modo Visual, solo deberian funcionar movimientos y comandos (v, d, y, etc.), no insertar caracteres
3. **Compilacion**: Error en line 89 con tipos de match
4. **Buscar mas bugs**
## Comandos Vim Implementados

### Modo Normal
- `i` - Insert mode
- `a` - Append (cursor +1)
- `I` - Insert at line start
- `A` - Insert at line end (CORRECCION: cursor al final de la linea)
- `o` - New line below
- `O` - New line above
- `v` - Visual mode
- `:` - Command mode
- `h/j/k/l` - Movimiento
- `w/b` - Palabra adelante/atras
- `0/$` - Inicio/fin de linea
- `gg/G` - Inicio/fin del archivo
- `x` - Delete character
- `dd` - Delete line (2-char command)
- `yy` - Yank line (2-char command)
- `p/P` - Paste after/before
- `u` - Undo

### Modo Insert
- `Esc` - Salir a modo Normal
- Caracteres - Insertar texto
- `Backspace` - Borrar caracter
- `Enter` - Nueva linea
- Arrow keys - Movimiento

### Modo Visual
- `Esc` o `v` - Salir a modo Normal
- `h/j/k/l/w/b/0/$` - Movimiento (extiende seleccion)
- `d/x` - Delete seleccion
- `y` - Yank seleccion
- `c` - Change seleccion (delete + insert mode)

## Pantallas

- `MainMenu` - Menu principal con opciones
- `DailyDrill` - Practica diaria (3-5 min)
- `FreePractice` - Practica libre en buffer Vim
- `GuidedLearning` - Aprendizaje guiado paso a paso
- `Progress` - Estadisticas y logros
- `Settings` - Configuracion (hints, duracion, dificultad, sonido)
- `Help` - Atajos de teclado

## Dependencias Principales

```toml
[dependencies]
ratatui = "0.26"      # TUI framework
crossterm = "0.27"    # Terminal input/output
rusqlite = "0.31"     # SQLite database
anyhow = "1.0"        # Error handling
```

## Para Continuar el Trabajo

1. **Arreglar compilacion**:
   ```rust
   // En src/app/app.rs linea 89
   _ => Ok(())  // En vez de _ => {}
   ```

2. **Eliminar import no usado** (linea 10):
   ```rust
   use crate::engine::vim_buffer::MoveDirection;  // Solo esto
   ```

3. **Verificar que el buffer Vim funciona al 100%**:
   - Escape solo cambia modo, no sale de practica
   - Modo Visual no permite typing
   - Todos los comandos funcionan correctamente

## Siguientes Tareas

- [x] Fix compilation error en app.rs:89
- [x] Remove unused VimBuffer import warning
- [x] Test Vim buffer mode switching
- [x] Test Visual mode (no typing allowed)
- [x] Test Escape key behavior
- [ ] Verificar todos los comandos Vim implementados
