use crate::commands::command::{Command, CommandCategory, Level};

pub struct CommandDatabase {
    commands: Vec<Command>,
}

impl CommandDatabase {
    pub fn new() -> Self {
        Self {
            commands: Self::init_commands(),
        }
    }

    fn init_commands() -> Vec<Command> {
        vec![
            Command {
                name: "Move Left".to_string(),
                keybinding: "h".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor one character to the left".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Move Down".to_string(),
                keybinding: "j".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor one line down".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Move Up".to_string(),
                keybinding: "k".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor one line up".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Move Right".to_string(),
                keybinding: "l".to_string(),
                category: CommandCategory::Movement,
                description: "Move cursor one character to the right".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Insert Mode".to_string(),
                keybinding: "i".to_string(),
                category: CommandCategory::Edit,
                description: "Enter insert mode before cursor".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Append".to_string(),
                keybinding: "a".to_string(),
                category: CommandCategory::Edit,
                description: "Enter insert mode after cursor".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Insert Line Start".to_string(),
                keybinding: "I".to_string(),
                category: CommandCategory::Edit,
                description: "Insert at beginning of line".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Append Line End".to_string(),
                keybinding: "A".to_string(),
                category: CommandCategory::Edit,
                description: "Insert at end of line".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "New Line Below".to_string(),
                keybinding: "o".to_string(),
                category: CommandCategory::Edit,
                description: "Create new line below and enter insert mode".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "New Line Above".to_string(),
                keybinding: "O".to_string(),
                category: CommandCategory::Edit,
                description: "Create new line above and enter insert mode".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Delete Character".to_string(),
                keybinding: "x".to_string(),
                category: CommandCategory::Edit,
                description: "Delete character under cursor".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Delete Line".to_string(),
                keybinding: "dd".to_string(),
                category: CommandCategory::Edit,
                description: "Delete current line".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Yank Line".to_string(),
                keybinding: "yy".to_string(),
                category: CommandCategory::Edit,
                description: "Yank (copy) current line".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Paste Below".to_string(),
                keybinding: "p".to_string(),
                category: CommandCategory::Edit,
                description: "Paste after cursor".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Paste Above".to_string(),
                keybinding: "P".to_string(),
                category: CommandCategory::Edit,
                description: "Paste before cursor".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Undo".to_string(),
                keybinding: "u".to_string(),
                category: CommandCategory::Edit,
                description: "Undo last change".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Redo".to_string(),
                keybinding: "Ctrl-r".to_string(),
                category: CommandCategory::Edit,
                description: "Redo last undone change".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Word Forward".to_string(),
                keybinding: "w".to_string(),
                category: CommandCategory::Movement,
                description: "Move to start of next word".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Word Backward".to_string(),
                keybinding: "b".to_string(),
                category: CommandCategory::Movement,
                description: "Move to start of previous word".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Line End".to_string(),
                keybinding: "$".to_string(),
                category: CommandCategory::Movement,
                description: "Move to end of line".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "Line Start".to_string(),
                keybinding: "0".to_string(),
                category: CommandCategory::Movement,
                description: "Move to start of line".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "File Start".to_string(),
                keybinding: "gg".to_string(),
                category: CommandCategory::Movement,
                description: "Move to beginning of file".to_string(),
                level: Level::Beginner,
            },
            Command {
                name: "File End".to_string(),
                keybinding: "G".to_string(),
                category: CommandCategory::Movement,
                description: "Move to end of file".to_string(),
                level: Level::Beginner,
            },
        ]
    }

    pub fn get_commands_by_level(&self, level: Level) -> Vec<&Command> {
        self.commands.iter().filter(|c| c.level == level).collect()
    }

    pub fn get_all_commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn get_command_by_keybinding(&self, key: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.keybinding == key)
    }
}

impl Default for CommandDatabase {
    fn default() -> Self {
        Self::new()
    }
}
