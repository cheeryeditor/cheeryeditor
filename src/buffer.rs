use ropey::Rope;
use std::path::PathBuf;

#[derive(Clone)]
struct Snapshot {
    text: Rope,
    cursor: Cursor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

pub struct Buffer {
    pub text: Rope,
    pub cursor: Cursor,
    pub scroll_offset: usize,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub name: String,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
}

impl Buffer {
    pub fn empty() -> Self {
        Self {
            text: Rope::new(),
            cursor: Cursor::default(),
            scroll_offset: 0,
            path: None,
            modified: false,
            name: String::from("[scratch]"),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        Ok(Self {
            text: Rope::from_str(&content),
            cursor: Cursor::default(),
            scroll_offset: 0,
            path: Some(path.to_path_buf()),
            modified: false,
            name,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| std::io::Error::other("no file path set"))?
            .clone();
        self.save_as(&path)
    }

    pub fn save_as(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        let content: String = self.text.to_string();
        std::fs::write(path, &content)?;
        self.path = Some(path.to_path_buf());
        self.name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        self.modified = false;
        Ok(())
    }

    fn save_undo(&mut self) {
        self.undo_stack.push(Snapshot {
            text: self.text.clone(),
            cursor: self.cursor,
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                text: self.text.clone(),
                cursor: self.cursor,
            });
            self.text = snap.text;
            self.cursor = snap.cursor;
            self.modified = true;
        }
    }

    pub fn redo(&mut self) {
        if let Some(snap) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                text: self.text.clone(),
                cursor: self.cursor,
            });
            self.text = snap.text;
            self.cursor = snap.cursor;
            self.modified = true;
        }
    }

    fn char_index_at_cursor(&self) -> usize {
        let line_start = self.text.line_to_char(self.cursor.line);
        let line_len = self.line_len(self.cursor.line);
        line_start + self.cursor.col.min(line_len)
    }

    pub fn line_count(&self) -> usize {
        self.text.len_lines()
    }

    pub fn line_len(&self, line: usize) -> usize {
        if line >= self.text.len_lines() {
            return 0;
        }
        let line_str = self.text.line(line);
        let len = line_str.len_chars();
        // Exclude trailing newline
        if len > 0 && line_str.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.save_undo();
        let idx = self.char_index_at_cursor();
        self.text.insert_char(idx, ch);
        if ch == '\n' {
            self.cursor.line += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
        self.modified = true;
    }

    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    pub fn delete_char_backward(&mut self) {
        let idx = self.char_index_at_cursor();
        if idx == 0 {
            return;
        }
        self.save_undo();
        let ch = self.text.char(idx - 1);
        self.text.remove(idx - 1..idx);
        if ch == '\n' {
            self.cursor.line -= 1;
            self.cursor.col = self.line_len(self.cursor.line);
        } else {
            self.cursor.col -= 1;
        }
        self.modified = true;
    }

    pub fn delete_char_forward(&mut self) {
        let idx = self.char_index_at_cursor();
        if idx >= self.text.len_chars() {
            return;
        }
        self.save_undo();
        self.text.remove(idx..idx + 1);
        self.modified = true;
    }

    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.line_len(self.cursor.line);
        }
    }

    pub fn move_right(&mut self) {
        let len = self.line_len(self.cursor.line);
        if self.cursor.col < len {
            self.cursor.col += 1;
        } else if self.cursor.line + 1 < self.line_count() {
            self.cursor.line += 1;
            self.cursor.col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.cursor.col.min(self.line_len(self.cursor.line));
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.line + 1 < self.line_count() {
            self.cursor.line += 1;
            self.cursor.col = self.cursor.col.min(self.line_len(self.cursor.line));
        }
    }

    pub fn move_home(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor.col = self.line_len(self.cursor.line);
    }

    pub fn move_to_top(&mut self) {
        self.cursor.line = 0;
        self.cursor.col = 0;
    }

    pub fn move_to_bottom(&mut self) {
        self.cursor.line = self.line_count().saturating_sub(1);
        self.cursor.col = 0;
    }

    pub fn line_text(&self, line: usize) -> String {
        if line >= self.text.len_lines() {
            return String::new();
        }
        let rope_line = self.text.line(line);
        let s: String = rope_line.to_string();
        s.trim_end_matches('\n').to_string()
    }

    pub fn ensure_cursor_visible(&mut self, viewport_height: usize) {
        if self.cursor.line < self.scroll_offset {
            self.scroll_offset = self.cursor.line;
        } else if self.cursor.line >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor.line - viewport_height + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer() {
        let buf = Buffer::empty();
        assert_eq!(buf.line_count(), 1);
        assert_eq!(buf.cursor, Cursor { line: 0, col: 0 });
        assert!(!buf.modified);
    }

    #[test]
    fn insert_and_delete() {
        let mut buf = Buffer::empty();
        buf.insert_char('h');
        buf.insert_char('i');
        assert_eq!(buf.line_text(0), "hi");
        assert!(buf.modified);
        buf.delete_char_backward();
        assert_eq!(buf.line_text(0), "h");
    }

    #[test]
    fn undo_redo() {
        let mut buf = Buffer::empty();
        buf.insert_char('a');
        buf.insert_char('b');
        assert_eq!(buf.line_text(0), "ab");
        buf.undo();
        assert_eq!(buf.line_text(0), "a");
        buf.redo();
        assert_eq!(buf.line_text(0), "ab");
    }

    #[test]
    fn newline_handling() {
        let mut buf = Buffer::empty();
        buf.insert_char('a');
        buf.insert_newline();
        buf.insert_char('b');
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_text(0), "a");
        assert_eq!(buf.line_text(1), "b");
        assert_eq!(buf.cursor, Cursor { line: 1, col: 1 });
    }

    #[test]
    fn cursor_movement() {
        let mut buf = Buffer::empty();
        buf.insert_char('a');
        buf.insert_char('b');
        buf.insert_newline();
        buf.insert_char('c');
        // At line 1, col 1
        buf.move_up();
        assert_eq!(buf.cursor.line, 0);
        assert_eq!(buf.cursor.col, 1);
        buf.move_home();
        assert_eq!(buf.cursor.col, 0);
        buf.move_end();
        assert_eq!(buf.cursor.col, 2);
        buf.move_down();
        assert_eq!(buf.cursor.line, 1);
        assert_eq!(buf.cursor.col, 1); // clamped to line length
    }

    // --- Stability / Stress Tests ---

    #[test]
    fn stress_rapid_insert_delete_cycle() {
        let mut buf = Buffer::empty();
        for i in 0..5_000 {
            if i % 3 == 0 {
                buf.delete_char_backward();
            } else if i % 7 == 0 {
                buf.insert_newline();
            } else {
                buf.insert_char((b'a' + (i % 26) as u8) as char);
            }
        }
        // Buffer should remain consistent
        assert!(buf.cursor.line < buf.line_count());
        assert!(buf.cursor.col <= buf.line_len(buf.cursor.line));
    }

    #[test]
    fn stress_undo_redo_full_cycle() {
        let mut buf = Buffer::empty();
        let edit_count = 200;
        for i in 0..edit_count {
            buf.insert_char((b'a' + (i % 26) as u8) as char);
        }
        // Undo everything
        for _ in 0..edit_count {
            buf.undo();
        }
        assert_eq!(buf.text.len_chars(), 0);
        // Redo everything
        for _ in 0..edit_count {
            buf.redo();
        }
        assert_eq!(buf.text.len_chars(), edit_count);
    }

    #[test]
    fn stress_large_document_navigation() {
        let mut buf = Buffer::empty();
        // Build a 1000-line document
        for i in 0..1_000 {
            for ch in format!("Line {}\n", i).chars() {
                buf.insert_char(ch);
            }
        }
        assert_eq!(buf.line_count(), 1001); // 1000 newlines + trailing empty line
        // Navigate to top and back to bottom
        buf.move_to_top();
        assert_eq!(buf.cursor.line, 0);
        buf.move_to_bottom();
        assert_eq!(buf.cursor.line, buf.line_count() - 1);
        // Rapid up/down movement
        for _ in 0..2_000 {
            buf.move_up();
        }
        assert_eq!(buf.cursor.line, 0);
        for _ in 0..2_000 {
            buf.move_down();
        }
        assert_eq!(buf.cursor.line, buf.line_count() - 1);
    }

    #[test]
    fn stress_cursor_bounds_after_edits() {
        let mut buf = Buffer::empty();
        // Insert multi-line content
        for ch in "Hello\nWorld\nFoo\nBar".chars() {
            buf.insert_char(ch);
        }
        // Move to a long line then delete chars to shorten it
        buf.move_to_top();
        buf.move_end(); // col = 5
        buf.move_down(); // col clamped to 5 on "World"
        assert!(buf.cursor.col <= buf.line_len(buf.cursor.line));

        // Delete everything on this line via backspace
        let len = buf.line_len(buf.cursor.line);
        for _ in 0..len {
            buf.delete_char_backward();
        }
        assert_eq!(buf.cursor.col, 0);
    }

    #[test]
    fn stress_file_roundtrip() {
        let mut buf = Buffer::empty();
        for ch in "Hello\nWorld\n".chars() {
            buf.insert_char(ch);
        }
        let dir = std::env::temp_dir();
        let path = dir.join("cheeryeditor_test_roundtrip.txt");
        buf.save_as(&path).unwrap();

        let loaded = Buffer::from_file(&path).unwrap();
        assert_eq!(loaded.text.to_string(), buf.text.to_string());
        assert!(!loaded.modified);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn stress_unicode_content() {
        let mut buf = Buffer::empty();
        let unicode_text = "你好世界\n🎉🚀\nこんにちは\nПривет\n";
        for ch in unicode_text.chars() {
            buf.insert_char(ch);
        }
        assert_eq!(buf.line_count(), 5);
        assert_eq!(buf.line_text(0), "你好世界");
        assert_eq!(buf.line_text(1), "🎉🚀");
        assert_eq!(buf.line_text(2), "こんにちは");
        assert_eq!(buf.line_text(3), "Привет");

        // Cursor movement should work with unicode
        buf.move_to_top();
        buf.move_end();
        assert_eq!(buf.cursor.col, 4); // 4 CJK chars
        buf.move_down();
        assert_eq!(buf.cursor.col, 2); // clamped to 2 emoji chars
    }

    #[test]
    fn stress_scroll_visibility() {
        let mut buf = Buffer::empty();
        for i in 0..200 {
            for ch in format!("Line {}\n", i).chars() {
                buf.insert_char(ch);
            }
        }
        let viewport = 40;
        buf.move_to_top();
        buf.ensure_cursor_visible(viewport);
        assert_eq!(buf.scroll_offset, 0);

        buf.move_to_bottom();
        buf.ensure_cursor_visible(viewport);
        assert!(buf.scroll_offset > 0);
        assert!(buf.cursor.line >= buf.scroll_offset);
        assert!(buf.cursor.line < buf.scroll_offset + viewport);
    }
}
