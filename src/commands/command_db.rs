use crate::commands::command::{Command, CommandCategory, Exercise, Level};

pub struct CommandDatabase {
    commands: Vec<Command>,
    exercises: Vec<Exercise>,
}

impl CommandDatabase {
    pub fn new() -> Self {
        Self {
            commands: Self::init_commands(),
            exercises: Self::init_exercises(),
        }
    }

    fn init_exercises() -> Vec<Exercise> {
        vec![
            // --- LEVEL 1: SURVIVOR ---
            
            // 1.1 Navigation
            Exercise {
                id: "N1".to_string(),
                level: Level::Survivor,
                title: "The Basics".to_string(),
                description: "Move cursor down to the 'io' import".to_string(),
                context: "Navigation in imports".to_string(),
                initial_lines: vec![
                    "use std::env;".to_string(),
                    "use std::fs;".to_string(),
                    "use std::io;".to_string(),
                    "use std::path;".to_string(),
                ],
                initial_cursor: (0, 4), // on 'std'
                expected_lines: None, // No editing
                expected_cursor: Some((2, 4)),
                hint: "Use 'j' to move down".to_string(),
                solution_keys: "jj".to_string(),
            },
            Exercise {
                id: "N2".to_string(),
                level: Level::Survivor,
                title: "Lateral Move".to_string(),
                description: "Move to the last argument 'z'".to_string(),
                context: "Function arguments".to_string(),
                initial_lines: vec!["fn calculate(x: i32, y: i32, z: i32) {".to_string()],
                initial_cursor: (0, 13), // on 'x'
                expected_lines: None,
                expected_cursor: Some((0, 29)), // on 'z'
                hint: "Use 'l' to move right".to_string(),
                solution_keys: "llllllllllllllll".to_string(), // In real app, w is better, but this drills basics
            },
            Exercise {
                id: "N3".to_string(),
                level: Level::Survivor,
                title: "Word Jump".to_string(),
                description: "Jump words to reach the number 0".to_string(),
                context: "Variable initialization".to_string(),
                initial_lines: vec!["let mut counter = 0;".to_string()],
                initial_cursor: (0, 0),
                expected_lines: None,
                expected_cursor: Some((0, 18)),
                hint: "Use 'w' to jump by words".to_string(),
                solution_keys: "wwww".to_string(),
            },
            Exercise {
                id: "N4".to_string(),
                level: Level::Survivor,
                title: "Backtrack".to_string(),
                description: "Go back to the start of 'return'".to_string(),
                context: "Return statement".to_string(),
                initial_lines: vec!["    return result;".to_string()],
                initial_cursor: (0, 16), // on ';'
                expected_lines: None,
                expected_cursor: Some((0, 4)), // start of return
                hint: "Use 'b' to jump back".to_string(),
                solution_keys: "bb".to_string(),
            },

            // 1.2 Insertion
            Exercise {
                id: "I1".to_string(),
                level: Level::Survivor,
                title: "Append End".to_string(),
                description: "Add a semicolon at the end of the line".to_string(),
                context: "Fixing syntax errors".to_string(),
                initial_lines: vec!["let x = 5".to_string()],
                initial_cursor: (0, 0),
                expected_lines: Some(vec!["let x = 5;".to_string()]),
                expected_cursor: None,
                hint: "Use 'A' to append at end of line".to_string(),
                solution_keys: "A;".to_string(),
            },
            Exercise {
                id: "I2".to_string(),
                level: Level::Survivor,
                title: "Insert Start".to_string(),
                description: "Add 'pub ' at the start of the line".to_string(),
                context: "Making function public".to_string(),
                initial_lines: vec!["fn main()".to_string()],
                initial_cursor: (0, 3),
                expected_lines: Some(vec!["pub fn main()".to_string()]),
                expected_cursor: None,
                hint: "Use 'I' to insert at line start".to_string(),
                solution_keys: "Ipub ".to_string(),
            },
            Exercise {
                id: "I3".to_string(),
                level: Level::Survivor,
                title: "Open Below".to_string(),
                description: "Open a new line below to start coding".to_string(),
                context: "Starting function body".to_string(),
                initial_lines: vec!["fn main() {".to_string(), "}".to_string()],
                initial_cursor: (0, 0),
                expected_lines: Some(vec!["fn main() {".to_string(), "".to_string(), "}".to_string()]),
                expected_cursor: None,
                hint: "Use 'o' to open a line below".to_string(),
                solution_keys: "o".to_string(), 
            },

            // --- LEVEL 2: SNIPER ---
            
            Exercise {
                id: "S1".to_string(),
                level: Level::Sniper,
                title: "Find Char".to_string(),
                description: "Jump directly to the dot '.'".to_string(),
                context: "Method chaining".to_string(),
                initial_lines: vec!["user.get_id();".to_string()],
                initial_cursor: (0, 0),
                expected_lines: None,
                expected_cursor: Some((0, 4)),
                hint: "Use 'f' to find a character forward".to_string(),
                solution_keys: "f.".to_string(),
            },
            Exercise {
                id: "S2".to_string(),
                level: Level::Sniper,
                title: "Find & Edit".to_string(),
                description: "Jump to ':' to change type".to_string(),
                context: "Type definition".to_string(),
                initial_lines: vec!["const MAX: u32 = 100;".to_string()],
                initial_cursor: (0, 0),
                expected_lines: None,
                expected_cursor: Some((0, 9)),
                hint: "Use 'f' to find ':'".to_string(),
                solution_keys: "f:".to_string(),
            },
            Exercise {
                id: "S3".to_string(),
                level: Level::Sniper,
                title: "Till Char".to_string(),
                description: "Jump right before the closing ')'".to_string(),
                context: "Inside parenthesis".to_string(),
                initial_lines: vec!["(\"text content\")".to_string()],
                initial_cursor: (0, 1),
                expected_lines: None,
                expected_cursor: Some((0, 14)), // char before )
                hint: "Use 't' to jump 'till' a character".to_string(),
                solution_keys: "t)".to_string(),
            },
            
            // Line Mastery
            Exercise {
                id: "L1".to_string(),
                level: Level::Sniper,
                title: "Hard Start".to_string(),
                description: "Go to the first non-blank character (ignore spaces)".to_string(),
                context: "Indented code".to_string(),
                initial_lines: vec!["    let x = 1;".to_string()],
                initial_cursor: (0, 0),
                expected_lines: None,
                expected_cursor: Some((0, 4)), // First 'l'
                hint: "Use '^' to go to start of text".to_string(),
                solution_keys: "^".to_string(),
            },

            // --- LEVEL 3: REFACTORER ---
            
            Exercise {
                id: "R1".to_string(),
                level: Level::Refactorer,
                title: "Change Word".to_string(),
                description: "Change the variable name 'count' to 'total'".to_string(),
                context: "Refactoring variable names".to_string(),
                initial_lines: vec!["let count = 0;".to_string()],
                initial_cursor: (0, 4), // on 'c'
                expected_lines: Some(vec!["let total = 0;".to_string()]),
                expected_cursor: None,
                hint: "Use 'cw' to change a word".to_string(),
                solution_keys: "cwtotal\u{1b}".to_string(),
            },
            Exercise {
                id: "R2".to_string(),
                level: Level::Refactorer,
                title: "Delete Word".to_string(),
                description: "Delete the unnecessary 'mut' keyword".to_string(),
                context: "Cleaning up code".to_string(),
                initial_lines: vec!["let mut name = \"Vim\";".to_string()],
                initial_cursor: (0, 4), // on 'm'
                expected_lines: Some(vec!["let name = \"Vim\";".to_string()]),
                expected_cursor: None,
                hint: "Use 'dw' to delete a word".to_string(),
                solution_keys: "dw".to_string(),
            },
            Exercise {
                id: "R3".to_string(),
                level: Level::Refactorer,
                title: "Change to End".to_string(),
                description: "Change the string content to 'Master'".to_string(),
                context: "Updating string values".to_string(),
                initial_lines: vec!["let msg = \"Hello World\";".to_string()],
                initial_cursor: (0, 11), // on 'H'
                expected_lines: Some(vec!["let msg = \"Master\";".to_string()]),
                expected_cursor: None,
                hint: "Use 'c' with a motion or 'c$'".to_string(),
                solution_keys: "c$Master\";\u{1b}".to_string(), 
            },
            Exercise {
                id: "R4".to_string(),
                level: Level::Refactorer,
                title: "Clean Line".to_string(),
                description: "Clear the entire line content to rewrite it".to_string(),
                context: "Major refactor".to_string(),
                initial_lines: vec!["// This logic is deprecated and needs to be replaced".to_string()],
                initial_cursor: (0, 10),
                expected_lines: Some(vec!["".to_string()]),
                expected_cursor: None,
                hint: "Use 'cc' to change the whole line".to_string(),
                solution_keys: "cc\u{1b}".to_string(),
            },
        ]
    }

    fn init_commands() -> Vec<Command> {
        vec![
            Command {
                name: "Move Left".to_string(),
                keybinding: "h".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor left".to_string(),
                level: Level::Survivor,
            },
            Command {
                name: "Move Down".to_string(),
                keybinding: "j".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor down".to_string(),
                level: Level::Survivor,
            },
            Command {
                name: "Move Up".to_string(),
                keybinding: "k".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor up".to_string(),
                level: Level::Survivor,
            },
            Command {
                name: "Move Right".to_string(),
                keybinding: "l".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor right".to_string(),
                level: Level::Survivor,
            },
            Command {
                name: "Word Forward".to_string(),
                keybinding: "w".to_string(),
                category: CommandCategory::Movement,
                description: "Move forward by word".to_string(),
                level: Level::Survivor,
            },
            Command {
                name: "Find Char".to_string(),
                keybinding: "f".to_string(),
                category: CommandCategory::Search,
                description: "Find character in line".to_string(),
                level: Level::Sniper,
            },
             Command {
                name: "Till Char".to_string(),
                keybinding: "t".to_string(),
                category: CommandCategory::Search,
                description: "Move till character in line".to_string(),
                level: Level::Sniper,
            },
        ]
    }

    pub fn get_exercises_by_level(&self, level: Level) -> Vec<&Exercise> {
        self.exercises.iter().filter(|e| e.level == level).collect()
    }

    pub fn get_all_exercises(&self) -> &[Exercise] {
        &self.exercises
    }

    pub fn get_command_by_keybinding(&self, key: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.keybinding == key)
    }

    pub fn get_levels(&self) -> Vec<(Level, Vec<Exercise>)> {
        let levels = vec![
            Level::Survivor,
            Level::Sniper,
            Level::Refactorer,
            Level::Surgeon,
            Level::Architect,
            Level::Wizard,
        ];

        levels
            .into_iter()
            .map(|level| {
                let exercises = self
                    .exercises
                    .iter()
                    .filter(|e| e.level == level)
                    .cloned()
                    .collect();
                (level, exercises)
            })
            .filter(|(_, exercises): &(_, Vec<Exercise>)| !exercises.is_empty())
            .collect()
    }
}

impl Default for CommandDatabase {
    fn default() -> Self {
        Self::new()
    }
}
