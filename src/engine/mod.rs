pub mod vim_buffer;
pub mod mode;

use crate::commands::Command;

pub struct VimEngine {
    pub mode: mode::VimMode,
    pub buffer: vim_buffer::VimBuffer,
    pub current_command: String,
}

impl VimEngine {
    pub fn new() -> Self {
        Self {
            mode: mode::VimMode::Normal,
            buffer: vim_buffer::VimBuffer::new(),
            current_command: String::new(),
        }
    }

    pub fn process_key(&mut self, _key: crossterm::event::KeyEvent) -> Option<Command> {
        // Placeholder for key processing
        None
    }
}

impl Default for VimEngine {
    fn default() -> Self {
        Self::new()
    }
}
