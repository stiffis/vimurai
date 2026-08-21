//! Currículo declarativo de Vimurai.
//!
//! Este módulo no conoce la interfaz, SQLite ni el bucle de eventos. Su única
//! responsabilidad es describir qué se enseña y cómo reconocer el estado final
//! de un ejercicio. Mantenerlo puro permite validar todo el contenido en tests
//! sin tocar la configuración o el progreso real del usuario.

use std::collections::{HashMap, HashSet};

use crate::editor::{Mode, Position};

/// Los seis cinturones de la campaña, en orden pedagógico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Belt {
    Survivor = 0,
    Sniper = 1,
    Refactorer = 2,
    Surgeon = 3,
    Architect = 4,
    Wizard = 5,
}

/// Texto y color semántico que la UI puede presentar para un cinturón.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeltMetadata {
    pub order: u8,
    pub slug: &'static str,
    pub name: &'static str,
    pub rank: &'static str,
    pub focus: &'static str,
    pub accent_rgb: (u8, u8, u8),
}

impl Belt {
    pub const ALL: [Self; 6] = [
        Self::Survivor,
        Self::Sniper,
        Self::Refactorer,
        Self::Surgeon,
        Self::Architect,
        Self::Wizard,
    ];

    pub const fn all() -> &'static [Self; 6] {
        &Self::ALL
    }

    pub const fn metadata(self) -> BeltMetadata {
        match self {
            Self::Survivor => BeltMetadata {
                order: 1,
                slug: "survivor",
                name: "The Survivor",
                rank: "Cinturón blanco",
                focus: "Moverse con intención y entrar o salir de Insert",
                accent_rgb: (220, 230, 240),
            },
            Self::Sniper => BeltMetadata {
                order: 2,
                slug: "sniper",
                name: "The Sniper",
                rank: "Cinturón amarillo",
                focus: "Alcanzar un carácter exacto sin repetir h o l",
                accent_rgb: (255, 214, 64),
            },
            Self::Refactorer => BeltMetadata {
                order: 3,
                slug: "refactorer",
                name: "The Refactorer",
                rank: "Cinturón naranja",
                focus: "Combinar operadores, movimientos y conteos",
                accent_rgb: (255, 139, 61),
            },
            Self::Surgeon => BeltMetadata {
                order: 4,
                slug: "surgeon",
                name: "The Surgeon",
                rank: "Cinturón rojo",
                focus: "Editar objetos de texto y selecciones completas",
                accent_rgb: (255, 77, 109),
            },
            Self::Architect => BeltMetadata {
                order: 5,
                slug: "architect",
                name: "The Architect",
                rank: "Cinturón azul",
                focus: "Buscar y recorrer archivos como una estructura",
                accent_rgb: (72, 169, 255),
            },
            Self::Wizard => BeltMetadata {
                order: 6,
                slug: "wizard",
                name: "The Wizard",
                rank: "Cinturón negro",
                focus: "Reutilizar ediciones con registros y repetición",
                accent_rgb: (194, 112, 255),
            },
        }
    }
}

/// Condición observable de victoria.
///
/// El modo forma parte de todas las metas: completar una edición también exige
/// volver conscientemente a Normal cuando esa sea la respuesta esperada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Goal {
    Cursor {
        position: Position,
        mode: Mode,
    },
    Buffer {
        lines: &'static [&'static str],
        mode: Mode,
    },
    BufferAndCursor {
        lines: &'static [&'static str],
        position: Position,
        mode: Mode,
    },
}

impl Goal {
    /// Comprueba la meta directamente contra el estado puro del editor.
    pub fn is_met(&self, buffer: &[Vec<char>], cursor: Position, mode: Mode) -> bool {
        match self {
            Self::Cursor {
                position,
                mode: expected_mode,
            } => cursor == *position && mode == *expected_mode,
            Self::Buffer {
                lines,
                mode: expected_mode,
            } => mode == *expected_mode && buffer_matches(buffer, lines),
            Self::BufferAndCursor {
                lines,
                position,
                mode: expected_mode,
            } => mode == *expected_mode && cursor == *position && buffer_matches(buffer, lines),
        }
    }

    pub const fn expected_mode(&self) -> Mode {
        match self {
            Self::Cursor { mode, .. }
            | Self::Buffer { mode, .. }
            | Self::BufferAndCursor { mode, .. } => *mode,
        }
    }

    pub const fn expected_lines(&self) -> Option<&'static [&'static str]> {
        match self {
            Self::Cursor { .. } => None,
            Self::Buffer { lines, .. } | Self::BufferAndCursor { lines, .. } => Some(lines),
        }
    }

    pub const fn expected_position(&self) -> Option<Position> {
        match self {
            Self::Cursor { position, .. } | Self::BufferAndCursor { position, .. } => {
                Some(*position)
            }
            Self::Buffer { .. } => None,
        }
    }
}

fn buffer_matches(actual: &[Vec<char>], expected: &[&str]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual_line, expected_line)| {
                actual_line.iter().copied().eq(expected_line.chars())
            })
}

/// Una práctica autocontenida y estable para persistencia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exercise {
    pub id: &'static str,
    pub order: u16,
    pub belt: Belt,
    pub title: &'static str,
    pub context: &'static str,
    pub objective: &'static str,
    pub initial_lines: &'static [&'static str],
    pub start: Position,
    pub goal: Goal,
    pub hint: &'static str,
    /// Secuencia de referencia; `<Esc>`, `<Enter>` y similares son teclas únicas.
    pub solution: &'static str,
    /// IDs estables del catálogo que este ejercicio entrena.
    pub skills: &'static [&'static str],
    /// Atajos que invalidan la intención pedagógica, expresados como IDs o tokens.
    pub forbidden: &'static [&'static str],
    pub estimated_secs: u16,
    /// Número de eventos de teclado de la solución de referencia.
    pub optimal_actions: u16,
}

impl Exercise {
    pub fn initial_buffer(&self) -> Vec<Vec<char>> {
        self.initial_lines
            .iter()
            .map(|line| line.chars().collect())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCategory {
    Motion,
    Insert,
    Edit,
    Operator,
    TextObject,
    Visual,
    Search,
    Jump,
    View,
    Register,
    Repeat,
}

/// Entrada para la ayuda dinámica y para validar `Exercise::skills`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandInfo {
    pub id: &'static str,
    pub keys: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: CommandCategory,
    pub introduced_in: Belt,
}

const fn command(
    id: &'static str,
    keys: &'static str,
    name: &'static str,
    description: &'static str,
    category: CommandCategory,
    introduced_in: Belt,
) -> CommandInfo {
    CommandInfo {
        id,
        keys,
        name,
        description,
        category,
        introduced_in,
    }
}

/// Catálogo canónico de las funciones que enseña esta primera campaña.
pub fn command_catalog() -> Vec<CommandInfo> {
    use Belt::*;
    use CommandCategory::*;

    vec![
        command(
            "move-left",
            "h",
            "Izquierda",
            "Mueve una columna a la izquierda",
            Motion,
            Survivor,
        ),
        command(
            "move-down",
            "j",
            "Abajo",
            "Mueve una línea hacia abajo",
            Motion,
            Survivor,
        ),
        command(
            "move-up",
            "k",
            "Arriba",
            "Mueve una línea hacia arriba",
            Motion,
            Survivor,
        ),
        command(
            "move-right",
            "l",
            "Derecha",
            "Mueve una columna a la derecha",
            Motion,
            Survivor,
        ),
        command(
            "word-forward",
            "w",
            "Siguiente palabra",
            "Salta al inicio de la siguiente palabra",
            Motion,
            Survivor,
        ),
        command(
            "word-backward",
            "b",
            "Palabra anterior",
            "Salta al inicio de la palabra anterior",
            Motion,
            Survivor,
        ),
        command(
            "word-end",
            "e",
            "Fin de palabra",
            "Salta al final de la palabra",
            Motion,
            Survivor,
        ),
        command(
            "insert-before",
            "i",
            "Insertar",
            "Entra a Insert antes del cursor",
            Insert,
            Survivor,
        ),
        command(
            "append-after",
            "a",
            "Añadir",
            "Entra a Insert después del cursor",
            Insert,
            Survivor,
        ),
        command(
            "insert-line-start",
            "I",
            "Insertar al inicio",
            "Inserta en el primer carácter no blanco",
            Insert,
            Survivor,
        ),
        command(
            "append-line-end",
            "A",
            "Añadir al final",
            "Inserta al final de la línea",
            Insert,
            Survivor,
        ),
        command(
            "open-below",
            "o",
            "Abrir debajo",
            "Crea una línea debajo y entra a Insert",
            Insert,
            Survivor,
        ),
        command(
            "open-above",
            "O",
            "Abrir encima",
            "Crea una línea encima y entra a Insert",
            Insert,
            Survivor,
        ),
        command(
            "escape",
            "<Esc>",
            "Modo Normal",
            "Sale del modo actual y vuelve a Normal",
            Insert,
            Survivor,
        ),
        command(
            "delete-char",
            "x",
            "Borrar carácter",
            "Borra el carácter bajo el cursor",
            Edit,
            Survivor,
        ),
        command(
            "replace-char",
            "r{char}",
            "Reemplazar carácter",
            "Reemplaza un carácter sin entrar a Insert",
            Edit,
            Survivor,
        ),
        command(
            "find-forward",
            "f{char}",
            "Encontrar delante",
            "Salta sobre el próximo carácter indicado",
            Motion,
            Sniper,
        ),
        command(
            "find-backward",
            "F{char}",
            "Encontrar detrás",
            "Salta hacia atrás sobre el carácter indicado",
            Motion,
            Sniper,
        ),
        command(
            "till-forward",
            "t{char}",
            "Hasta delante",
            "Salta justo antes del próximo carácter",
            Motion,
            Sniper,
        ),
        command(
            "till-backward",
            "T{char}",
            "Hasta detrás",
            "Salta justo después del carácter anterior",
            Motion,
            Sniper,
        ),
        command(
            "line-start",
            "0",
            "Inicio absoluto",
            "Va a la columna cero",
            Motion,
            Sniper,
        ),
        command(
            "first-nonblank",
            "^",
            "Inicio del texto",
            "Va al primer carácter no blanco",
            Motion,
            Sniper,
        ),
        command(
            "line-end",
            "$",
            "Fin de línea",
            "Va al último carácter de la línea",
            Motion,
            Sniper,
        ),
        command(
            "count",
            "{n}",
            "Conteo",
            "Repite el movimiento u operador n veces",
            Operator,
            Refactorer,
        ),
        command(
            "delete-op",
            "d{motion}",
            "Eliminar",
            "Elimina el texto cubierto por un movimiento",
            Operator,
            Refactorer,
        ),
        command(
            "yank-op",
            "y{motion}",
            "Copiar",
            "Copia el texto cubierto por un movimiento",
            Operator,
            Refactorer,
        ),
        command(
            "change-op",
            "c{motion}",
            "Cambiar",
            "Elimina un rango y entra a Insert",
            Operator,
            Refactorer,
        ),
        command(
            "delete-line",
            "dd",
            "Eliminar línea",
            "Elimina la línea actual",
            Operator,
            Refactorer,
        ),
        command(
            "yank-line",
            "yy",
            "Copiar línea",
            "Copia la línea actual",
            Operator,
            Refactorer,
        ),
        command(
            "change-line",
            "cc",
            "Cambiar línea",
            "Vacía la línea actual y entra a Insert",
            Operator,
            Refactorer,
        ),
        command(
            "paste-after",
            "p",
            "Pegar después",
            "Pega después del cursor o debajo de la línea",
            Edit,
            Refactorer,
        ),
        command(
            "paste-before",
            "P",
            "Pegar antes",
            "Pega antes del cursor o encima de la línea",
            Edit,
            Refactorer,
        ),
        command(
            "inner-word",
            "iw",
            "Dentro de palabra",
            "Selecciona el contenido de la palabra",
            TextObject,
            Surgeon,
        ),
        command(
            "around-word",
            "aw",
            "Alrededor de palabra",
            "Selecciona la palabra y su espacio adyacente",
            TextObject,
            Surgeon,
        ),
        command(
            "inner-quotes",
            "i\"",
            "Dentro de comillas",
            "Selecciona el contenido entre comillas",
            TextObject,
            Surgeon,
        ),
        command(
            "around-quotes",
            "a\"",
            "Alrededor de comillas",
            "Selecciona comillas y contenido",
            TextObject,
            Surgeon,
        ),
        command(
            "inner-parens",
            "i(",
            "Dentro de paréntesis",
            "Selecciona el contenido entre paréntesis",
            TextObject,
            Surgeon,
        ),
        command(
            "around-parens",
            "a(",
            "Alrededor de paréntesis",
            "Selecciona paréntesis y contenido",
            TextObject,
            Surgeon,
        ),
        command(
            "inner-brackets",
            "i[",
            "Dentro de corchetes",
            "Selecciona el contenido entre corchetes",
            TextObject,
            Surgeon,
        ),
        command(
            "around-brackets",
            "a[",
            "Alrededor de corchetes",
            "Selecciona corchetes y contenido",
            TextObject,
            Surgeon,
        ),
        command(
            "visual-char",
            "v",
            "Visual por carácter",
            "Inicia una selección por caracteres",
            Visual,
            Surgeon,
        ),
        command(
            "visual-line",
            "V",
            "Visual por línea",
            "Inicia una selección de líneas completas",
            Visual,
            Surgeon,
        ),
        command(
            "search-forward",
            "/texto<Enter>",
            "Buscar delante",
            "Busca texto hacia delante",
            Search,
            Architect,
        ),
        command(
            "search-backward",
            "?texto<Enter>",
            "Buscar detrás",
            "Busca texto hacia atrás",
            Search,
            Architect,
        ),
        command(
            "next-match",
            "n",
            "Siguiente coincidencia",
            "Repite la búsqueda en su dirección",
            Repeat,
            Architect,
        ),
        command(
            "previous-match",
            "N",
            "Coincidencia anterior",
            "Repite la búsqueda en dirección contraria",
            Repeat,
            Architect,
        ),
        command(
            "file-start",
            "gg",
            "Inicio del archivo",
            "Salta a la primera línea",
            Jump,
            Architect,
        ),
        command(
            "file-end",
            "G",
            "Fin del archivo",
            "Salta a la última línea",
            Jump,
            Architect,
        ),
        command(
            "goto-line",
            "{n}G",
            "Ir a línea",
            "Salta a una línea por su número",
            Jump,
            Architect,
        ),
        command(
            "match-pair",
            "%",
            "Pareja sintáctica",
            "Salta entre paréntesis, corchetes o llaves",
            Jump,
            Architect,
        ),
        command(
            "paragraph-next",
            "}",
            "Párrafo siguiente",
            "Salta al siguiente bloque de texto",
            Jump,
            Architect,
        ),
        command(
            "paragraph-prev",
            "{",
            "Párrafo anterior",
            "Salta al bloque de texto anterior",
            Jump,
            Architect,
        ),
        command(
            "half-page-down",
            "<C-d>",
            "Media página abajo",
            "Desplaza la vista media página hacia abajo",
            View,
            Architect,
        ),
        command(
            "half-page-up",
            "<C-u>",
            "Media página arriba",
            "Desplaza la vista media página hacia arriba",
            View,
            Architect,
        ),
        command(
            "full-page-down",
            "<C-f>",
            "Página abajo",
            "Desplaza la vista una página hacia abajo",
            View,
            Architect,
        ),
        command(
            "full-page-up",
            "<C-b>",
            "Página arriba",
            "Desplaza la vista una página hacia arriba",
            View,
            Architect,
        ),
        command(
            "unnamed-register",
            "y/d/c + p/P",
            "Registro sin nombre",
            "Guarda implícitamente la última copia o eliminación para reutilizarla",
            Register,
            Wizard,
        ),
    ]
}

/// Los ejercicios se construyen como datos; no realizan E/S ni consultan progreso.
pub fn exercises() -> Vec<Exercise> {
    use Belt::*;

    vec![
        Exercise {
            id: "SUR-01",
            order: 1,
            belt: Survivor,
            title: "Ruta de imports",
            context: "Un módulo Rust tiene varios imports y necesitas llegar a `io`.",
            objective: "Baja dos líneas y avanza hasta la `i` de `io`.",
            initial_lines: &[
                "use std::env;",
                "use std::fs;",
                "use std::io;",
                "use std::path;",
            ],
            start: Position::new(0, 4),
            goal: Goal::Cursor {
                position: Position::new(2, 9),
                mode: Mode::Normal,
            },
            hint: "j baja; l avanza una columna.",
            solution: "jjlllll",
            skills: &["move-down", "move-right"],
            forbidden: &["flechas", "word-forward"],
            estimated_secs: 25,
            optimal_actions: 7,
        },
        Exercise {
            id: "SUR-02",
            order: 2,
            belt: Survivor,
            title: "Regreso al origen",
            context: "Terminaste de inspeccionar el último import.",
            objective: "Vuelve a `std` en la segunda línea usando solo arriba e izquierda.",
            initial_lines: &[
                "use std::env;",
                "use std::fs;",
                "use std::io;",
                "use std::path;",
            ],
            start: Position::new(3, 9),
            goal: Goal::Cursor {
                position: Position::new(1, 4),
                mode: Mode::Normal,
            },
            hint: "k sube; h retrocede una columna.",
            solution: "kkhhhhh",
            skills: &["move-up", "move-left"],
            forbidden: &["flechas", "word-backward"],
            estimated_secs: 25,
            optimal_actions: 7,
        },
        Exercise {
            id: "SUR-03",
            order: 3,
            belt: Survivor,
            title: "Salto por tokens",
            context: "Estás leyendo una asignación en código de producción.",
            objective: "Llega al final de `max_attempts` sin recorrer carácter por carácter.",
            initial_lines: &["let mut retries = max_attempts;"],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(0, 29),
                mode: Mode::Normal,
            },
            hint: "w llega al inicio del siguiente token; e termina la palabra.",
            solution: "wwwwe",
            skills: &["word-forward", "word-end"],
            forbidden: &["move-left", "move-right", "flechas"],
            estimated_secs: 30,
            optimal_actions: 5,
        },
        Exercise {
            id: "SUR-04",
            order: 4,
            belt: Survivor,
            title: "Un paso atrás",
            context: "El cursor quedó al final de un acceso a campo.",
            objective: "Regresa al inicio de `body`.",
            initial_lines: &["return response.body;"],
            start: Position::new(0, 19),
            goal: Goal::Cursor {
                position: Position::new(0, 16),
                mode: Mode::Normal,
            },
            hint: "b busca el inicio de la palabra anterior o actual.",
            solution: "b",
            skills: &["word-backward"],
            forbidden: &["move-left", "flechas"],
            estimated_secs: 20,
            optimal_actions: 1,
        },
        Exercise {
            id: "SUR-05",
            order: 5,
            belt: Survivor,
            title: "Preparar el arranque",
            context: "Una función necesita visibilidad, inicialización y un punto y coma.",
            objective: "Haz pública la función, abre su cuerpo con `init();` y completa `ready`.",
            initial_lines: &["fn boot() {", "}", "let ready = true"],
            start: Position::new(0, 3),
            goal: Goal::Buffer {
                lines: &["pub fn boot() {", "    init();", "}", "let ready = true;"],
                mode: Mode::Normal,
            },
            hint: "I inserta al inicio, o abre debajo y A añade al final; vuelve con Esc.",
            solution: "Ipub <Esc>o    init();<Esc>jjA;<Esc>",
            skills: &[
                "insert-line-start",
                "open-below",
                "append-line-end",
                "escape",
                "move-down",
            ],
            forbidden: &["flechas"],
            estimated_secs: 90,
            optimal_actions: 5,
        },
        Exercise {
            id: "SNI-01",
            order: 6,
            belt: Sniper,
            title: "Punto de acceso",
            context: "Una cadena de métodos empieza lejos del cursor.",
            objective: "Salta exactamente al primer punto.",
            initial_lines: &["config.database.connect(timeout);"],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(0, 6),
                mode: Mode::Normal,
            },
            hint: "f aterriza sobre el carácter buscado.",
            solution: "f.",
            skills: &["find-forward"],
            forbidden: &["move-right", "word-forward", "flechas"],
            estimated_secs: 20,
            optimal_actions: 1,
        },
        Exercise {
            id: "SNI-02",
            order: 7,
            belt: Sniper,
            title: "Hasta el límite",
            context: "Quieres quedar dentro de una llamada antes de su cierre.",
            objective: "Salta al último carácter de `payload`, justo antes de `)`.",
            initial_lines: &["response.send(payload);"],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(0, 20),
                mode: Mode::Normal,
            },
            hint: "t se detiene una posición antes del carácter buscado.",
            solution: "t)",
            skills: &["till-forward"],
            forbidden: &["move-right", "word-forward", "flechas"],
            estimated_secs: 20,
            optimal_actions: 1,
        },
        Exercise {
            id: "SNI-03",
            order: 8,
            belt: Sniper,
            title: "Paréntesis a contracorriente",
            context: "Estás al final de una expresión ya larga.",
            objective: "Encuentra hacia atrás el paréntesis de `parse`.",
            initial_lines: &["let value = parse(raw);"],
            start: Position::new(0, 22),
            goal: Goal::Cursor {
                position: Position::new(0, 17),
                mode: Mode::Normal,
            },
            hint: "F busca hacia atrás y cae sobre el carácter.",
            solution: "F(",
            skills: &["find-backward"],
            forbidden: &["move-left", "word-backward", "flechas"],
            estimated_secs: 20,
            optimal_actions: 1,
        },
        Exercise {
            id: "SNI-04",
            order: 9,
            belt: Sniper,
            title: "Después del separador",
            context: "Una llamada quedó detrás del cursor.",
            objective: "Queda justo después del punto, sobre la `g` de `get`.",
            initial_lines: &["cache.get(user_id);"],
            start: Position::new(0, 18),
            goal: Goal::Cursor {
                position: Position::new(0, 6),
                mode: Mode::Normal,
            },
            hint: "T busca hacia atrás y se detiene después del objetivo.",
            solution: "T.",
            skills: &["till-backward"],
            forbidden: &["move-left", "word-backward", "flechas"],
            estimated_secs: 20,
            optimal_actions: 1,
        },
        Exercise {
            id: "SNI-05",
            order: 10,
            belt: Sniper,
            title: "Limpieza de línea",
            context: "Un TODO temporal debe convertirse en texto plano sin perder la sangría.",
            objective: "Borra el `;` final y los dos `/`, conservando los espacios.",
            initial_lines: &["    // TODO: remove debug;"],
            start: Position::new(0, 13),
            goal: Goal::BufferAndCursor {
                lines: &["     TODO: remove debug"],
                position: Position::new(0, 4),
                mode: Mode::Normal,
            },
            hint: "$ llega al final, 0 al inicio absoluto y ^ al primer texto.",
            solution: "$x0^xx",
            skills: &["line-end", "delete-char", "line-start", "first-nonblank"],
            forbidden: &["move-left", "move-right", "flechas"],
            estimated_secs: 45,
            optimal_actions: 6,
        },
        Exercise {
            id: "REF-01",
            order: 11,
            belt: Refactorer,
            title: "Renombrado local",
            context: "El nombre `count` ya no expresa lo que acumula la variable.",
            objective: "Cambia `count` por `total`.",
            initial_lines: &["let count = 0;"],
            start: Position::new(0, 4),
            goal: Goal::Buffer {
                lines: &["let total = 0;"],
                mode: Mode::Normal,
            },
            hint: "Piensa verbo + objeto: cambiar palabra.",
            solution: "cwtotal<Esc>",
            skills: &["change-op", "word-forward", "escape"],
            forbidden: &["delete-char", "flechas"],
            estimated_secs: 35,
            optimal_actions: 1,
        },
        Exercise {
            id: "REF-02",
            order: 12,
            belt: Refactorer,
            title: "Tres elementos menos",
            context: "Una colección de prueba debe quedarse con un único valor.",
            objective: "Elimina `one, two, three,` con un operador y un conteo.",
            initial_lines: &["let values = vec![one, two, three, four];"],
            start: Position::new(0, 18),
            goal: Goal::Buffer {
                lines: &["let values = vec![four];"],
                mode: Mode::Normal,
            },
            hint: "Un conteo puede ir entre d y w.",
            solution: "d3w",
            skills: &["delete-op", "count", "word-forward"],
            forbidden: &["delete-char", "flechas"],
            estimated_secs: 35,
            optimal_actions: 1,
        },
        Exercise {
            id: "REF-03",
            order: 13,
            belt: Refactorer,
            title: "Firma sin argumentos",
            context: "Una función conservará su nombre, pero ya no recibe parámetros.",
            objective: "Borra los argumentos sin borrar el paréntesis de cierre.",
            initial_lines: &["fn process(a, b, c)"],
            start: Position::new(0, 11),
            goal: Goal::Buffer {
                lines: &["fn process()"],
                mode: Mode::Normal,
            },
            hint: "d es el verbo; t) define hasta dónde, sin incluir `)`.",
            solution: "dt)",
            skills: &["delete-op", "till-forward"],
            forbidden: &["delete-char", "flechas"],
            estimated_secs: 30,
            optimal_actions: 1,
        },
        Exercise {
            id: "REF-04",
            order: 14,
            belt: Refactorer,
            title: "Duplicar una preparación",
            context: "La segunda llamada necesita la misma preparación que la primera.",
            objective: "Copia la primera línea y pégala debajo de `run(primary);`.",
            initial_lines: &["let primary = build();", "run(primary);"],
            start: Position::new(0, 0),
            goal: Goal::Buffer {
                lines: &[
                    "let primary = build();",
                    "run(primary);",
                    "let primary = build();",
                ],
                mode: Mode::Normal,
            },
            hint: "yy copia la línea; p la pega debajo.",
            solution: "yyjp",
            skills: &["yank-line", "move-down", "paste-after"],
            forbidden: &["insert-before"],
            estimated_secs: 35,
            optimal_actions: 3,
        },
        Exercise {
            id: "REF-05",
            order: 15,
            belt: Refactorer,
            title: "Silenciar trazas",
            context: "Dos trazas de depuración contiguas llegaron a producción.",
            objective: "Elimina ambas líneas `debug!` con una sola orden contada.",
            initial_lines: &[
                "trace!(\"enter\");",
                "debug!(\"cache\");",
                "debug!(\"payload\");",
                "run();",
            ],
            start: Position::new(1, 0),
            goal: Goal::BufferAndCursor {
                lines: &["trace!(\"enter\");", "run();"],
                position: Position::new(1, 0),
                mode: Mode::Normal,
            },
            hint: "El conteo también multiplica órdenes de línea completa.",
            solution: "2dd",
            skills: &["count", "delete-line"],
            forbidden: &["delete-char", "flechas"],
            estimated_secs: 30,
            optimal_actions: 1,
        },
        Exercise {
            id: "SURG-01",
            order: 16,
            belt: Surgeon,
            title: "Dentro de las comillas",
            context: "Un flag conserva las comillas, pero cambia de valor.",
            objective: "Reemplaza solo `debug` por `release`.",
            initial_lines: &["let mode = \"debug\";"],
            start: Position::new(0, 14),
            goal: Goal::Buffer {
                lines: &["let mode = \"release\";"],
                mode: Mode::Normal,
            },
            hint: "ci\" cambia el interior de las comillas desde cualquier punto.",
            solution: "ci\"release<Esc>",
            skills: &["change-op", "inner-quotes", "escape"],
            forbidden: &["find-forward", "delete-char", "flechas"],
            estimated_secs: 35,
            optimal_actions: 1,
        },
        Exercise {
            id: "SURG-02",
            order: 17,
            belt: Surgeon,
            title: "Condición completa",
            context: "Una guarda compleja se sustituyó por un estado calculado.",
            objective: "Cambia el interior de los paréntesis por `enabled`.",
            initial_lines: &["if (ready && cached) {"],
            start: Position::new(0, 6),
            goal: Goal::Buffer {
                lines: &["if (enabled) {"],
                mode: Mode::Normal,
            },
            hint: "ci( opera dentro de los paréntesis sin tocarlos.",
            solution: "ci(enabled<Esc>",
            skills: &["change-op", "inner-parens", "escape"],
            forbidden: &["delete-char", "flechas"],
            estimated_secs: 40,
            optimal_actions: 1,
        },
        Exercise {
            id: "SURG-03",
            order: 18,
            belt: Surgeon,
            title: "Palabra bajo bisturí",
            context: "Una devolución usa un identificador demasiado específico.",
            objective: "Cambia `temporary_value` por `result` desde el centro de la palabra.",
            initial_lines: &["return temporary_value;"],
            start: Position::new(0, 12),
            goal: Goal::Buffer {
                lines: &["return result;"],
                mode: Mode::Normal,
            },
            hint: "ciw no necesita que estés al inicio de la palabra.",
            solution: "ciwresult<Esc>",
            skills: &["change-op", "inner-word", "escape"],
            forbidden: &["word-backward", "delete-char", "flechas"],
            estimated_secs: 35,
            optimal_actions: 1,
        },
        Exercise {
            id: "SURG-04",
            order: 19,
            belt: Surgeon,
            title: "Llamada sin carga",
            context: "El logger ya obtiene su contexto de forma implícita.",
            objective: "Elimina `(payload)` incluyendo ambos paréntesis.",
            initial_lines: &["logger.debug(payload);"],
            start: Position::new(0, 15),
            goal: Goal::Buffer {
                lines: &["logger.debug;"],
                mode: Mode::Normal,
            },
            hint: "da( elimina alrededor de los paréntesis.",
            solution: "da(",
            skills: &["delete-op", "around-parens"],
            forbidden: &["delete-char", "flechas"],
            estimated_secs: 30,
            optimal_actions: 1,
        },
        Exercise {
            id: "SURG-05",
            order: 20,
            belt: Surgeon,
            title: "Escisión visual",
            context: "Dos trazas temporales quedaron dentro de una rama estable.",
            objective: "Selecciona las dos líneas de diagnóstico completas y elimínalas.",
            initial_lines: &[
                "if ready {",
                "    debug_state();",
                "    debug_cache();",
                "    work();",
                "}",
            ],
            start: Position::new(1, 0),
            goal: Goal::Buffer {
                lines: &["if ready {", "    work();", "}"],
                mode: Mode::Normal,
            },
            hint: "V selecciona la línea; j amplía la selección y d aplica la eliminación.",
            solution: "Vjd",
            skills: &["visual-line", "move-down", "delete-op"],
            forbidden: &["delete-line", "flechas"],
            estimated_secs: 35,
            optimal_actions: 2,
        },
        Exercise {
            id: "ARC-01",
            order: 21,
            belt: Architect,
            title: "Segundo TODO",
            context: "Un archivo contiene dos tareas pendientes separadas.",
            objective: "Busca `TODO` y avanza a su segunda coincidencia.",
            initial_lines: &[
                "fn boot() {",
                "    log(\"TODO: config\");",
                "    warm_up();",
                "    log(\"TODO: cache\");",
                "}",
            ],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(3, 9),
                mode: Mode::Normal,
            },
            hint: "/ inicia la búsqueda; Enter confirma y n repite.",
            solution: "/TODO<Enter>n",
            skills: &["search-forward", "next-match"],
            forbidden: &["move-down", "goto-line", "flechas"],
            estimated_secs: 40,
            optimal_actions: 2,
        },
        Exercise {
            id: "ARC-02",
            order: 22,
            belt: Architect,
            title: "De vuelta a cabecera",
            context: "Después de leer una función quieres revisar sus imports.",
            objective: "Salta a la primera línea y primera columna.",
            initial_lines: &[
                "use crate::api;",
                "",
                "fn fetch() {",
                "    request();",
                "    Ok(())",
            ],
            start: Position::new(4, 4),
            goal: Goal::Cursor {
                position: Position::new(0, 0),
                mode: Mode::Normal,
            },
            hint: "Dos g minúsculas significan inicio del archivo.",
            solution: "gg",
            skills: &["file-start"],
            forbidden: &["move-up", "goto-line", "flechas"],
            estimated_secs: 20,
            optimal_actions: 1,
        },
        Exercise {
            id: "ARC-03",
            order: 23,
            belt: Architect,
            title: "Llave compañera",
            context: "Necesitas comprobar dónde termina una función anidada.",
            objective: "Salta de la llave de apertura a su cierre correspondiente.",
            initial_lines: &["fn main() {", "    prepare();", "    execute();", "}"],
            start: Position::new(0, 10),
            goal: Goal::Cursor {
                position: Position::new(3, 0),
                mode: Mode::Normal,
            },
            hint: "% conecta parejas sintácticas.",
            solution: "%",
            skills: &["match-pair"],
            forbidden: &["move-down", "file-end", "flechas"],
            estimated_secs: 25,
            optimal_actions: 1,
        },
        Exercise {
            id: "ARC-04",
            order: 24,
            belt: Architect,
            title: "Siguiente bloque",
            context: "Dos unidades lógicas están separadas por una línea vacía.",
            objective: "Salta al inicio del siguiente párrafo de código.",
            initial_lines: &[
                "fn parse() {}",
                "let x = 1;",
                "",
                "fn render() {}",
                "let y = 2;",
            ],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(3, 0),
                mode: Mode::Normal,
            },
            hint: "} avanza por bloques separados por líneas vacías.",
            solution: "}",
            skills: &["paragraph-next"],
            forbidden: &["move-down", "search-forward", "flechas"],
            estimated_secs: 25,
            optimal_actions: 1,
        },
        Exercise {
            id: "ARC-05",
            order: 25,
            belt: Architect,
            title: "Final bajo control",
            context: "Un archivo de servicio termina con su valor de retorno.",
            objective: "Ve al final, inspecciona media página arriba y regresa abajo.",
            initial_lines: &[
                "fn service() {",
                "    authenticate();",
                "    authorize();",
                "    fetch();",
                "    transform();",
                "    persist();",
                "    notify();",
                "return status;",
            ],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(7, 0),
                mode: Mode::Normal,
            },
            hint: "G salta al final; Ctrl-u y Ctrl-d recorren media pantalla.",
            solution: "G<C-u><C-d>",
            skills: &["file-end", "half-page-up", "half-page-down"],
            forbidden: &["move-down", "goto-line", "flechas"],
            estimated_secs: 30,
            optimal_actions: 3,
        },
        Exercise {
            id: "WIZ-01",
            order: 26,
            belt: Wizard,
            title: "Memoria inmediata",
            context: "Una preparación debe reutilizarse unas líneas más abajo.",
            objective: "Copia la primera línea y pégala debajo de la segunda.",
            initial_lines: &["let alpha = 1;", "let beta = 2;"],
            start: Position::new(0, 0),
            goal: Goal::Buffer {
                lines: &["let alpha = 1;", "let beta = 2;", "let alpha = 1;"],
                mode: Mode::Normal,
            },
            hint: "yy llena el registro sin nombre; p reutiliza su contenido.",
            solution: "yyjp",
            skills: &["unnamed-register", "yank-line", "move-down", "paste-after"],
            forbidden: &["insert-before"],
            estimated_secs: 35,
            optimal_actions: 3,
        },
        Exercise {
            id: "WIZ-02",
            order: 27,
            belt: Wizard,
            title: "Mover sin Insert",
            context: "La declaración `beta` debe ir después de `gamma`.",
            objective: "Corta la línea `beta` y pégala al final usando el registro implícito.",
            initial_lines: &["let alpha = 1;", "let beta = 2;", "let gamma = 3;"],
            start: Position::new(1, 0),
            goal: Goal::Buffer {
                lines: &["let alpha = 1;", "let gamma = 3;", "let beta = 2;"],
                mode: Mode::Normal,
            },
            hint: "dd también llena el registro sin nombre; p pega la línea cortada.",
            solution: "ddp",
            skills: &["unnamed-register", "delete-line", "paste-after"],
            forbidden: &["insert-before"],
            estimated_secs: 35,
            optimal_actions: 2,
        },
        Exercise {
            id: "WIZ-03",
            order: 28,
            belt: Wizard,
            title: "Pegar antes",
            context: "La configuración de `beta` también debe encabezar el archivo.",
            objective: "Copia `beta` y pégala por encima de `alpha`.",
            initial_lines: &["let alpha = 1;", "let beta = 2;"],
            start: Position::new(1, 0),
            goal: Goal::Buffer {
                lines: &["let beta = 2;", "let alpha = 1;", "let beta = 2;"],
                mode: Mode::Normal,
            },
            hint: "P pega antes o encima, a diferencia de p.",
            solution: "yykP",
            skills: &["unnamed-register", "yank-line", "move-up", "paste-before"],
            forbidden: &["insert-before"],
            estimated_secs: 35,
            optimal_actions: 3,
        },
        Exercise {
            id: "WIZ-04",
            order: 29,
            belt: Wizard,
            title: "Eco de búsqueda",
            context: "Tres TODO requieren inspección sin volver a escribir la consulta.",
            objective: "Busca TODO, recorre las tres coincidencias y vuelve a la segunda.",
            initial_lines: &[
                "fn run() {",
                "TODO auth",
                "prepare();",
                "TODO cache",
                "execute();",
                "TODO metrics",
                "}",
            ],
            start: Position::new(0, 0),
            goal: Goal::Cursor {
                position: Position::new(3, 0),
                mode: Mode::Normal,
            },
            hint: "Después de /TODO, n avanza y N invierte la búsqueda.",
            solution: "/TODO<Enter>nnN",
            skills: &["search-forward", "next-match", "previous-match"],
            forbidden: &["move-down", "goto-line", "flechas"],
            estimated_secs: 45,
            optimal_actions: 4,
        },
    ]
}

pub fn exercises_for_belt(belt: Belt) -> Vec<Exercise> {
    exercises()
        .into_iter()
        .filter(|exercise| exercise.belt == belt)
        .collect()
}

/// Valida referencias, orden y coordenadas del currículo.
///
/// Devuelve todos los errores juntos para que una edición de contenido no se
/// convierta en un ciclo lento de “arreglar uno y volver a compilar”.
pub fn validate_curriculum() -> Result<(), Vec<String>> {
    let commands = command_catalog();
    let lessons = exercises();
    let mut errors = Vec::new();

    let mut command_ids = HashSet::new();
    for command in &commands {
        if command.id.trim().is_empty() {
            errors.push("hay un comando sin id".to_string());
        }
        if !command_ids.insert(command.id) {
            errors.push(format!("id de comando duplicado: {}", command.id));
        }
        if command.keys.trim().is_empty() || command.name.trim().is_empty() {
            errors.push(format!("comando incompleto: {}", command.id));
        }
    }

    let command_by_id: HashMap<_, _> = commands
        .iter()
        .map(|command| (command.id, command))
        .collect();
    let mut exercise_ids = HashSet::new();
    let mut orders = HashSet::new();

    for (index, exercise) in lessons.iter().enumerate() {
        if exercise.id.trim().is_empty() {
            errors.push(format!("el ejercicio #{} no tiene id", index + 1));
        }
        if !exercise_ids.insert(exercise.id) {
            errors.push(format!("id de ejercicio duplicado: {}", exercise.id));
        }
        if !orders.insert(exercise.order) {
            errors.push(format!("orden de ejercicio duplicado: {}", exercise.order));
        }
        if exercise.order as usize != index + 1 {
            errors.push(format!(
                "{} tiene orden {}, se esperaba {}",
                exercise.id,
                exercise.order,
                index + 1
            ));
        }
        if exercise.title.trim().is_empty()
            || exercise.context.trim().is_empty()
            || exercise.objective.trim().is_empty()
            || exercise.hint.trim().is_empty()
            || exercise.solution.trim().is_empty()
        {
            errors.push(format!("{} tiene texto pedagógico incompleto", exercise.id));
        }
        if exercise.initial_lines.is_empty() {
            errors.push(format!("{} no tiene buffer inicial", exercise.id));
            continue;
        }
        if !position_is_valid(exercise.initial_lines, exercise.start, Mode::Normal) {
            errors.push(format!(
                "{} tiene cursor inicial fuera del buffer",
                exercise.id
            ));
        }
        if exercise.skills.is_empty() {
            errors.push(format!("{} no declara skills", exercise.id));
        }
        let mut seen_skills = HashSet::new();
        for skill in exercise.skills {
            if !seen_skills.insert(*skill) {
                errors.push(format!("{} repite el skill {}", exercise.id, skill));
            }
            match command_by_id.get(skill) {
                None => errors.push(format!(
                    "{} referencia un skill inexistente: {}",
                    exercise.id, skill
                )),
                Some(command) if command.introduced_in > exercise.belt => errors.push(format!(
                    "{} usa {} antes de su cinturón",
                    exercise.id, skill
                )),
                Some(_) => {}
            }
        }
        if exercise.estimated_secs == 0 || exercise.optimal_actions == 0 {
            errors.push(format!(
                "{} tiene duración o acciones inválidas",
                exercise.id
            ));
        }

        match &exercise.goal {
            Goal::Cursor { position, mode } => {
                if !position_is_valid(exercise.initial_lines, *position, *mode) {
                    errors.push(format!(
                        "{} tiene cursor objetivo fuera del buffer",
                        exercise.id
                    ));
                }
            }
            Goal::Buffer { lines, .. } => {
                if lines.is_empty() {
                    errors.push(format!("{} tiene buffer objetivo vacío", exercise.id));
                }
            }
            Goal::BufferAndCursor {
                lines,
                position,
                mode,
            } => {
                if lines.is_empty() {
                    errors.push(format!("{} tiene buffer objetivo vacío", exercise.id));
                } else if !position_is_valid(lines, *position, *mode) {
                    errors.push(format!(
                        "{} tiene cursor objetivo fuera del buffer",
                        exercise.id
                    ));
                }
            }
        }

        if exercise
            .goal
            .is_met(&exercise.initial_buffer(), exercise.start, Mode::Normal)
        {
            errors.push(format!("{} ya empieza resuelto", exercise.id));
        }
    }

    for belt in Belt::all() {
        if !lessons.iter().any(|exercise| exercise.belt == *belt) {
            errors.push(format!("el cinturón {:?} no tiene ejercicios", belt));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn position_is_valid(lines: &[&str], position: Position, mode: Mode) -> bool {
    let Some(line) = lines.get(position.row) else {
        return false;
    };
    let width = line.chars().count();
    if width == 0 {
        return position.col == 0;
    }
    if mode == Mode::Insert {
        position.col <= width
    } else {
        position.col < width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curriculum_is_internally_consistent() {
        if let Err(errors) = validate_curriculum() {
            panic!("currículo inválido:\n{}", errors.join("\n"));
        }
    }

    #[test]
    fn all_belts_have_ordered_metadata_and_content() {
        assert_eq!(Belt::all().len(), 6);
        for (index, belt) in Belt::all().iter().enumerate() {
            assert_eq!(belt.metadata().order as usize, index + 1);
            assert!(!belt.metadata().focus.is_empty());
            assert!(!exercises_for_belt(*belt).is_empty());
        }
    }

    #[test]
    fn campaign_is_broad_but_session_sized() {
        let lessons = exercises();
        assert!((28..=32).contains(&lessons.len()));
        assert_eq!(lessons.first().map(|item| item.belt), Some(Belt::Survivor));
        assert_eq!(lessons.last().map(|item| item.belt), Some(Belt::Wizard));
    }

    #[test]
    fn goal_checks_buffer_cursor_and_mode() {
        let buffer = vec!["let ok = true;".chars().collect::<Vec<_>>()];

        let cursor_goal = Goal::Cursor {
            position: Position::new(0, 4),
            mode: Mode::Normal,
        };
        assert!(cursor_goal.is_met(&buffer, Position::new(0, 4), Mode::Normal));
        assert!(!cursor_goal.is_met(&buffer, Position::new(0, 5), Mode::Normal));

        let buffer_goal = Goal::Buffer {
            lines: &["let ok = true;"],
            mode: Mode::Normal,
        };
        assert!(buffer_goal.is_met(&buffer, Position::new(0, 0), Mode::Normal));
        assert!(!buffer_goal.is_met(&buffer, Position::new(0, 0), Mode::Insert));

        let combined_goal = Goal::BufferAndCursor {
            lines: &["let ok = true;"],
            position: Position::new(0, 3),
            mode: Mode::Normal,
        };
        assert!(combined_goal.is_met(&buffer, Position::new(0, 3), Mode::Normal));
        assert!(!combined_goal.is_met(&buffer, Position::new(0, 2), Mode::Normal));
    }

    #[test]
    fn every_exercise_skill_exists_in_the_catalog() {
        let known: HashSet<_> = command_catalog()
            .into_iter()
            .map(|command| command.id)
            .collect();
        for exercise in exercises() {
            for skill in exercise.skills {
                assert!(known.contains(skill), "{} -> {}", exercise.id, skill);
            }
        }
    }

    #[test]
    fn validation_and_tests_never_need_persistence() {
        // Este test documenta deliberadamente la frontera: crear, convertir y
        // validar ejercicios solo opera sobre memoria local.
        for exercise in exercises() {
            let initial = exercise.initial_buffer();
            assert_eq!(initial.len(), exercise.initial_lines.len());
        }
    }
}
