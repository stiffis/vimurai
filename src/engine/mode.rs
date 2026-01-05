#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Yank,
    Change,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
    OperatorPending(Operator),
}

impl Default for VimMode {
    fn default() -> Self {
        VimMode::Normal
    }
}
