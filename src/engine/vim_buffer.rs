#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

#[derive(Clone, Debug)]
pub struct VimBuffer {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    
    // History
    history: Vec<Snapshot>,
    history_index: usize,

    // Visual Mode Anchor
    pub selection_start: Option<(usize, usize)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MoveDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

impl VimBuffer {
    pub fn new() -> Self {
        let initial_lines = vec!["Welcome to Vimurai!".to_string()];
        let initial_snapshot = Snapshot {
            lines: initial_lines.clone(),
            cursor_row: 0,
            cursor_col: 0,
        };

        Self {
            lines: initial_lines,
            cursor_row: 0,
            cursor_col: 0,
            history: vec![initial_snapshot],
            history_index: 0,
            selection_start: None,
        }
    }

    // --- History Management ---

    pub fn save_history(&mut self) {
        // If we are not at the end of history (because of undo), truncate the future
        if self.history_index < self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }

        let snapshot = Snapshot {
            lines: self.lines.clone(),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
        };

        // Don't save if identical to last state
        if let Some(last) = self.history.last() {
            if last == &snapshot {
                return;
            }
        }

        self.history.push(snapshot);
        self.history_index = self.history.len() - 1;
    }

    pub fn undo(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.restore_snapshot();
        }
    }

    pub fn redo(&mut self) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            self.restore_snapshot();
        }
    }

    fn restore_snapshot(&mut self) {
        let snapshot = &self.history[self.history_index];
        self.lines = snapshot.lines.clone();
        self.cursor_row = snapshot.cursor_row;
        self.cursor_col = snapshot.cursor_col;
    }

    // --- Helpers ---

    pub fn current_line(&self) -> &str {
        if self.cursor_row < self.lines.len() {
            &self.lines[self.cursor_row]
        } else {
            ""
        }
    }

    pub fn line_len(&self, row: usize) -> usize {
        self.lines.get(row).map(|l| l.len()).unwrap_or(0)
    }

    pub fn get_char_at(&self, row: usize, col: usize) -> Option<char> {
        self.lines.get(row)?.chars().nth(col)
    }

    pub fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn clamp_cursor(&mut self) {
        if self.cursor_row >= self.lines.len() {
            self.cursor_row = self.lines.len().saturating_sub(1);
        }
        let len = self.line_len(self.cursor_row);
        if self.cursor_col >= len && len > 0 {
            self.cursor_col = len - 1;
        } else if len == 0 {
            self.cursor_col = 0;
        }
    }

    // --- Text Objects & Ranges ---

    /// Returns (start, end) points of the word under cursor (for 'iw')
    pub fn get_word_bounds_at_cursor(&self) -> Option<(Point, Point)> {
        let row = self.cursor_row;
        let col = self.cursor_col;
        
        let c = self.get_char_at(row, col)?;
        if c.is_whitespace() { return None; } // Simple implementation: ignore whitespace selection for now
        
        let is_word = Self::is_word_char(c);

        // Find start
        let mut start_col = col;
        while start_col > 0 {
            if let Some(prev) = self.get_char_at(row, start_col - 1) {
                if Self::is_word_char(prev) != is_word || prev.is_whitespace() {
                    break;
                }
            }
            start_col -= 1;
        }

        // Find end
        let mut end_col = col;
        let len = self.line_len(row);
        while end_col + 1 < len {
            if let Some(next) = self.get_char_at(row, end_col + 1) {
                if Self::is_word_char(next) != is_word || next.is_whitespace() {
                    break;
                }
            }
            end_col += 1;
        }

        Some((Point { row, col: start_col }, Point { row, col: end_col }))
    }

    pub fn get_range_text(&self, start: Point, end: Point) -> String {
        let (s, e) = if start.row < end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        };

        if s.row == e.row {
             let line = &self.lines[s.row];
             let end_idx = (e.col + 1).min(line.len());
             if s.col < line.len() {
                 return line[s.col..end_idx].to_string();
             }
             return String::new();
        }

        // Multi-line support
        let mut result = String::new();
        // First line
        if s.col < self.lines[s.row].len() {
            result.push_str(&self.lines[s.row][s.col..]);
        }
        result.push('\n');
        
        // Middle lines
        for r in (s.row + 1)..e.row {
            result.push_str(&self.lines[r]);
            result.push('\n');
        }

        // Last line
        let last_line = &self.lines[e.row];
        let end_idx = (e.col + 1).min(last_line.len());
        result.push_str(&last_line[..end_idx]);

        result
    }

    pub fn delete_range(&mut self, start: Point, end: Point) {
        self.save_history();
        
        let (s, e) = if start.row < end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        };

        if s.row == e.row {
            // Single line delete
            let line = &mut self.lines[s.row];
            if s.col < line.len() {
                let end_idx = (e.col + 1).min(line.len());
                line.replace_range(s.col..end_idx, "");
            }
            self.cursor_row = s.row;
            self.cursor_col = s.col;
            self.clamp_cursor();
            return;
        }

        // Multi-line delete
        // 1. Keep start of first line
        let prefix = if s.col < self.lines[s.row].len() {
            self.lines[s.row][..s.col].to_string()
        } else {
            self.lines[s.row].clone()
        };

        // 2. Keep end of last line
        let suffix = if e.col + 1 < self.lines[e.row].len() {
            self.lines[e.row][(e.col + 1)..].to_string()
        } else {
            String::new()
        };

        // 3. Remove intermediate lines
        let rows_to_remove = e.row - s.row;
        for _ in 0..rows_to_remove {
            self.lines.remove(s.row + 1);
        }

        // 4. Merge
        self.lines[s.row] = format!("{}{}", prefix, suffix);
        
        self.cursor_row = s.row;
        self.cursor_col = s.col;
        self.clamp_cursor();
    }

    // --- Basic Editing ---

    pub fn insert_char(&mut self, c: char) {
        self.save_history();
        if self.cursor_row >= self.lines.len() {
            self.lines.push(String::new());
        }
        let line = &mut self.lines[self.cursor_row];
        if self.cursor_col > line.len() {
             self.cursor_col = line.len();
        }
        line.insert(self.cursor_col, c);
    }

    // --- Navigation ---

    pub fn move_cursor(&mut self, direction: MoveDirection) {
        match direction {
            MoveDirection::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            MoveDirection::Right => {
                let len = self.current_line().len();
                if self.cursor_col + 1 < len {
                    self.cursor_col += 1;
                }
            }
            MoveDirection::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.clamp_cursor();
                }
            }
            MoveDirection::Down => {
                if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.clamp_cursor();
                }
            }
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_to_line_end(&mut self) {
        let len = self.current_line().len();
        self.cursor_col = len.saturating_sub(1);
    }

    pub fn move_to_non_blank_start(&mut self) {
        let line = self.current_line();
        if let Some((idx, _)) = line.chars().enumerate().find(|(_, c)| !c.is_whitespace()) {
            self.cursor_col = idx;
        } else {
            self.cursor_col = line.len().saturating_sub(1);
        }
    }

    pub fn move_word_forward(&mut self) {
        let mut row = self.cursor_row;
        let mut col = self.cursor_col;
        
        // Logic to move word forward (simplified for brevity, similar to previous)
        // 1. Consume current token
        if let Some(start_char) = self.get_char_at(row, col) {
            let start_is_word = Self::is_word_char(start_char);
            if !start_char.is_whitespace() {
                 while let Some(c) = self.get_char_at(row, col) {
                    if Self::is_word_char(c) != start_is_word || c.is_whitespace() { break; }
                    col += 1;
                    if col >= self.line_len(row) { break; }
                }
            }
        }
        
        // 2. Consume whitespace
        while let Some(c) = self.get_char_at(row, col) {
            if !c.is_whitespace() { break; }
            col += 1;
            if col >= self.line_len(row) {
                if row + 1 < self.lines.len() {
                    row += 1;
                    col = 0;
                     // consume leading whitespace on new line? usually yes.
                     while let Some(nc) = self.get_char_at(row, col) {
                         if !nc.is_whitespace() { break; }
                         col += 1;
                         if col >= self.line_len(row) { break; }
                     }
                    break;
                } else { break; }
            }
        }
        
        self.cursor_row = row;
        self.cursor_col = col;
        self.clamp_cursor();
    }

    pub fn move_word_backward(&mut self) {
        let mut row = self.cursor_row;
        let mut col = self.cursor_col;

        let step_back = |r: &mut usize, c: &mut usize| -> bool {
            if *c > 0 { *c -= 1; true } 
            else if *r > 0 { *r -= 1; *c = self.line_len(*r).saturating_sub(1); true } 
            else { false }
        };

        if !step_back(&mut row, &mut col) { return; }
        while let Some(c) = self.get_char_at(row, col) {
            if !c.is_whitespace() { break; }
            if !step_back(&mut row, &mut col) { return; }
        }

        if let Some(target_char) = self.get_char_at(row, col) {
            let target_is_word = Self::is_word_char(target_char);
            loop {
                if col == 0 { break; }
                if let Some(c) = self.get_char_at(row, col - 1) {
                     if c.is_whitespace() || Self::is_word_char(c) != target_is_word { break; }
                }
                col -= 1;
            }
        }
        self.cursor_row = row;
        self.cursor_col = col;
    }

    pub fn move_word_end(&mut self) {
         // Logic same as previous turn
         let mut row = self.cursor_row;
         let mut col = self.cursor_col;
         let len = self.line_len(row);
         if col + 1 >= len || self.get_char_at(row, col + 1).map_or(false, |c| c.is_whitespace()) { col += 1; }
         while col < self.line_len(row) {
             if let Some(c) = self.get_char_at(row, col) { if !c.is_whitespace() { break; } }
             col += 1;
         }
         if col >= self.line_len(row) && row + 1 < self.lines.len() {
            row += 1; col = 0;
            while col < self.line_len(row) { if let Some(c) = self.get_char_at(row, col) { if !c.is_whitespace() { break; } } col += 1; }
         }
         if let Some(start_char) = self.get_char_at(row, col) {
            let start_is_word = Self::is_word_char(start_char);
            while col + 1 < self.line_len(row) {
                if let Some(next_c) = self.get_char_at(row, col + 1) {
                    if next_c.is_whitespace() || Self::is_word_char(next_c) != start_is_word { break; }
                }
                col += 1;
            }
        }
        self.cursor_row = row;
        self.cursor_col = col;
    }
    
    pub fn find_char_in_line(&mut self, target: char, forward: bool, inclusive: bool) -> bool {
        let start_col = self.cursor_col;
        let line = self.current_line();
        if forward {
            if let Some(idx) = line.chars().enumerate().skip(start_col + 1).find(|(_, c)| *c == target).map(|(i, _)| i) {
                self.cursor_col = if inclusive { idx } else { idx.saturating_sub(1) };
                return true;
            }
        } else {
            if start_col > 0 {
                let chars: Vec<(usize, char)> = line.chars().enumerate().collect();
                for i in (0..start_col).rev() {
                    if chars[i].1 == target {
                        self.cursor_col = if inclusive { i } else { i + 1 };
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn simulate_motion<F>(&self, motion: F) -> Point
    where
        F: Fn(&mut VimBuffer),
    {
        let mut temp = self.clone();
        motion(&mut temp);
        Point {
            row: temp.cursor_row,
            col: temp.cursor_col,
        }
    }
}

impl Default for VimBuffer {
    fn default() -> Self {
        Self::new()
    }
}
