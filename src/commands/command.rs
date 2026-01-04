#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    Survivor,   // Basic Motion & Edits
    Sniper,     // Horizontal Precision (f, t)
    Refactorer, // Change & Delete (cw, dt)
    Surgeon,    // Text Objects (ciw, yi")
    Architect,  // Multi-file & Search
    Wizard,     // Macros & Registers
}

#[derive(Debug, Clone)]
pub struct Exercise {
    pub id: String,
    pub level: Level,
    pub title: String,
    pub description: String,
    pub context: String, // Contextual explanation (e.g. "Navigation in logs")
    
    // Initial State
    pub initial_lines: Vec<String>,
    pub initial_cursor: (usize, usize),
    
    // Goal State (Optional constraints)
    pub expected_lines: Option<Vec<String>>,
    pub expected_cursor: Option<(usize, usize)>,
    
    // Hints
    pub hint: String,
    pub solution_keys: String, // One possible optimal solution string
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub keybinding: String,
    pub category: CommandCategory,
    pub description: String,
    pub level: Level, // Updated to use new Level enum
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    Movement,
    Edit,
    Visual,
    Search,
    File,
    Window,
    Other,
}

// Legacy structs kept for compatibility if needed, though Exercise supersedes them for drills
#[derive(Debug, Clone)]
pub struct Motion {
    pub direction: MotionDirection,
    pub count: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum MotionDirection {
    Left,
    Right,
    Up,
    Down,
    WordForward,
    WordBackward,
    LineEnd,
    LineStart,
    FileStart,
    FileEnd,
    Matching,
}

#[derive(Debug, Clone)]
pub struct TextObject {
    pub kind: TextObjectKind,
    pub inner: bool,
}

#[derive(Debug, Clone)]
pub enum TextObjectKind {
    Word,
    Sentence,
    Paragraph,
    Bracket,
    Quote,
    Tag,
}
