//! A small, deterministic Vim-like editing engine.
//!
//! The module deliberately has no terminal or UI dependencies.  Text is kept as
//! `Vec<char>` so every public column is a Unicode scalar index, never a UTF-8
//! byte offset.

use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    VisualChar,
    VisualLine,
    Command,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorKey {
    Char(char),
    Esc,
    Enter,
    Backspace,
    Delete,
    Tab,
    Ctrl(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewportCommand {
    HalfPageDown,
    HalfPageUp,
    PageDown,
    PageUp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorEvent {
    Executed {
        notation: String,
        changed: bool,
        moved: bool,
    },
    Pending {
        notation: String,
    },
    Invalid {
        notation: String,
    },
    ModeChanged {
        from: Mode,
        to: Mode,
    },
    QuitRequested,
    Viewport {
        command: ViewportCommand,
        lines: usize,
        moved: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Snapshot {
    lines: Vec<Vec<char>>,
    cursor: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Register {
    text: String,
    linewise: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Delete,
    Yank,
    Change,
}

impl Operator {
    fn key(self) -> char {
        match self {
            Self::Delete => 'd',
            Self::Yank => 'y',
            Self::Change => 'c',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FindKind {
    Forward,
    Backward,
    TillForward,
    TillBackward,
}

impl FindKind {
    fn reversed(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
            Self::TillForward => Self::TillBackward,
            Self::TillBackward => Self::TillForward,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LastFind {
    kind: FindKind,
    target: char,
}

#[derive(Debug, Clone, Copy)]
struct LastSearchDirection(bool); // true = forward

#[derive(Debug, Clone)]
enum Pending {
    None,
    G {
        count: usize,
    },
    Find {
        kind: FindKind,
        count: usize,
        operator: Option<Operator>,
    },
    Replace {
        count: usize,
    },
    Operator {
        op: Operator,
        op_count: usize,
        motion_count: String,
    },
    OperatorG {
        op: Operator,
        count: usize,
    },
    TextObject {
        op: Operator,
        count: usize,
        around: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotionKind {
    Exclusive,
    Inclusive,
    Linewise,
}

#[derive(Debug, Clone, Copy)]
struct Motion {
    target: Position,
    kind: MotionKind,
}

#[derive(Debug, Clone)]
struct InsertSession {
    before: Snapshot,
    return_cursor: Position,
}

#[derive(Debug, Clone)]
pub struct Editor {
    lines: Vec<Vec<char>>,
    cursor: Position,
    desired_col: usize,
    mode: Mode,
    visual_anchor: Option<Position>,
    pending: Pending,
    count: String,
    pending_display: String,
    register: Register,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    insert_session: Option<InsertSession>,
    command_line: String,
    search_forward: bool,
    last_search: Option<(String, LastSearchDirection)>,
    last_find: Option<LastFind>,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new(vec![String::new()], Position::default())
    }
}

impl Editor {
    pub fn new(lines: Vec<String>, cursor: Position) -> Self {
        let mut editor = Self {
            lines: Self::to_char_lines(lines),
            cursor,
            desired_col: cursor.col,
            mode: Mode::Normal,
            visual_anchor: None,
            pending: Pending::None,
            count: String::new(),
            pending_display: String::new(),
            register: Register::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            insert_session: None,
            command_line: String::new(),
            search_forward: true,
            last_search: None,
            last_find: None,
        };
        editor.clamp_normal_cursor();
        editor.desired_col = editor.cursor.col;
        editor
    }

    pub fn reset(&mut self, lines: Vec<String>, cursor: Position) {
        *self = Self::new(lines, cursor);
    }

    pub fn lines(&self) -> &[Vec<char>] {
        &self.lines
    }

    pub fn lines_as_strings(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|line| line.iter().collect())
            .collect()
    }

    pub const fn cursor(&self) -> Position {
        self.cursor
    }

    pub const fn mode(&self) -> Mode {
        self.mode
    }

    pub fn selection(&self) -> Option<(Position, Position)> {
        self.visual_anchor.map(|anchor| (anchor, self.cursor))
    }

    pub fn pending_display(&self) -> &str {
        &self.pending_display
    }

    pub fn register(&self) -> &str {
        &self.register.text
    }

    pub const fn register_is_linewise(&self) -> bool {
        self.register.linewise
    }

    pub fn handle_key(&mut self, key: EditorKey, viewport_height: usize) -> EditorEvent {
        match self.mode {
            Mode::Insert => self.handle_insert(key),
            Mode::VisualChar | Mode::VisualLine => self.handle_visual(key),
            Mode::Command => self.handle_command_line(key),
            Mode::Search => self.handle_search_line(key),
            Mode::Normal => self.handle_normal(key, viewport_height),
        }
    }

    fn to_char_lines(lines: Vec<String>) -> Vec<Vec<char>> {
        let mut result: Vec<Vec<char>> = lines
            .into_iter()
            .map(|line| line.chars().collect())
            .collect();
        if result.is_empty() {
            result.push(Vec::new());
        }
        result
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            lines: self.lines.clone(),
            cursor: self.cursor,
        }
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.lines = snapshot.lines;
        self.cursor = snapshot.cursor;
        self.mode = Mode::Normal;
        self.visual_anchor = None;
        self.clear_parser();
        self.clamp_normal_cursor();
        self.desired_col = self.cursor.col;
    }

    fn record_change(&mut self, before: Snapshot) -> bool {
        if before.lines == self.lines {
            return false;
        }
        self.undo.push(before);
        self.redo.clear();
        true
    }

    fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        let current = self.snapshot();
        self.redo.push(current);
        self.restore(previous);
        true
    }

    fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        let current = self.snapshot();
        self.undo.push(current);
        self.restore(next);
        true
    }

    fn line_max_col(&self, row: usize) -> usize {
        self.lines[row].len().saturating_sub(1)
    }

    fn clamp_normal_cursor(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);
        self.cursor.col = self.cursor.col.min(self.line_max_col(self.cursor.row));
    }

    fn clamp_insert_cursor(&mut self) {
        self.cursor.row = self.cursor.row.min(self.lines.len() - 1);
        self.cursor.col = self.cursor.col.min(self.lines[self.cursor.row].len());
    }

    fn first_non_blank(&self, row: usize) -> usize {
        self.lines[row]
            .iter()
            .position(|c| !c.is_whitespace())
            .unwrap_or(0)
    }

    fn last_non_blank(&self, row: usize) -> usize {
        self.lines[row]
            .iter()
            .rposition(|c| !c.is_whitespace())
            .unwrap_or(0)
    }

    fn clear_parser(&mut self) {
        self.pending = Pending::None;
        self.count.clear();
        self.pending_display.clear();
    }

    fn count_value(&self) -> usize {
        self.count.parse().unwrap_or(1).max(1)
    }

    fn take_count(&mut self) -> usize {
        let count = self.count_value();
        self.count.clear();
        count
    }

    fn executed(notation: impl Into<String>, changed: bool, moved: bool) -> EditorEvent {
        EditorEvent::Executed {
            notation: notation.into(),
            changed,
            moved,
        }
    }

    fn invalid(&mut self, notation: impl Into<String>) -> EditorEvent {
        let notation = notation.into();
        self.clear_parser();
        EditorEvent::Invalid { notation }
    }

    fn set_pending(&mut self, pending: Pending, notation: String) -> EditorEvent {
        self.pending = pending;
        self.pending_display = notation.clone();
        EditorEvent::Pending { notation }
    }

    fn handle_normal(&mut self, key: EditorKey, viewport_height: usize) -> EditorEvent {
        if !matches!(self.pending, Pending::None) {
            return self.handle_pending(key);
        }

        if let EditorKey::Ctrl(c) = key {
            return self.handle_control(c, viewport_height);
        }

        let EditorKey::Char(c) = key else {
            return match key {
                EditorKey::Esc => {
                    self.clear_parser();
                    Self::executed("<Esc>", false, false)
                }
                _ => self.invalid(format!("{key:?}")),
            };
        };

        if c.is_ascii_digit() && (c != '0' || !self.count.is_empty()) {
            self.count.push(c);
            self.pending_display = self.count.clone();
            return EditorEvent::Pending {
                notation: self.pending_display.clone(),
            };
        }

        let explicit_count = !self.count.is_empty();
        let count = self.take_count();
        match c {
            'h' | 'j' | 'k' | 'l' | 'w' | 'W' | 'b' | 'B' | 'e' | 'E' | '0' | '^' | '$' | 'g'
            | 'G' | '{' | '}' | '%' => self.handle_motion_key(c, count, explicit_count),
            'f' | 'F' | 't' | 'T' => {
                let kind = Self::find_kind(c).expect("matched find key");
                self.set_pending(
                    Pending::Find {
                        kind,
                        count,
                        operator: None,
                    },
                    format!(
                        "{count_prefix}{c}",
                        count_prefix = Self::display_count(count)
                    ),
                )
            }
            ';' | ',' => self.repeat_find(c == ',', count, None),
            'i' => self.enter_insert(self.cursor, self.cursor, "i"),
            'a' => {
                let normal = self.cursor;
                let caret = Position::new(
                    self.cursor.row,
                    (self.cursor.col + 1).min(self.lines[self.cursor.row].len()),
                );
                self.enter_insert(caret, normal, "a")
            }
            'I' => {
                let col = self.first_non_blank(self.cursor.row);
                let pos = Position::new(self.cursor.row, col);
                self.enter_insert(pos, pos, "I")
            }
            'A' => {
                let caret = Position::new(self.cursor.row, self.lines[self.cursor.row].len());
                let normal = Position::new(self.cursor.row, self.line_max_col(self.cursor.row));
                self.enter_insert(caret, normal, "A")
            }
            'o' | 'O' => self.open_line(c),
            'x' => self.delete_chars(count, false),
            'X' => self.delete_chars(count, true),
            'r' => self.set_pending(
                Pending::Replace { count },
                format!("{}r", Self::display_count(count)),
            ),
            's' => self.substitute_chars(count),
            'D' => self.direct_operator(Operator::Delete, '$', count, "D"),
            'C' => self.direct_operator(Operator::Change, '$', count, "C"),
            'J' => self.join_lines(count),
            'd' | 'y' | 'c' => {
                let op = Self::operator(c).expect("matched operator");
                self.set_pending(
                    Pending::Operator {
                        op,
                        op_count: count,
                        motion_count: String::new(),
                    },
                    format!("{}{c}", Self::display_count(count)),
                )
            }
            'p' | 'P' => self.paste(c == 'p', count),
            'v' => self.enter_visual(Mode::VisualChar),
            'V' => self.enter_visual(Mode::VisualLine),
            '/' | '?' => self.enter_search(c == '/'),
            '*' | '#' => self.search_word(c == '*'),
            'n' | 'N' => self.repeat_search(c == 'N', count),
            ':' => {
                let from = self.mode;
                self.mode = Mode::Command;
                self.command_line.clear();
                self.pending_display = ":".into();
                EditorEvent::ModeChanged {
                    from,
                    to: self.mode,
                }
            }
            'u' => {
                let changed = self.undo();
                Self::executed("u", changed, changed)
            }
            _ => self.invalid(c.to_string()),
        }
    }

    fn handle_control(&mut self, c: char, viewport_height: usize) -> EditorEvent {
        if c.eq_ignore_ascii_case(&'r') {
            let changed = self.redo();
            return Self::executed("<C-r>", changed, changed);
        }
        let (command, down, amount) = match c.to_ascii_lowercase() {
            'd' => (
                ViewportCommand::HalfPageDown,
                true,
                max(1, viewport_height / 2),
            ),
            'u' => (
                ViewportCommand::HalfPageUp,
                false,
                max(1, viewport_height / 2),
            ),
            'f' => (ViewportCommand::PageDown, true, max(1, viewport_height)),
            'b' => (ViewportCommand::PageUp, false, max(1, viewport_height)),
            _ => return self.invalid(format!("<C-{c}>")),
        };
        let before = self.cursor;
        self.move_vertical(if down {
            amount as isize
        } else {
            -(amount as isize)
        });
        EditorEvent::Viewport {
            command,
            lines: amount,
            moved: before != self.cursor,
        }
    }

    fn handle_motion_key(&mut self, c: char, count: usize, explicit_count: bool) -> EditorEvent {
        if c == 'g' {
            return self.set_pending(
                Pending::G { count },
                format!("{}g", Self::display_count(count)),
            );
        }
        let before = self.cursor;
        let motion = if c == 'G' && explicit_count {
            let row = count.saturating_sub(1).min(self.lines.len() - 1);
            Some(Motion {
                target: Position::new(row, self.first_non_blank(row)),
                kind: MotionKind::Linewise,
            })
        } else {
            self.motion_for(c, count)
        };
        let Some(motion) = motion else {
            return self.invalid(format!("{}{c}", Self::display_count(count)));
        };
        self.cursor = motion.target;
        if !matches!(c, 'j' | 'k') {
            self.desired_col = self.cursor.col;
        }
        self.clear_parser();
        Self::executed(
            format!("{}{c}", Self::display_count(count)),
            false,
            before != self.cursor,
        )
    }

    fn handle_pending(&mut self, key: EditorKey) -> EditorEvent {
        let pending = self.pending.clone();
        match pending {
            Pending::None => unreachable!(),
            Pending::G { count } => match key {
                EditorKey::Esc => self.cancel_pending(),
                EditorKey::Char('g') => {
                    let before = self.cursor;
                    let row = count.saturating_sub(1).min(self.lines.len() - 1);
                    self.cursor = Position::new(row, self.first_non_blank(row));
                    self.desired_col = self.cursor.col;
                    let notation = format!("{}gg", Self::display_count(count));
                    self.clear_parser();
                    Self::executed(notation, false, before != self.cursor)
                }
                EditorKey::Char('_') => {
                    let before = self.cursor;
                    let row = (self.cursor.row + count - 1).min(self.lines.len() - 1);
                    self.cursor = Position::new(row, self.last_non_blank(row));
                    self.desired_col = self.cursor.col;
                    let notation = format!("{}g_", Self::display_count(count));
                    self.clear_parser();
                    Self::executed(notation, false, before != self.cursor)
                }
                _ => self.invalid(self.pending_display.clone()),
            },
            Pending::Find {
                kind,
                count,
                operator,
            } => match key {
                EditorKey::Esc => self.cancel_pending(),
                EditorKey::Char(target) => self.finish_find(kind, target, count, operator),
                _ => self.invalid(self.pending_display.clone()),
            },
            Pending::Replace { count } => match key {
                EditorKey::Esc => self.cancel_pending(),
                EditorKey::Char(replacement) => self.replace_chars(replacement, count),
                _ => self.invalid(self.pending_display.clone()),
            },
            Pending::Operator {
                op,
                op_count,
                mut motion_count,
            } => match key {
                EditorKey::Esc => self.cancel_pending(),
                EditorKey::Char(c)
                    if c.is_ascii_digit() && (c != '0' || !motion_count.is_empty()) =>
                {
                    motion_count.push(c);
                    let notation = format!(
                        "{}{op_key}{motion_count}",
                        Self::display_count(op_count),
                        op_key = op.key()
                    );
                    self.set_pending(
                        Pending::Operator {
                            op,
                            op_count,
                            motion_count,
                        },
                        notation,
                    )
                }
                EditorKey::Char(c) if c == op.key() => {
                    let motion_count = motion_count.parse::<usize>().unwrap_or(1).max(1);
                    self.apply_line_operator(op, op_count.saturating_mul(motion_count))
                }
                EditorKey::Char(c @ ('f' | 'F' | 't' | 'T')) => {
                    let motion_count = motion_count.parse::<usize>().unwrap_or(1).max(1);
                    let count = op_count.saturating_mul(motion_count);
                    let kind = Self::find_kind(c).unwrap();
                    let notation = format!(
                        "{}{op_key}{}{c}",
                        Self::display_count(op_count),
                        Self::display_count(motion_count),
                        op_key = op.key()
                    );
                    self.set_pending(
                        Pending::Find {
                            kind,
                            count,
                            operator: Some(op),
                        },
                        notation,
                    )
                }
                EditorKey::Char('g') => {
                    let motion_count = motion_count.parse::<usize>().unwrap_or(1).max(1);
                    let count = op_count.saturating_mul(motion_count);
                    let notation = format!(
                        "{}{op_key}{}g",
                        Self::display_count(op_count),
                        Self::display_count(motion_count),
                        op_key = op.key()
                    );
                    self.set_pending(Pending::OperatorG { op, count }, notation)
                }
                EditorKey::Char(object @ ('i' | 'a')) => {
                    let motion_count = motion_count.parse::<usize>().unwrap_or(1).max(1);
                    let count = op_count.saturating_mul(motion_count);
                    let notation = format!(
                        "{}{op_key}{}{object}",
                        Self::display_count(op_count),
                        Self::display_count(motion_count),
                        op_key = op.key()
                    );
                    self.set_pending(
                        Pending::TextObject {
                            op,
                            count,
                            around: object == 'a',
                        },
                        notation,
                    )
                }
                EditorKey::Char(
                    c @ ('h' | 'j' | 'k' | 'l' | 'w' | 'W' | 'b' | 'B' | 'e' | 'E' | '0' | '^'
                    | '$' | 'G' | '{' | '}' | '%'),
                ) => {
                    let motion_count = motion_count.parse::<usize>().unwrap_or(1).max(1);
                    let count = op_count.saturating_mul(motion_count);
                    self.apply_operator_motion(
                        op,
                        c,
                        count,
                        format!(
                            "{}{op_key}{}{c}",
                            Self::display_count(op_count),
                            Self::display_count(motion_count),
                            op_key = op.key()
                        ),
                    )
                }
                EditorKey::Char(c @ (';' | ',')) => {
                    let motion_count = motion_count.parse::<usize>().unwrap_or(1).max(1);
                    self.repeat_find(c == ',', op_count.saturating_mul(motion_count), Some(op))
                }
                _ => self.invalid(self.pending_display.clone()),
            },
            Pending::OperatorG { op, count } => match key {
                EditorKey::Esc => self.cancel_pending(),
                EditorKey::Char('g') => {
                    let row = count.saturating_sub(1).min(self.lines.len() - 1);
                    self.apply_operator(
                        op,
                        Motion {
                            target: Position::new(row, 0),
                            kind: MotionKind::Linewise,
                        },
                        format!("{}{}gg", Self::display_count(count), op.key()),
                    )
                }
                _ => self.invalid(self.pending_display.clone()),
            },
            Pending::TextObject { op, count, around } => match key {
                EditorKey::Esc => self.cancel_pending(),
                EditorKey::Char(object @ ('w' | '"' | '(' | ')' | '[' | ']')) => {
                    self.finish_text_object(op, object, around, count)
                }
                _ => self.invalid(self.pending_display.clone()),
            },
        }
    }

    fn cancel_pending(&mut self) -> EditorEvent {
        self.clear_parser();
        Self::executed("<Esc>", false, false)
    }

    fn display_count(count: usize) -> String {
        if count == 1 {
            String::new()
        } else {
            count.to_string()
        }
    }

    fn operator(c: char) -> Option<Operator> {
        match c {
            'd' => Some(Operator::Delete),
            'y' => Some(Operator::Yank),
            'c' => Some(Operator::Change),
            _ => None,
        }
    }

    fn find_kind(c: char) -> Option<FindKind> {
        match c {
            'f' => Some(FindKind::Forward),
            'F' => Some(FindKind::Backward),
            't' => Some(FindKind::TillForward),
            'T' => Some(FindKind::TillBackward),
            _ => None,
        }
    }

    fn motion_for(&self, c: char, count: usize) -> Option<Motion> {
        let count = count.max(1);
        let target = match c {
            'h' => Position::new(self.cursor.row, self.cursor.col.saturating_sub(count)),
            'l' => Position::new(
                self.cursor.row,
                min(
                    self.cursor.col.saturating_add(count),
                    self.line_max_col(self.cursor.row),
                ),
            ),
            'j' => {
                let row = min(self.cursor.row.saturating_add(count), self.lines.len() - 1);
                Position::new(row, min(self.desired_col, self.line_max_col(row)))
            }
            'k' => {
                let row = self.cursor.row.saturating_sub(count);
                Position::new(row, min(self.desired_col, self.line_max_col(row)))
            }
            '0' => Position::new(self.cursor.row, 0),
            '^' => Position::new(self.cursor.row, self.first_non_blank(self.cursor.row)),
            '$' => {
                let row = min(
                    self.cursor.row.saturating_add(count - 1),
                    self.lines.len() - 1,
                );
                Position::new(row, self.line_max_col(row))
            }
            'G' => {
                let row = if count == 1 {
                    self.lines.len() - 1
                } else {
                    min(count - 1, self.lines.len() - 1)
                };
                Position::new(row, self.first_non_blank(row))
            }
            'w' => self.word_forward(count, false),
            'W' => self.word_forward(count, true),
            'b' => self.word_backward(count, false),
            'B' => self.word_backward(count, true),
            'e' => self.word_end(count, false),
            'E' => self.word_end(count, true),
            '{' => self.paragraph_backward(count),
            '}' => self.paragraph_forward(count),
            '%' => self.matching_bracket()?,
            _ => return None,
        };
        let kind = match c {
            'j' | 'k' | 'G' => MotionKind::Linewise,
            'e' | 'E' | '$' | '%' => MotionKind::Inclusive,
            _ => MotionKind::Exclusive,
        };
        Some(Motion { target, kind })
    }

    fn move_vertical(&mut self, delta: isize) {
        let row = if delta < 0 {
            self.cursor.row.saturating_sub(delta.unsigned_abs())
        } else {
            min(
                self.cursor.row.saturating_add(delta as usize),
                self.lines.len() - 1,
            )
        };
        self.cursor = Position::new(row, min(self.desired_col, self.line_max_col(row)));
    }

    fn flat_chars(&self) -> Vec<char> {
        let mut flat = Vec::new();
        for (index, line) in self.lines.iter().enumerate() {
            flat.extend(line.iter().copied());
            if index + 1 < self.lines.len() {
                flat.push('\n');
            }
        }
        flat
    }

    fn position_to_offset(&self, pos: Position) -> usize {
        let prefix: usize = self
            .lines
            .iter()
            .take(pos.row)
            .map(|line| line.len() + 1)
            .sum();
        prefix + min(pos.col, self.lines[pos.row].len())
    }

    fn offset_to_position(&self, offset: usize) -> Position {
        let mut left = offset;
        for (row, line) in self.lines.iter().enumerate() {
            if left < line.len() {
                return Position::new(row, left);
            }
            if row + 1 == self.lines.len() {
                return Position::new(row, line.len().saturating_sub(1));
            }
            if left == line.len() {
                return Position::new(row + 1, 0);
            }
            left = left.saturating_sub(line.len() + 1);
        }
        self.last_position()
    }

    fn last_position(&self) -> Position {
        let row = self.lines.len() - 1;
        Position::new(row, self.line_max_col(row))
    }

    fn word_class(c: char, big: bool) -> u8 {
        if c.is_whitespace() {
            0
        } else if big || c.is_alphanumeric() || c == '_' {
            1
        } else {
            2
        }
    }

    fn word_forward(&self, count: usize, big: bool) -> Position {
        let chars = self.flat_chars();
        if chars.is_empty() {
            return self.cursor;
        }
        let mut at = self.position_to_offset(self.cursor).min(chars.len() - 1);
        for _ in 0..count {
            let class = Self::word_class(chars[at], big);
            if class != 0 {
                while at + 1 < chars.len() && Self::word_class(chars[at], big) == class {
                    at += 1;
                }
                if at + 1 < chars.len() {
                    at += 1;
                }
            }
            while at < chars.len() && Self::word_class(chars[at], big) == 0 {
                at += 1;
            }
            if at >= chars.len() {
                at = chars.len() - 1;
            }
        }
        self.offset_to_position(at)
    }

    fn word_backward(&self, count: usize, big: bool) -> Position {
        let chars = self.flat_chars();
        if chars.is_empty() {
            return self.cursor;
        }
        let mut at = self.position_to_offset(self.cursor).min(chars.len() - 1);
        for _ in 0..count {
            if at == 0 {
                break;
            }
            at -= 1;
            while at > 0 && Self::word_class(chars[at], big) == 0 {
                at -= 1;
            }
            let class = Self::word_class(chars[at], big);
            while at > 0 && Self::word_class(chars[at - 1], big) == class {
                at -= 1;
            }
        }
        self.offset_to_position(at)
    }

    fn word_end(&self, count: usize, big: bool) -> Position {
        let chars = self.flat_chars();
        if chars.is_empty() {
            return self.cursor;
        }
        let mut at = self.position_to_offset(self.cursor).min(chars.len() - 1);
        for iteration in 0..count {
            if iteration > 0 && at + 1 < chars.len() {
                at += 1;
            }
            if at + 1 < chars.len() && Self::word_class(chars[at + 1], big) != 0 {
                at += 1;
            }
            while at < chars.len() && Self::word_class(chars[at], big) == 0 {
                at += 1;
            }
            if at >= chars.len() {
                at = chars.len() - 1;
                break;
            }
            let class = Self::word_class(chars[at], big);
            while at + 1 < chars.len() && Self::word_class(chars[at + 1], big) == class {
                at += 1;
            }
        }
        self.offset_to_position(at)
    }

    fn paragraph_forward(&self, count: usize) -> Position {
        let mut row = self.cursor.row;
        for _ in 0..count {
            row = min(row + 1, self.lines.len() - 1);
            while row + 1 < self.lines.len() && !self.lines[row].is_empty() {
                row += 1;
            }
            while row + 1 < self.lines.len() && self.lines[row].is_empty() {
                row += 1;
            }
        }
        Position::new(row, self.first_non_blank(row))
    }

    fn paragraph_backward(&self, count: usize) -> Position {
        let mut row = self.cursor.row;
        for _ in 0..count {
            row = row.saturating_sub(1);
            while row > 0 && !self.lines[row].is_empty() {
                row -= 1;
            }
            while row > 0 && self.lines[row].is_empty() {
                row -= 1;
            }
            if !self.lines[row].is_empty() && row + 1 < self.lines.len() {
                row += 1;
            }
        }
        Position::new(row, self.first_non_blank(row))
    }

    fn matching_bracket(&self) -> Option<Position> {
        let pairs = [('(', ')'), ('[', ']'), ('{', '}')];
        let line = &self.lines[self.cursor.row];
        let mut start = self.cursor.col;
        while start < line.len()
            && !pairs
                .iter()
                .any(|(a, b)| line[start] == *a || line[start] == *b)
        {
            start += 1;
        }
        let ch = *line.get(start)?;
        let (open, close, forward) = pairs.iter().find_map(|(a, b)| {
            if ch == *a {
                Some((*a, *b, true))
            } else if ch == *b {
                Some((*a, *b, false))
            } else {
                None
            }
        })?;
        let flat = self.flat_chars();
        let origin = self.position_to_offset(Position::new(self.cursor.row, start));
        let mut depth = 0usize;
        if forward {
            for (idx, c) in flat.iter().enumerate().skip(origin) {
                if *c == open {
                    depth += 1;
                }
                if *c == close {
                    depth -= 1;
                    if depth == 0 {
                        return Some(self.offset_to_position(idx));
                    }
                }
            }
        } else {
            for idx in (0..=origin).rev() {
                let c = flat[idx];
                if c == close {
                    depth += 1;
                }
                if c == open {
                    depth -= 1;
                    if depth == 0 {
                        return Some(self.offset_to_position(idx));
                    }
                }
            }
        }
        None
    }

    fn find_target(&self, kind: FindKind, target: char, count: usize) -> Option<Position> {
        let line = &self.lines[self.cursor.row];
        let mut found = self.cursor.col;
        match kind {
            FindKind::Forward | FindKind::TillForward => {
                for _ in 0..count {
                    found = ((found + 1)..line.len()).find(|index| line[*index] == target)?;
                }
                if matches!(kind, FindKind::TillForward) {
                    found = found.saturating_sub(1);
                }
            }
            FindKind::Backward | FindKind::TillBackward => {
                for _ in 0..count {
                    found = (0..found).rev().find(|index| line[*index] == target)?;
                }
                if matches!(kind, FindKind::TillBackward) {
                    found = min(found + 1, self.line_max_col(self.cursor.row));
                }
            }
        }
        Some(Position::new(self.cursor.row, found))
    }

    fn finish_find(
        &mut self,
        kind: FindKind,
        target: char,
        count: usize,
        operator: Option<Operator>,
    ) -> EditorEvent {
        let notation = if operator.is_some() {
            format!("{}{}{}", self.pending_display, target, "")
        } else {
            format!("{}{}", self.pending_display, target)
        };
        let Some(position) = self.find_target(kind, target, count) else {
            return self.invalid(notation);
        };
        self.last_find = Some(LastFind { kind, target });
        self.clear_parser();
        if let Some(op) = operator {
            self.apply_operator(
                op,
                Motion {
                    target: position,
                    kind: MotionKind::Inclusive,
                },
                notation.to_string(),
            )
        } else {
            let before = self.cursor;
            self.cursor = position;
            self.desired_col = self.cursor.col;
            Self::executed(notation, false, before != self.cursor)
        }
    }

    fn repeat_find(
        &mut self,
        reverse: bool,
        count: usize,
        operator: Option<Operator>,
    ) -> EditorEvent {
        let Some(last) = self.last_find else {
            return self.invalid(if reverse { "," } else { ";" });
        };
        let kind = if reverse {
            last.kind.reversed()
        } else {
            last.kind
        };
        let notation = if reverse { "," } else { ";" };
        let Some(position) = self.find_target(kind, last.target, count) else {
            return self.invalid(notation);
        };
        self.clear_parser();
        if let Some(op) = operator {
            self.apply_operator(
                op,
                Motion {
                    target: position,
                    kind: MotionKind::Inclusive,
                },
                notation.to_owned(),
            )
        } else {
            let before = self.cursor;
            self.cursor = position;
            self.desired_col = self.cursor.col;
            Self::executed(notation, false, before != self.cursor)
        }
    }

    fn finish_text_object(
        &mut self,
        op: Operator,
        object: char,
        around: bool,
        count: usize,
    ) -> EditorEvent {
        let notation = format!("{}{object}", self.pending_display);
        let range = match object {
            'w' => self.word_object_range(around, count),
            '"' => self.quote_object_range(around),
            '(' | ')' => self.delimited_object_range('(', ')', around),
            '[' | ']' => self.delimited_object_range('[', ']', around),
            _ => None,
        };
        let Some((start, end)) = range else {
            return self.invalid(notation);
        };
        self.clear_parser();
        self.apply_operator_range(op, start, end, notation)
    }

    fn word_object_range(&self, around: bool, count: usize) -> Option<(Position, Position)> {
        let line = &self.lines[self.cursor.row];
        let current = *line.get(self.cursor.col)?;
        let class = Self::word_class(current, false);
        if class == 0 {
            return None;
        }
        let mut start = self.cursor.col;
        let mut end = self.cursor.col + 1;
        while start > 0 && Self::word_class(line[start - 1], false) == class {
            start -= 1;
        }
        while end < line.len() && Self::word_class(line[end], false) == class {
            end += 1;
        }

        for _ in 1..count.max(1) {
            while end < line.len() && line[end].is_whitespace() {
                end += 1;
            }
            if end >= line.len() {
                break;
            }
            let next_class = Self::word_class(line[end], false);
            while end < line.len() && Self::word_class(line[end], false) == next_class {
                end += 1;
            }
        }

        if around {
            let content_end = end;
            while end < line.len() && line[end].is_whitespace() {
                end += 1;
            }
            if end == content_end {
                while start > 0 && line[start - 1].is_whitespace() {
                    start -= 1;
                }
            }
        }
        Some((
            Position::new(self.cursor.row, start),
            Position::new(self.cursor.row, end),
        ))
    }

    fn quote_object_range(&self, around: bool) -> Option<(Position, Position)> {
        let line = &self.lines[self.cursor.row];
        let quotes: Vec<usize> = line
            .iter()
            .enumerate()
            .filter_map(|(index, c)| (*c == '"').then_some(index))
            .collect();
        for pair in quotes.chunks_exact(2) {
            let open = pair[0];
            let close = pair[1];
            if open <= self.cursor.col && self.cursor.col <= close {
                let (start, end) = if around {
                    (open, close + 1)
                } else {
                    (open + 1, close)
                };
                return Some((
                    Position::new(self.cursor.row, start),
                    Position::new(self.cursor.row, end),
                ));
            }
        }
        None
    }

    fn delimited_object_range(
        &self,
        open: char,
        close: char,
        around: bool,
    ) -> Option<(Position, Position)> {
        let line = &self.lines[self.cursor.row];
        for open_index in (0..=self.cursor.col.min(line.len().saturating_sub(1))).rev() {
            if line.get(open_index) != Some(&open) {
                continue;
            }
            let mut depth = 0usize;
            for (close_index, character) in line.iter().copied().enumerate().skip(open_index) {
                match character {
                    c if c == open => depth += 1,
                    c if c == close => {
                        depth -= 1;
                        if depth == 0 {
                            if close_index >= self.cursor.col {
                                let (start, end) = if around {
                                    (open_index, close_index + 1)
                                } else {
                                    (open_index + 1, close_index)
                                };
                                return Some((
                                    Position::new(self.cursor.row, start),
                                    Position::new(self.cursor.row, end),
                                ));
                            }
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn apply_operator_range(
        &mut self,
        op: Operator,
        start: Position,
        end: Position,
        notation: String,
    ) -> EditorEvent {
        let text = self.extract_range(start, end);
        if text.is_empty() {
            return self.invalid(notation);
        }
        self.register = Register {
            text,
            linewise: false,
        };
        if op == Operator::Yank {
            self.cursor = start;
            self.mode = Mode::Normal;
            self.visual_anchor = None;
            self.clear_parser();
            return Self::executed(notation, false, true);
        }

        let before = self.snapshot();
        self.delete_range(start, end);
        if op == Operator::Change {
            self.clamp_insert_cursor();
            let return_cursor = self.cursor;
            self.enter_insert_after_change(before, return_cursor);
            Self::executed(notation, true, true)
        } else {
            self.mode = Mode::Normal;
            self.visual_anchor = None;
            self.clamp_normal_cursor();
            self.desired_col = self.cursor.col;
            let changed = self.record_change(before);
            self.clear_parser();
            Self::executed(notation, changed, true)
        }
    }

    fn enter_insert(
        &mut self,
        caret: Position,
        return_cursor: Position,
        notation: &str,
    ) -> EditorEvent {
        let from = self.mode;
        self.insert_session = Some(InsertSession {
            before: self.snapshot(),
            return_cursor,
        });
        self.cursor = caret;
        self.mode = Mode::Insert;
        self.clear_parser();
        self.pending_display = notation.to_string();
        EditorEvent::ModeChanged {
            from,
            to: self.mode,
        }
    }

    fn enter_insert_after_change(&mut self, before: Snapshot, return_cursor: Position) {
        self.insert_session = Some(InsertSession {
            before,
            return_cursor,
        });
        self.mode = Mode::Insert;
        self.visual_anchor = None;
        self.clear_parser();
    }

    fn open_line(&mut self, command: char) -> EditorEvent {
        let before = self.snapshot();
        let original = self.cursor;
        let row = if command == 'o' {
            self.cursor.row + 1
        } else {
            self.cursor.row
        };
        self.lines.insert(row, Vec::new());
        self.cursor = Position::new(row, 0);
        self.insert_session = Some(InsertSession {
            before,
            return_cursor: original,
        });
        let from = self.mode;
        self.mode = Mode::Insert;
        self.clear_parser();
        EditorEvent::ModeChanged {
            from,
            to: self.mode,
        }
    }

    fn handle_insert(&mut self, key: EditorKey) -> EditorEvent {
        match key {
            EditorKey::Esc | EditorKey::Ctrl('[') | EditorKey::Ctrl('c') => self.finish_insert(),
            EditorKey::Char(c) => {
                self.lines[self.cursor.row].insert(self.cursor.col, c);
                self.cursor.col += 1;
                Self::executed(c.to_string(), true, true)
            }
            EditorKey::Tab => {
                self.lines[self.cursor.row].insert(self.cursor.col, '\t');
                self.cursor.col += 1;
                Self::executed("<Tab>", true, true)
            }
            EditorKey::Enter => {
                let suffix = self.lines[self.cursor.row].split_off(self.cursor.col);
                self.lines.insert(self.cursor.row + 1, suffix);
                self.cursor = Position::new(self.cursor.row + 1, 0);
                Self::executed("<CR>", true, true)
            }
            EditorKey::Backspace => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                    self.lines[self.cursor.row].remove(self.cursor.col);
                    Self::executed("<BS>", true, true)
                } else if self.cursor.row > 0 {
                    let current = self.lines.remove(self.cursor.row);
                    self.cursor.row -= 1;
                    self.cursor.col = self.lines[self.cursor.row].len();
                    self.lines[self.cursor.row].extend(current);
                    Self::executed("<BS>", true, true)
                } else {
                    self.invalid("<BS>")
                }
            }
            EditorKey::Delete => {
                if self.cursor.col < self.lines[self.cursor.row].len() {
                    self.lines[self.cursor.row].remove(self.cursor.col);
                    Self::executed("<Del>", true, false)
                } else if self.cursor.row + 1 < self.lines.len() {
                    let next = self.lines.remove(self.cursor.row + 1);
                    self.lines[self.cursor.row].extend(next);
                    Self::executed("<Del>", true, false)
                } else {
                    self.invalid("<Del>")
                }
            }
            EditorKey::Ctrl(c) => self.invalid(format!("<C-{c}>")),
        }
    }

    fn finish_insert(&mut self) -> EditorEvent {
        let from = self.mode;
        let session = self.insert_session.take().expect("insert mode has session");
        let changed = session.before.lines != self.lines;
        if changed {
            if self.cursor.col > 0 {
                self.cursor.col -= 1;
            }
            self.record_change(session.before);
        } else {
            self.cursor = session.return_cursor;
        }
        self.mode = Mode::Normal;
        self.clamp_normal_cursor();
        self.desired_col = self.cursor.col;
        self.clear_parser();
        EditorEvent::ModeChanged {
            from,
            to: self.mode,
        }
    }

    fn delete_chars(&mut self, count: usize, backwards: bool) -> EditorEvent {
        let before = self.snapshot();
        let line = &mut self.lines[self.cursor.row];
        let (start, end) = if backwards {
            (self.cursor.col.saturating_sub(count), self.cursor.col)
        } else {
            (
                self.cursor.col,
                min(self.cursor.col.saturating_add(count), line.len()),
            )
        };
        if start >= end {
            return self.invalid(if backwards { "X" } else { "x" });
        }
        let text: String = line[start..end].iter().collect();
        line.drain(start..end);
        self.register = Register {
            text,
            linewise: false,
        };
        self.cursor.col = start;
        self.clamp_normal_cursor();
        self.desired_col = self.cursor.col;
        let changed = self.record_change(before);
        self.clear_parser();
        Self::executed(if backwards { "X" } else { "x" }, changed, backwards)
    }

    fn replace_chars(&mut self, replacement: char, count: usize) -> EditorEvent {
        let before = self.snapshot();
        let line = &mut self.lines[self.cursor.row];
        if self.cursor.col >= line.len() {
            return self.invalid("r");
        }
        let end = min(self.cursor.col + count, line.len());
        for c in &mut line[self.cursor.col..end] {
            *c = replacement;
        }
        let changed = self.record_change(before);
        self.clear_parser();
        Self::executed(format!("r{replacement}"), changed, false)
    }

    fn substitute_chars(&mut self, count: usize) -> EditorEvent {
        let before = self.snapshot();
        let line = &mut self.lines[self.cursor.row];
        let end = min(self.cursor.col + count, line.len());
        let text: String = line[self.cursor.col..end].iter().collect();
        line.drain(self.cursor.col..end);
        self.register = Register {
            text,
            linewise: false,
        };
        let return_cursor = self.cursor;
        self.enter_insert_after_change(before, return_cursor);
        EditorEvent::ModeChanged {
            from: Mode::Normal,
            to: Mode::Insert,
        }
    }

    fn join_lines(&mut self, count: usize) -> EditorEvent {
        let before = self.snapshot();
        let mut joins = 0;
        let requested_joins = count.saturating_sub(1).max(1);
        for _ in 0..requested_joins {
            if self.cursor.row + 1 >= self.lines.len() {
                break;
            }
            let mut next = self.lines.remove(self.cursor.row + 1);
            while next.first().is_some_and(|c| c.is_whitespace()) {
                next.remove(0);
            }
            let needs_space = !self.lines[self.cursor.row].is_empty()
                && !next.is_empty()
                && !self.lines[self.cursor.row]
                    .last()
                    .is_some_and(|c| c.is_whitespace());
            self.cursor.col = self.lines[self.cursor.row].len().saturating_sub(1);
            if needs_space {
                self.lines[self.cursor.row].push(' ');
            }
            self.lines[self.cursor.row].extend(next);
            joins += 1;
        }
        if joins == 0 {
            return self.invalid("J");
        }
        let changed = self.record_change(before);
        self.clamp_normal_cursor();
        self.desired_col = self.cursor.col;
        Self::executed("J", changed, true)
    }

    fn direct_operator(
        &mut self,
        op: Operator,
        motion: char,
        count: usize,
        notation: &str,
    ) -> EditorEvent {
        let Some(result) = self.motion_for(motion, count) else {
            return self.invalid(notation);
        };
        self.apply_operator(op, result, notation.to_string())
    }

    fn apply_operator_motion(
        &mut self,
        op: Operator,
        motion: char,
        count: usize,
        notation: String,
    ) -> EditorEvent {
        let result = if op == Operator::Change && matches!(motion, 'w' | 'W') {
            self.motion_for(if motion == 'w' { 'e' } else { 'E' }, count)
        } else {
            self.motion_for(motion, count)
        };
        let Some(result) = result else {
            return self.invalid(notation);
        };
        self.apply_operator(op, result, notation)
    }

    fn apply_operator(&mut self, op: Operator, motion: Motion, notation: String) -> EditorEvent {
        let start = self.cursor;
        if start == motion.target && motion.kind == MotionKind::Exclusive {
            return self.invalid(notation);
        }
        if motion.kind == MotionKind::Linewise {
            let rows = min(start.row, motion.target.row)..=max(start.row, motion.target.row);
            return self.apply_line_operator_range(
                op,
                rows.start().to_owned(),
                rows.end().to_owned(),
                notation,
            );
        }
        let before = self.snapshot();
        let (range_start, range_end) = self.motion_range(start, motion.target, motion.kind);
        let text = self.extract_range(range_start, range_end);
        if text.is_empty() {
            return self.invalid(notation);
        }
        self.register = Register {
            text,
            linewise: false,
        };
        match op {
            Operator::Yank => {
                self.cursor = start;
                self.mode = Mode::Normal;
                self.visual_anchor = None;
                self.clear_parser();
                Self::executed(notation, false, false)
            }
            Operator::Delete => {
                self.delete_range(range_start, range_end);
                self.mode = Mode::Normal;
                self.visual_anchor = None;
                self.clamp_normal_cursor();
                self.desired_col = self.cursor.col;
                let changed = self.record_change(before);
                self.clear_parser();
                Self::executed(notation, changed, self.cursor != start)
            }
            Operator::Change => {
                self.delete_range(range_start, range_end);
                self.clamp_insert_cursor();
                let return_cursor = self.cursor;
                self.enter_insert_after_change(before, return_cursor);
                Self::executed(notation, true, self.cursor != start)
            }
        }
    }

    fn motion_range(&self, a: Position, b: Position, kind: MotionKind) -> (Position, Position) {
        let start = min(a, b);
        let mut end = max(a, b);
        if kind == MotionKind::Inclusive {
            end.col = min(end.col + 1, self.lines[end.row].len());
        }
        (start, end)
    }

    fn extract_range(&self, start: Position, end: Position) -> String {
        if start.row == end.row {
            return self.lines[start.row][start.col..min(end.col, self.lines[start.row].len())]
                .iter()
                .collect();
        }
        let mut result: String = self.lines[start.row][start.col..].iter().collect();
        result.push('\n');
        for row in start.row + 1..end.row {
            result.extend(self.lines[row].iter());
            result.push('\n');
        }
        result.extend(self.lines[end.row][..min(end.col, self.lines[end.row].len())].iter());
        result
    }

    fn delete_range(&mut self, start: Position, end: Position) {
        if start.row == end.row {
            let line_len = self.lines[start.row].len();
            self.lines[start.row].drain(start.col..min(end.col, line_len));
        } else {
            let suffix: Vec<char> =
                self.lines[end.row][min(end.col, self.lines[end.row].len())..].to_vec();
            self.lines[start.row].truncate(start.col);
            self.lines[start.row].extend(suffix);
            self.lines.drain(start.row + 1..=end.row);
        }
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }
        self.cursor = start;
    }

    fn apply_line_operator(&mut self, op: Operator, count: usize) -> EditorEvent {
        let end = min(
            self.cursor.row + count.saturating_sub(1),
            self.lines.len() - 1,
        );
        let notation = format!("{}{}{}", Self::display_count(count), op.key(), op.key());
        self.apply_line_operator_range(op, self.cursor.row, end, notation)
    }

    fn apply_line_operator_range(
        &mut self,
        op: Operator,
        start_row: usize,
        end_row: usize,
        notation: String,
    ) -> EditorEvent {
        let (start_row, end_row) = (
            min(start_row, end_row),
            max(start_row, end_row).min(self.lines.len() - 1),
        );
        let text = self.lines[start_row..=end_row]
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        self.register = Register {
            text,
            linewise: true,
        };
        if op == Operator::Yank {
            self.mode = Mode::Normal;
            self.visual_anchor = None;
            self.clear_parser();
            return Self::executed(notation, false, false);
        }
        let before = self.snapshot();
        self.lines.drain(start_row..=end_row);
        if op == Operator::Change {
            let insert_at = min(start_row, self.lines.len());
            self.lines.insert(insert_at, Vec::new());
            self.cursor = Position::new(insert_at, 0);
            self.enter_insert_after_change(before, self.cursor);
            Self::executed(notation, true, true)
        } else {
            if self.lines.is_empty() {
                self.lines.push(Vec::new());
            }
            self.cursor.row = min(start_row, self.lines.len() - 1);
            self.cursor.col = self.first_non_blank(self.cursor.row);
            self.mode = Mode::Normal;
            self.visual_anchor = None;
            let changed = self.record_change(before);
            self.clamp_normal_cursor();
            self.desired_col = self.cursor.col;
            self.clear_parser();
            Self::executed(notation, changed, true)
        }
    }

    fn paste(&mut self, after: bool, count: usize) -> EditorEvent {
        if self.register.text.is_empty() && !self.register.linewise {
            return self.invalid(if after { "p" } else { "P" });
        }
        let before = self.snapshot();
        for _ in 0..count {
            if self.register.linewise {
                let insert_at = if after {
                    self.cursor.row + 1
                } else {
                    self.cursor.row
                };
                let new_lines: Vec<Vec<char>> = self
                    .register
                    .text
                    .split('\n')
                    .map(|s| s.chars().collect())
                    .collect();
                let added = new_lines.len();
                self.lines.splice(insert_at..insert_at, new_lines);
                self.cursor = Position::new(
                    insert_at + added - 1,
                    self.first_non_blank(insert_at + added - 1),
                );
            } else {
                let register_text = self.register.text.clone();
                let col = if after && !self.lines[self.cursor.row].is_empty() {
                    self.cursor.col + 1
                } else {
                    self.cursor.col
                };
                self.insert_text_at(Position::new(self.cursor.row, col), &register_text);
            }
        }
        self.clamp_normal_cursor();
        self.desired_col = self.cursor.col;
        let changed = self.record_change(before);
        Self::executed(if after { "p" } else { "P" }, changed, true)
    }

    fn insert_text_at(&mut self, at: Position, text: &str) {
        let parts: Vec<Vec<char>> = text
            .split('\n')
            .map(|part| part.chars().collect())
            .collect();
        if parts.len() == 1 {
            let len = parts[0].len();
            self.lines[at.row].splice(at.col..at.col, parts[0].iter().copied());
            self.cursor = Position::new(at.row, at.col + len.saturating_sub(1));
            return;
        }
        let suffix = self.lines[at.row].split_off(at.col);
        self.lines[at.row].extend(parts[0].iter().copied());
        let mut row = at.row + 1;
        for middle in &parts[1..parts.len() - 1] {
            self.lines.insert(row, middle.clone());
            row += 1;
        }
        let mut last = parts.last().cloned().unwrap_or_default();
        let last_len = last.len();
        last.extend(suffix);
        self.lines.insert(row, last);
        self.cursor = Position::new(row, last_len.saturating_sub(1));
    }

    fn enter_visual(&mut self, mode: Mode) -> EditorEvent {
        let from = self.mode;
        self.mode = mode;
        self.visual_anchor = Some(self.cursor);
        self.clear_parser();
        EditorEvent::ModeChanged { from, to: mode }
    }

    fn handle_visual(&mut self, key: EditorKey) -> EditorEvent {
        match key {
            EditorKey::Esc => self.leave_visual(),
            EditorKey::Char('v') if self.mode == Mode::VisualChar => self.leave_visual(),
            EditorKey::Char('V') if self.mode == Mode::VisualLine => self.leave_visual(),
            EditorKey::Char('v') => {
                self.mode = Mode::VisualChar;
                EditorEvent::ModeChanged {
                    from: Mode::VisualLine,
                    to: Mode::VisualChar,
                }
            }
            EditorKey::Char('V') => {
                self.mode = Mode::VisualLine;
                EditorEvent::ModeChanged {
                    from: Mode::VisualChar,
                    to: Mode::VisualLine,
                }
            }
            EditorKey::Char(
                c @ ('h' | 'j' | 'k' | 'l' | 'w' | 'W' | 'b' | 'B' | 'e' | 'E' | '0' | '^' | '$'
                | 'G' | '{' | '}' | '%'),
            ) => {
                let before = self.cursor;
                if let Some(motion) = self.motion_for(c, 1) {
                    self.cursor = motion.target;
                    self.desired_col = self.cursor.col;
                    Self::executed(c.to_string(), false, before != self.cursor)
                } else {
                    self.invalid(c.to_string())
                }
            }
            EditorKey::Char(c @ ('d' | 'x' | 'y' | 'c')) => self.apply_visual_operator(c),
            _ => self.invalid(format!("{key:?}")),
        }
    }

    fn leave_visual(&mut self) -> EditorEvent {
        let from = self.mode;
        self.mode = Mode::Normal;
        self.visual_anchor = None;
        self.clear_parser();
        EditorEvent::ModeChanged {
            from,
            to: self.mode,
        }
    }

    fn apply_visual_operator(&mut self, key: char) -> EditorEvent {
        let op = if key == 'y' {
            Operator::Yank
        } else if key == 'c' {
            Operator::Change
        } else {
            Operator::Delete
        };
        let anchor = self.visual_anchor.unwrap_or(self.cursor);
        let notation = format!("v{key}");
        if self.mode == Mode::VisualLine {
            return self.apply_line_operator_range(
                op,
                min(anchor.row, self.cursor.row),
                max(anchor.row, self.cursor.row),
                notation,
            );
        }
        self.apply_operator(
            op,
            Motion {
                target: anchor,
                kind: MotionKind::Inclusive,
            },
            notation,
        )
    }

    fn enter_search(&mut self, forward: bool) -> EditorEvent {
        let from = self.mode;
        self.mode = Mode::Search;
        self.search_forward = forward;
        self.command_line.clear();
        self.pending_display = if forward { "/".into() } else { "?".into() };
        EditorEvent::ModeChanged {
            from,
            to: self.mode,
        }
    }

    fn handle_search_line(&mut self, key: EditorKey) -> EditorEvent {
        match key {
            EditorKey::Esc => {
                let from = self.mode;
                self.mode = Mode::Normal;
                self.command_line.clear();
                self.pending_display.clear();
                EditorEvent::ModeChanged {
                    from,
                    to: self.mode,
                }
            }
            EditorKey::Backspace => {
                self.command_line.pop();
                self.pending_display = format!(
                    "{}{}",
                    if self.search_forward { '/' } else { '?' },
                    self.command_line
                );
                EditorEvent::Pending {
                    notation: self.pending_display.clone(),
                }
            }
            EditorKey::Char(c) => {
                self.command_line.push(c);
                self.pending_display = format!(
                    "{}{}",
                    if self.search_forward { '/' } else { '?' },
                    self.command_line
                );
                EditorEvent::Pending {
                    notation: self.pending_display.clone(),
                }
            }
            EditorKey::Enter => {
                let query = self.command_line.clone();
                let notation = format!(
                    "{}{}<CR>",
                    if self.search_forward { '/' } else { '?' },
                    query
                );
                self.mode = Mode::Normal;
                self.command_line.clear();
                self.pending_display.clear();
                if query.is_empty() {
                    return self.repeat_search(false, 1);
                }
                let before = self.cursor;
                let found = self.find_query(&query, self.search_forward, self.cursor);
                self.last_search = Some((query, LastSearchDirection(self.search_forward)));
                if let Some(pos) = found {
                    self.cursor = pos;
                    self.desired_col = pos.col;
                    Self::executed(notation, false, before != pos)
                } else {
                    EditorEvent::Invalid { notation }
                }
            }
            _ => self.invalid(format!("{key:?}")),
        }
    }

    fn find_query(&self, query: &str, forward: bool, from: Position) -> Option<Position> {
        let needle: Vec<char> = query.chars().collect();
        if needle.is_empty() {
            return None;
        }
        let mut candidates = Vec::new();
        for (row, line) in self.lines.iter().enumerate() {
            if needle.len() <= line.len() {
                for col in 0..=line.len() - needle.len() {
                    if line[col..col + needle.len()] == needle {
                        candidates.push(Position::new(row, col));
                    }
                }
            }
        }
        if forward {
            candidates
                .iter()
                .copied()
                .find(|p| *p > from)
                .or_else(|| candidates.first().copied())
        } else {
            candidates
                .iter()
                .copied()
                .rev()
                .find(|p| *p < from)
                .or_else(|| candidates.last().copied())
        }
    }

    fn repeat_search(&mut self, reverse: bool, count: usize) -> EditorEvent {
        let Some((query, LastSearchDirection(direction))) = self.last_search.clone() else {
            return self.invalid(if reverse { "N" } else { "n" });
        };
        let forward = if reverse { !direction } else { direction };
        let before = self.cursor;
        let mut cursor = self.cursor;
        for _ in 0..count {
            let Some(found) = self.find_query(&query, forward, cursor) else {
                return self.invalid(if reverse { "N" } else { "n" });
            };
            cursor = found;
        }
        self.cursor = cursor;
        self.desired_col = cursor.col;
        Self::executed(if reverse { "N" } else { "n" }, false, before != cursor)
    }

    fn search_word(&mut self, forward: bool) -> EditorEvent {
        let Some(word) = self.word_under_cursor() else {
            return self.invalid(if forward { "*" } else { "#" });
        };
        let before = self.cursor;
        let found = self.find_query(&word, forward, self.cursor);
        self.last_search = Some((word, LastSearchDirection(forward)));
        if let Some(pos) = found {
            self.cursor = pos;
            self.desired_col = pos.col;
            Self::executed(if forward { "*" } else { "#" }, false, before != pos)
        } else {
            self.invalid(if forward { "*" } else { "#" })
        }
    }

    fn word_under_cursor(&self) -> Option<String> {
        let line = &self.lines[self.cursor.row];
        let c = *line.get(self.cursor.col)?;
        if !(c.is_alphanumeric() || c == '_') {
            return None;
        }
        let mut start = self.cursor.col;
        let mut end = self.cursor.col + 1;
        while start > 0 && (line[start - 1].is_alphanumeric() || line[start - 1] == '_') {
            start -= 1;
        }
        while end < line.len() && (line[end].is_alphanumeric() || line[end] == '_') {
            end += 1;
        }
        Some(line[start..end].iter().collect())
    }

    fn handle_command_line(&mut self, key: EditorKey) -> EditorEvent {
        match key {
            EditorKey::Esc => {
                let from = self.mode;
                self.mode = Mode::Normal;
                self.command_line.clear();
                self.pending_display.clear();
                EditorEvent::ModeChanged {
                    from,
                    to: self.mode,
                }
            }
            EditorKey::Backspace => {
                self.command_line.pop();
                self.pending_display = format!(":{}", self.command_line);
                EditorEvent::Pending {
                    notation: self.pending_display.clone(),
                }
            }
            EditorKey::Char(c) => {
                self.command_line.push(c);
                self.pending_display = format!(":{}", self.command_line);
                EditorEvent::Pending {
                    notation: self.pending_display.clone(),
                }
            }
            EditorKey::Enter => {
                let command = self.command_line.trim().to_string();
                let notation = format!(":{command}<CR>");
                self.mode = Mode::Normal;
                self.command_line.clear();
                self.pending_display.clear();
                if matches!(command.as_str(), "q" | "q!" | "quit" | "quit!") {
                    EditorEvent::QuitRequested
                } else {
                    EditorEvent::Invalid { notation }
                }
            }
            _ => self.invalid(format!("{key:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor(text: &[&str], row: usize, col: usize) -> Editor {
        Editor::new(
            text.iter().map(|s| (*s).to_string()).collect(),
            Position::new(row, col),
        )
    }

    fn keys(ed: &mut Editor, input: &str) {
        for c in input.chars() {
            ed.handle_key(EditorKey::Char(c), 10);
        }
    }

    #[test]
    fn unicode_columns_are_character_indices() {
        let mut ed = editor(&["á猫🙂z"], 0, 0);
        keys(&mut ed, "llx");
        assert_eq!(ed.lines_as_strings(), vec!["á猫z"]);
        assert_eq!(ed.cursor(), Position::new(0, 2));
    }

    #[test]
    fn a_appends_after_last_character() {
        let mut ed = editor(&["x"], 0, 0);
        keys(&mut ed, "a!");
        ed.handle_key(EditorKey::Esc, 10);
        assert_eq!(ed.lines_as_strings(), vec!["x!"]);
        assert_eq!(ed.cursor(), Position::new(0, 1));
    }

    #[test]
    fn failed_find_is_a_no_op_even_with_operator() {
        let mut ed = editor(&["abc"], 0, 0);
        keys(&mut ed, "dfz");
        assert_eq!(ed.lines_as_strings(), vec!["abc"]);
        assert_eq!(ed.mode(), Mode::Normal);
    }

    #[test]
    fn undo_and_redo_group_an_insert_session() {
        let mut ed = editor(&["hello"], 0, 0);
        keys(&mut ed, "iabc");
        ed.handle_key(EditorKey::Esc, 10);
        assert_eq!(ed.lines_as_strings(), vec!["abchello"]);
        keys(&mut ed, "u");
        assert_eq!(ed.lines_as_strings(), vec!["hello"]);
        ed.handle_key(EditorKey::Ctrl('r'), 10);
        assert_eq!(ed.lines_as_strings(), vec!["abchello"]);
    }

    #[test]
    fn yank_does_not_poison_parser() {
        let mut ed = editor(&["alpha.beta", "tail"], 0, 0);
        keys(&mut ed, "yyf.");
        assert_eq!(ed.cursor(), Position::new(0, 5));
        keys(&mut ed, "gg");
        assert_eq!(ed.cursor().row, 0);
        assert_eq!(ed.register(), "alpha.beta");
        assert!(ed.register_is_linewise());
    }

    #[test]
    fn visual_selection_is_inclusive() {
        let mut ed = editor(&["abcd"], 0, 0);
        keys(&mut ed, "vld");
        assert_eq!(ed.lines_as_strings(), vec!["cd"]);
        assert_eq!(ed.mode(), Mode::Normal);
        assert_eq!(ed.selection(), None);
    }

    #[test]
    fn visual_yank_returns_to_normal_without_changing_text() {
        let mut ed = editor(&["abcd"], 0, 1);
        keys(&mut ed, "vly");
        assert_eq!(ed.lines_as_strings(), vec!["abcd"]);
        assert_eq!(ed.register(), "bc");
        assert_eq!(ed.mode(), Mode::Normal);
    }

    #[test]
    fn operator_vertical_motion_is_linewise() {
        let mut ed = editor(&["one", "two", "three"], 0, 1);
        keys(&mut ed, "dj");
        assert_eq!(ed.lines_as_strings(), vec!["three"]);
    }

    #[test]
    fn count_and_word_operator_work() {
        let mut ed = editor(&["one two three four"], 0, 0);
        keys(&mut ed, "d2w");
        assert_eq!(ed.lines_as_strings(), vec!["three four"]);
    }

    #[test]
    fn search_wraps_and_n_repeats() {
        let mut ed = editor(&["Error ok Error", "Error"], 0, 0);
        keys(&mut ed, "/Error");
        ed.handle_key(EditorKey::Enter, 10);
        assert_eq!(ed.cursor(), Position::new(0, 9));
        keys(&mut ed, "n");
        assert_eq!(ed.cursor(), Position::new(1, 0));
        keys(&mut ed, "N");
        assert_eq!(ed.cursor(), Position::new(0, 9));
    }

    #[test]
    fn linewise_yank_and_paste() {
        let mut ed = editor(&["a", "b"], 0, 0);
        keys(&mut ed, "yyp");
        assert_eq!(ed.lines_as_strings(), vec!["a", "a", "b"]);
    }

    #[test]
    fn an_empty_line_can_be_yanked_and_pasted_linewise() {
        let mut ed = editor(&["", "tail"], 0, 0);
        keys(&mut ed, "yyp");
        assert_eq!(ed.lines_as_strings(), vec!["", "", "tail"]);
    }

    #[test]
    fn changing_the_last_line_keeps_line_order_and_is_one_undo_step() {
        let mut ed = editor(&["first", "old"], 1, 0);
        keys(&mut ed, "ccnew");
        ed.handle_key(EditorKey::Esc, 10);
        assert_eq!(ed.lines_as_strings(), vec!["first", "new"]);
        keys(&mut ed, "u");
        assert_eq!(ed.lines_as_strings(), vec!["first", "old"]);
    }

    #[test]
    fn text_objects_preserve_or_remove_delimiters_as_requested() {
        let mut quoted = editor(&["let value = \"debug\";"], 0, 15);
        keys(&mut quoted, "ci\"release");
        quoted.handle_key(EditorKey::Esc, 10);
        assert_eq!(quoted.lines_as_strings(), vec!["let value = \"release\";"]);

        let mut parens = editor(&["call(payload);"], 0, 7);
        keys(&mut parens, "da(");
        assert_eq!(parens.lines_as_strings(), vec!["call;"]);

        let mut brackets = editor(&["items[obsolete]"], 0, 8);
        keys(&mut brackets, "di[");
        assert_eq!(brackets.lines_as_strings(), vec!["items[]"]);
    }

    #[test]
    fn one_g_means_first_line_while_bare_g_means_last_line() {
        let mut ed = editor(&["zero", "one", "two"], 1, 0);
        keys(&mut ed, "G");
        assert_eq!(ed.cursor().row, 2);
        keys(&mut ed, "1G");
        assert_eq!(ed.cursor().row, 0);
    }

    #[test]
    fn normal_editing_commands_are_transactional() {
        let mut replace = editor(&["abcd"], 0, 1);
        keys(&mut replace, "2rX");
        assert_eq!(replace.lines_as_strings(), vec!["aXXd"]);
        keys(&mut replace, "u");
        assert_eq!(replace.lines_as_strings(), vec!["abcd"]);

        let mut join = editor(&["one", "  two", "three"], 0, 0);
        keys(&mut join, "J");
        assert_eq!(join.lines_as_strings(), vec!["one two", "three"]);
        keys(&mut join, "u");
        assert_eq!(join.lines_as_strings(), vec!["one", "  two", "three"]);
    }

    #[test]
    fn desired_column_survives_short_line() {
        let mut ed = editor(&["abcdef", "x", "abcdef"], 0, 5);
        keys(&mut ed, "jj");
        assert_eq!(ed.cursor(), Position::new(2, 5));
    }

    #[test]
    fn percent_matches_nested_brackets() {
        let mut ed = editor(&["(a[b]c)"], 0, 0);
        keys(&mut ed, "%");
        assert_eq!(ed.cursor(), Position::new(0, 6));
    }

    #[test]
    fn deterministic_key_fuzz_preserves_editor_invariants() {
        use std::panic::{AssertUnwindSafe, catch_unwind};

        fn assert_invariants(editor: &Editor, seed: u64, step: usize, key: EditorKey) {
            assert!(
                !editor.lines.is_empty(),
                "buffer vacío: seed={seed:#x}, step={step}, key={key:?}"
            );
            assert!(
                editor.cursor.row < editor.lines.len(),
                "fila inválida: seed={seed:#x}, step={step}, key={key:?}, cursor={:?}, lines={}",
                editor.cursor,
                editor.lines.len()
            );
            let line_len = editor.lines[editor.cursor.row].len();
            let max_col = if editor.mode == Mode::Insert {
                line_len
            } else {
                line_len.saturating_sub(1)
            };
            assert!(
                editor.cursor.col <= max_col,
                "columna inválida: seed={seed:#x}, step={step}, key={key:?}, mode={:?}, cursor={:?}, line_len={line_len}",
                editor.mode,
                editor.cursor
            );

            if editor.mode == Mode::Insert {
                assert!(editor.insert_session.is_some(), "Insert sin transacción");
            } else {
                assert!(
                    editor.insert_session.is_none(),
                    "transacción Insert fuera de Insert"
                );
            }

            if matches!(editor.mode, Mode::VisualChar | Mode::VisualLine) {
                let anchor = editor.visual_anchor.expect("Visual sin ancla");
                assert!(anchor.row < editor.lines.len(), "ancla fuera del buffer");
                assert!(
                    anchor.col <= editor.lines[anchor.row].len().saturating_sub(1),
                    "columna del ancla fuera de línea"
                );
            } else {
                assert!(
                    editor.visual_anchor.is_none(),
                    "ancla Visual filtrada a otro modo"
                );
            }

            for snapshot in editor.undo.iter().chain(&editor.redo) {
                assert!(!snapshot.lines.is_empty(), "snapshot con buffer vacío");
                assert!(
                    snapshot.cursor.row < snapshot.lines.len(),
                    "snapshot con fila inválida"
                );
                assert!(
                    snapshot.cursor.col
                        <= snapshot.lines[snapshot.cursor.row].len().saturating_sub(1),
                    "snapshot con columna inválida"
                );
            }
        }

        fn mark_mode(editor: &Editor, seen: &mut [bool; 6]) {
            seen[match editor.mode {
                Mode::Normal => 0,
                Mode::Insert => 1,
                Mode::VisualChar => 2,
                Mode::VisualLine => 3,
                Mode::Command => 4,
                Mode::Search => 5,
            }] = true;
        }

        fn mark_pending(editor: &Editor, seen: &mut [bool; 8]) {
            seen[match &editor.pending {
                Pending::None => 0,
                Pending::G { .. } => 1,
                Pending::Find { operator: None, .. } => 2,
                Pending::Find {
                    operator: Some(_), ..
                } => 3,
                Pending::Replace { .. } => 4,
                Pending::Operator { .. } => 5,
                Pending::OperatorG { .. } => 6,
                Pending::TextObject { .. } => 7,
            }] = true;
        }

        fn mark_event(event: &EditorEvent, seen: &mut [bool; 6]) {
            seen[match event {
                EditorEvent::Executed { .. } => 0,
                EditorEvent::Pending { .. } => 1,
                EditorEvent::Invalid { .. } => 2,
                EditorEvent::ModeChanged { .. } => 3,
                EditorEvent::QuitRequested => 4,
                EditorEvent::Viewport { .. } => 5,
            }] = true;
        }

        fn apply_observed(
            editor: &mut Editor,
            key: EditorKey,
            modes: &mut [bool; 6],
            pending: &mut [bool; 8],
            events: &mut [bool; 6],
        ) {
            let event = editor.handle_key(key, 11);
            mark_event(&event, events);
            mark_mode(editor, modes);
            mark_pending(editor, pending);
        }

        // A directed prefix guarantees coverage of states that random streams
        // reach only rarely, while the long xorshift phase explores transitions.
        let mut probe = editor(&["alpha(beta) gamma", "", "á猫🙂"], 0, 0);
        let mut seen_modes = [false; 6];
        let mut seen_pending = [false; 8];
        let mut seen_events = [false; 6];
        mark_mode(&probe, &mut seen_modes);
        mark_pending(&probe, &mut seen_pending);

        for sequence in [
            vec![EditorKey::Char('i'), EditorKey::Esc],
            vec![EditorKey::Char('v'), EditorKey::Esc],
            vec![EditorKey::Char('V'), EditorKey::Esc],
            vec![EditorKey::Char(':'), EditorKey::Esc],
            vec![EditorKey::Char('/'), EditorKey::Esc],
            vec![EditorKey::Char('g'), EditorKey::Esc],
            vec![EditorKey::Char('f'), EditorKey::Esc],
            vec![EditorKey::Char('r'), EditorKey::Esc],
            vec![EditorKey::Char('d'), EditorKey::Esc],
            vec![EditorKey::Char('d'), EditorKey::Char('g'), EditorKey::Esc],
            vec![EditorKey::Char('d'), EditorKey::Char('f'), EditorKey::Esc],
            vec![EditorKey::Char('d'), EditorKey::Char('i'), EditorKey::Esc],
            vec![EditorKey::Char('q')],
            vec![EditorKey::Ctrl('d')],
            vec![EditorKey::Char(':'), EditorKey::Char('q'), EditorKey::Enter],
        ] {
            for key in sequence {
                apply_observed(
                    &mut probe,
                    key,
                    &mut seen_modes,
                    &mut seen_pending,
                    &mut seen_events,
                );
            }
        }
        assert!(
            seen_modes.into_iter().all(|seen| seen),
            "faltó visitar un modo"
        );
        assert!(
            seen_pending.into_iter().all(|seen| seen),
            "faltó visitar un estado pending"
        );
        assert!(
            seen_events.into_iter().all(|seen| seen),
            "faltó producir una variante de EditorEvent"
        );

        const CHAR_KEYS: &[char] = &[
            'h', 'j', 'k', 'l', 'w', 'W', 'b', 'B', 'e', 'E', '0', '^', '$', 'g', '_', 'G', '{',
            '}', '%', 'f', 'F', 't', 'T', ';', ',', 'i', 'a', 'I', 'A', 'o', 'O', 'x', 'X', 'r',
            's', 'D', 'C', 'J', 'd', 'y', 'c', 'p', 'P', 'v', 'V', '/', '?', 'n', 'N', '*', '#',
            ':', 'q', '1', '2', '9', '(', ')', '[', ']', '"', ' ', 'á', '猫', '🙂', '\n', '_', '.',
        ];
        const CTRL_KEYS: &[char] = &['d', 'u', 'f', 'b', 'r', '[', 'c', 'x'];
        const SEEDS: &[u64] = &[
            0x243f_6a88_85a3_08d3,
            0x1319_8a2e_0370_7344,
            0xa409_3822_299f_31d0,
            0x082e_fa98_ec4e_6c89,
        ];

        for &seed in SEEDS {
            let mut state = seed;
            let mut editor = Editor::new(
                vec![
                    "alpha βeta (gamma)".to_string(),
                    String::new(),
                    "á猫🙂 [delta] omega".to_string(),
                ],
                Position::new(0, 0),
            );
            let mut key_kinds = [false; 7];

            for step in 0..25_000 {
                if step > 0 && step % 512 == 0 {
                    let reset_lines = match (step / 512) % 4 {
                        0 => vec![String::new()],
                        1 => vec!["ascii words".into(), "next line".into()],
                        2 => vec!["á猫🙂e\u{301}".into(), String::new()],
                        _ => vec!["({[]})".into(), String::new(), "tail".into()],
                    };
                    editor.reset(reset_lines, Position::default());
                }

                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let choice = (state as usize) % (CHAR_KEYS.len() + CTRL_KEYS.len() + 6);
                let key = if choice < CHAR_KEYS.len() {
                    key_kinds[0] = true;
                    EditorKey::Char(CHAR_KEYS[choice])
                } else if choice < CHAR_KEYS.len() + CTRL_KEYS.len() {
                    key_kinds[6] = true;
                    EditorKey::Ctrl(CTRL_KEYS[choice - CHAR_KEYS.len()])
                } else {
                    match choice - CHAR_KEYS.len() - CTRL_KEYS.len() {
                        0 => {
                            key_kinds[1] = true;
                            EditorKey::Esc
                        }
                        1 => {
                            key_kinds[2] = true;
                            EditorKey::Enter
                        }
                        2 => {
                            key_kinds[3] = true;
                            EditorKey::Backspace
                        }
                        3 => {
                            key_kinds[4] = true;
                            EditorKey::Delete
                        }
                        _ => {
                            key_kinds[5] = true;
                            EditorKey::Tab
                        }
                    }
                };

                let result = catch_unwind(AssertUnwindSafe(|| {
                    editor.handle_key(key, (state as usize) % 31)
                }));
                assert!(
                    result.is_ok(),
                    "panic: seed={seed:#x}, step={step}, key={key:?}, mode={:?}, pending={:?}, cursor={:?}",
                    editor.mode,
                    editor.pending,
                    editor.cursor
                );
                assert_invariants(&editor, seed, step, key);
            }
            assert!(
                key_kinds.into_iter().all(|seen| seen),
                "el fuzz no cubrió todas las variantes de EditorKey"
            );
        }
    }
}
