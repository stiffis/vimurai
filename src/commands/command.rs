#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub keybinding: String,
    pub category: CommandCategory,
    pub description: String,
    pub level: Level,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

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
