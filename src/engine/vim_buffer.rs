#[derive(Clone, Debug)]
pub struct VimBuffer {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl VimBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec!["Welcome to Vimurai!".to_string()],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_row >= self.lines.len() {
            self.lines.push(String::new());
        }
        let line = &mut self.lines[self.cursor_row];
        line.insert(self.cursor_col, c);
        self.cursor_col += 1;
    }

    pub fn move_cursor(&mut self, direction: MoveDirection) {
        match direction {
            MoveDirection::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            MoveDirection::Right => {
                let line_len = self.lines[self.cursor_row].len();
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                }
            }
            MoveDirection::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
                }
            }
            MoveDirection::Down => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
                }
            }
        }
    }
}

pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

impl Default for VimBuffer {
    fn default() -> Self {
        Self::new()
    }
}
