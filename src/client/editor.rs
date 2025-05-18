// rsql.rs - Nathanael "NateNateNate" Thevarajah
// <natenatenat3@protonmail.com> - Refer to the license for more
// information.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ropey::Rope;

#[derive(Debug)]
pub enum EditorCommand {
    Newline,
    Backspace,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveBeginningLine,
    MoveEndLine,
    Undo,
    Redo,
    SearchMode,
    Quit,
    InsertChar(char),
}

/// The cursor represented as x and y.
pub type Cursor = (usize, usize);

/// The scroll offset represented as x and y.
pub type ScrollOffset = (usize, usize);

#[derive(Clone, Debug)]
enum EditKind {
    Insert { index: usize, text: String },
    Delete { index: usize, text: String },
}

#[derive(Clone, Debug)]
struct Edit {
    kind: EditKind,
    before_cursor: Cursor,
    after_cursor: Cursor,
}

#[derive(Clone, Debug)]
struct EditBatch {
    edits: Vec<Edit>,
    timestamp: Instant,
}

impl EditBatch {
    fn new() -> Self {
        Self {
            edits: vec![],
            timestamp: Instant::now(),
        }
    }

    fn push(&mut self, edit: Edit) {
        self.edits.push(edit);
        self.timestamp = Instant::now();
    }

    fn is_stale(&self) -> bool {
        self.timestamp.elapsed() > Duration::from_millis(1000)
    }
}

/// Our editor state.
pub struct EditorState {
    pub buffer: Rope,
    pub cursor: Cursor,
    pub scroll: ScrollOffset,
    undo_stack: Vec<EditBatch>,
    redo_stack: Vec<EditBatch>,
    current_batch: EditBatch,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            buffer: Rope::new(),
            cursor: (0, 0),
            scroll: (0, 0),
            undo_stack: vec![],
            redo_stack: vec![],
            current_batch: EditBatch::new(),
        }
    }
}

/// Handles key mappings ONLY.
pub fn map_key_event_to_command(key: KeyEvent) -> Option<EditorCommand> {
    match key.code {
        KeyCode::Enter => Some(EditorCommand::Newline),
        KeyCode::Backspace => Some(EditorCommand::Backspace),
        KeyCode::Left => Some(EditorCommand::MoveLeft),
        KeyCode::Right => Some(EditorCommand::MoveRight),
        KeyCode::Up => Some(EditorCommand::MoveUp),
        KeyCode::Down => Some(EditorCommand::MoveDown),

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::Undo)
        }
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::Redo)
        }
        KeyCode::Char('/') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::SearchMode)
        }
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::Quit)
        }

        // Emacs-styled movement bindings
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::MoveUp)
        }
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::MoveDown)
        }
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::MoveLeft)
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::MoveRight)
        }
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::Newline)
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::MoveBeginningLine)
        }
        KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(EditorCommand::MoveEndLine)
        }

        // Anything else can be considered text the user wants to pass
        // to the editor.
        KeyCode::Char(c) if key.modifiers.is_empty() => Some(EditorCommand::InsertChar(c)),
        _ => None,
    }
}

/// Handles all things related to mutating the buffer and shii.
impl EditorState {
    pub fn handle_buffer(&mut self, cmd: EditorCommand) {
        let (x, y) = self.cursor;
        let rope_index = self.buffer.line_to_char(y) + x;

        match cmd {
            EditorCommand::InsertChar(c) => {
                self.buffer.insert_char(rope_index, c);
                let edit = Edit {
                    kind: EditKind::Insert {
                        index: rope_index,
                        text: c.to_string(),
                    },
                    before_cursor: (x, y),
                    after_cursor: (x + 1, y),
                };
                self.record_edit(edit);
                self.cursor.0 += 1;
            }
            EditorCommand::Newline => {
                self.buffer.insert_char(rope_index, '\n');
                let edit = Edit {
                    kind: EditKind::Insert {
                        index: rope_index,
                        text: "\n".to_string(),
                    },
                    before_cursor: (x, y),
                    after_cursor: (0, y + 1),
                };
                self.record_edit(edit);
                self.cursor = (0, y + 1);
            }
            EditorCommand::Backspace => {
                if rope_index > 0 {
                    let deleted = self.buffer.char(rope_index - 1);
                    self.buffer.remove(rope_index - 1..rope_index);
                    let edit = Edit {
                        kind: EditKind::Delete {
                            index: rope_index - 1,
                            text: deleted.to_string(),
                        },
                        before_cursor: self.cursor,
                        after_cursor: (x.saturating_sub(1), y),
                    };
                    self.record_edit(edit);
                    self.cursor.0 = self.cursor.0.saturating_sub(1);
                }
            }
            EditorCommand::Undo => {
                self.undo();
            }
            EditorCommand::Redo => {
                self.redo();
            }
            EditorCommand::MoveLeft => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                }
            }
            EditorCommand::MoveRight => {
                let line_len = self.buffer.line(self.cursor.1).len_chars();
                if self.cursor.0 < line_len {
                    self.cursor.0 += 1;
                }
            }
            EditorCommand::MoveUp => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                    let new_line_len = self.buffer.line(self.cursor.1).len_chars();
                    self.cursor.0 = self.cursor.0.min(new_line_len);
                }
            }
            EditorCommand::MoveDown => {
                if self.cursor.1 + 1 < self.buffer.len_lines() {
                    self.cursor.1 += 1;
                    let new_line_len = self.buffer.line(self.cursor.1).len_chars();
                    self.cursor.0 = self.cursor.0.min(new_line_len);
                }
            }
            EditorCommand::MoveBeginningLine => self.cursor.0 = 0,
            EditorCommand::MoveEndLine => {
                let line_len = self.buffer.line(self.cursor.1).len_chars();
                if self.cursor.1 < self.buffer.len_lines() {
                    self.cursor.0 = line_len;
                }
            }
            _ => {}
        }
    }

    fn record_edit(&mut self, edit: Edit) {
        if self.current_batch.is_stale() {
            if !self.current_batch.edits.is_empty() {
                self.undo_stack.push(self.current_batch.clone());
            }
            self.current_batch = EditBatch::new();
        }

        self.current_batch.push(edit);
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if !self.current_batch.edits.is_empty() {
            self.undo_stack.push(self.current_batch.clone());
            self.current_batch = EditBatch::new();
        }

        if let Some(batch) = self.undo_stack.pop() {
            for edit in batch.edits.iter().rev() {
                match &edit.kind {
                    EditKind::Insert { index, text } => {
                        self.buffer.remove(*index..*index + text.chars().count());
                        self.cursor = edit.before_cursor;
                    }
                    EditKind::Delete { index, text } => {
                        self.buffer.insert(*index, text);
                        self.cursor = edit.before_cursor;
                    }
                }
            }
            self.redo_stack.push(batch);
        }
    }

    fn redo(&mut self) {
        if let Some(batch) = self.redo_stack.pop() {
            for edit in &batch.edits {
                match &edit.kind {
                    EditKind::Insert { index, text } => {
                        self.buffer.insert(*index, text);
                        self.cursor = edit.after_cursor;
                    }
                    EditKind::Delete { index, text } => {
                        self.buffer.remove(*index..*index + text.chars().count());
                        self.cursor = edit.after_cursor;
                    }
                }
            }
            self.undo_stack.push(batch);
        }
    }
}
