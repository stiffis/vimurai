#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl Default for VimMode {
    fn default() -> Self {
        VimMode::Normal
    }
}
