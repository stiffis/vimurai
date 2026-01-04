# Vimurai Curriculum: The Path to Mastery

Este currículo está diseñado para transformar a un usuario desde "apenas puedo salir" hasta "edición a la velocidad del pensamiento". Los ejercicios simulan escenarios reales de código (Rust, JS, HTML, JSON).

## Nivel 1: The Survivor (Sobrevivencia)
**Foco:** Moverse sin romper nada y ediciones de emergencia.

### 1.1 Basic Navigation (The Dungeon Crawler)
*Escenario: Navegar por un archivo de log simple.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| N1 | The Basics | Lista de imports | Moverse a `use std::io;` | `j` / `k` |
| N2 | Lateral Move | Argumentos de función | Ir al último argumento | `l` (varias veces) |
| N3 | Word Jump | `let mut counter = 0;` | Saltar por palabras hasta `0` | `w` `w` `w` `w` |
| N4 | Backtrack | `return result;` | Volver al inicio de `return` | `b` |

### 1.2 Insertion (The Writer)
*Escenario: Corregir typos y añadir punto y coma.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| I1 | Append End | `let x = 5` (falta ;) | Añadir `;` al final | `A;` <Esc> |
| I2 | Insert Start | `pub fn main()` (falta pub) | Insertar `pub ` al inicio | `Ipub ` <Esc> |
| I3 | Open Below | `fn main() {` | Abrir nueva línea para código | `o` |
| I4 | Correction | `prnitln!` | Corregir typo (cursor en 'n') | `x` |

---

## Nivel 2: The Sniper (Precisión Horizontal)
**Foco:** Dejar de spammear `l` y `h`. Moverse al carácter exacto.

### 2.1 Character Search (f/t)
*Escenario: Editar parámetros y objetos JSON.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| S1 | Find Char | `user.get_id();` | Saltar directo al punto `.` | `f.` |
| S2 | Find & Edit | `const MAX_SIZE: u32 = 100;` | Saltar al `:` para cambiar tipo | `f:` |
| S3 | Till Char | `("cadena de texto")` | Saltar justo antes del cierre `)` | `t)` |
| S4 | Find Back | `let res = calculate(x);` | Volver a la `(` desde el final | `F(` |

### 2.2 Line Mastery (0, ^, $)
*Escenario: Indentación y comentarios.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| L1 | Hard Start | `    let x = 1;` | Ir a la 'l' (ignorar espacios) | `^` |
| L2 | Absolute End | `// TODO: Fix this` | Ir al final para escribir más | `A` (o `$a`) |
| L3 | Clean Line | `    // Comentario viejo` | Borrar contenido manteninedo indent | `^D` |

---

## Nivel 3: The Refactorer (Gramática Vim)
**Foco:** Operador + Movimiento. Pensar en "Verbo + Sustantivo".

### 3.1 Deletion & Change
*Escenario: Limpieza de código legacy.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| R1 | Change Word | `let count = 0;` | Cambiar `count` por `total` | `cwtotal`<Esc> |
| R2 | Delete 3 Words| `vec![1, 2, 3, 4];` | Borrar `1, 2, 3,` | `d3w` |
| R3 | Delete to Char| `fn process(a, b, c)` | Borrar argumentos hasta `)` | `dt)` |
| R4 | Change Line | `let complex_logic = ...` | Reescribir línea entera | `S` (o `cc`) |

### 3.2 Visual Mode Operations
*Escenario: Manipulación de bloques.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| V1 | Select & Yank | Bloque de función | Copiar 3 líneas | `Vjjy` |
| V2 | Visual Indent | 4 líneas mal indentadas | Indentar bloque a derecha | `Vjjj>` |

---

## Nivel 4: The Surgeon (Text Objects)
**Foco:** Editar "dentro" de cosas. Aquí es donde Vim enamora.

### 4.1 Inner Objects (i)
*Escenario: Strings, Paréntesis y Tags HTML.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| O1 | Inside Quotes | `let s = "borrame";` | Cambiar texto entre comillas | `ci"` |
| O2 | Inside Parens | `if (x > 10 && y < 5)` | Cambiar condición completa | `ci(` |
| O3 | Inside Word | `function getUserData()` | Cambiar nombre (cursor en medio) | `ciw` |
| O4 | Inside Tag | `<div>Contenido viejo</div>` | Cambiar contenido HTML | `cit` |

### 4.2 Around Objects (a)
*Escenario: Borrar funciones completas o bloques.*
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| A1 | Delete Call | `main(args);` | Borrar `(args)` entero | `da(` |
| A2 | Delete Block | `{ code_block }` | Borrar llaves y contenido | `da{` |

---

## Nivel 5: The Architect (Multi-file & Search)
**Foco:** Moverse por el proyecto.

### 5.1 Search & Jump
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| J1 | Search Forward| Archivo largo | Buscar "Error" | `/Error`<Enter> |
| J2 | Next Match | (Después de búsqueda) | Ir al siguiente "Error" | `n` |
| J3 | Prev Match | (Después de búsqueda) | Ir al anterior "Error" | `N` |
| J4 | File Navigation| Archivo de 100 líneas | Ir a la línea 42 | `42G` o `42gg` |

### 5.2 Scrolling
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| Z1 | Center View | Cursor al fondo de pantalla | Centrar línea en pantalla | `zz` |
| Z2 | Top View | Cursor en medio | Mover línea al tope | `zt` |

---

## Nivel 6: The Wizard (Registers & Macros)
**Foco:** Automatización de tareas repetitivas.

### 6.1 Registers (Clipboard)
| ID | Título | Contexto | Objetivo | Solución Óptima |
|----|--------|----------|----------|-----------------|
| M1 | System Copy | `export const data` | Copiar al clipboard del sistema | `"+yy` |
| M2 | System Paste | (Clipboard tiene código) | Pegar desde sistema | `"+p` |

### 6.2 Macros (Boss Battle)
*Escenario: Convertir una lista CSV a JSON.*
*Input:*
```
Juan,25
Ana,30
Luis,22
```
*Goal:*
```
{name: "Juan", age: 25},
{name: "Ana", age: 30},
...
```
*Action:* Grabar macro `qq`, ejecutar transformación en línea 1, bajar, parar `q`. Ejecutar `2@q`.

---

## Boss Battles (Exámenes Finales)

Al final de cada cinturón, un ejercicio compuesto:

1.  **Yellow Belt Boss:** Navegar por un laberinto de código usando solo `w`, `b`, `f`, `t`.
2.  **Orange Belt Boss:** Refactorizar una función sucia usando `cw`, `ct`, `dt`.
3.  **Black Belt Boss:** Convertir un bloque de HTML feo en un struct de Rust usando macros y Visual Block.