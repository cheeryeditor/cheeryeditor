use crate::buffer::Buffer;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    InsertChar(char),
    Newline,
    Backspace,
    Delete,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Home,
    End,
    PageUp,
    PageDown,
    MoveToTop,
    MoveToBottom,
    Save,
    SaveAs,
    Quit,
    ForceQuit,
    Undo,
    Redo,
    NextBuffer,
    PrevBuffer,
    CommandPalette,
    None,
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
}

pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub active: usize,
    pub running: bool,
    pub mode: Mode,
    pub command_input: String,
    pub status_msg: String,
    pub viewport_lines: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffers: vec![Buffer::empty()],
            active: 0,
            running: true,
            mode: Mode::Normal,
            command_input: String::new(),
            status_msg: String::new(),
            viewport_lines: 40,
        }
    }

    pub fn open_file(&mut self, path: &std::path::Path) -> std::io::Result<()> {
        let buf = Buffer::from_file(path)?;
        if self.buffers.len() == 1 && !self.buffers[0].modified && self.buffers[0].path.is_none() {
            self.buffers[0] = buf;
        } else {
            self.buffers.push(buf);
            self.active = self.buffers.len() - 1;
        }
        Ok(())
    }

    pub fn buf(&self) -> &Buffer {
        &self.buffers[self.active]
    }

    pub fn buf_mut(&mut self) -> &mut Buffer {
        &mut self.buffers[self.active]
    }

    pub fn handle_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::InsertChar(c) => self.buf_mut().insert_char(c),
            EditorAction::Newline => self.buf_mut().insert_newline(),
            EditorAction::Backspace => self.buf_mut().delete_char_backward(),
            EditorAction::Delete => self.buf_mut().delete_char_forward(),
            EditorAction::MoveLeft => self.buf_mut().move_left(),
            EditorAction::MoveRight => self.buf_mut().move_right(),
            EditorAction::MoveUp => self.buf_mut().move_up(),
            EditorAction::MoveDown => self.buf_mut().move_down(),
            EditorAction::Home => self.buf_mut().move_home(),
            EditorAction::End => self.buf_mut().move_end(),
            EditorAction::MoveToTop => self.buf_mut().move_to_top(),
            EditorAction::MoveToBottom => self.buf_mut().move_to_bottom(),
            EditorAction::PageUp => {
                for _ in 0..self.viewport_lines {
                    self.buf_mut().move_up();
                }
            }
            EditorAction::PageDown => {
                for _ in 0..self.viewport_lines {
                    self.buf_mut().move_down();
                }
            }
            EditorAction::Save => match self.buf_mut().save() {
                Ok(()) => self.status_msg = "Saved.".into(),
                Err(e) => self.status_msg = format!("Save error: {e}"),
            },
            EditorAction::SaveAs => {
                self.mode = Mode::Command;
                self.command_input.clear();
                self.status_msg = "Save as: ".into();
            }
            EditorAction::Quit => {
                if self.buffers.iter().any(|b| b.modified) {
                    self.status_msg = "Unsaved changes. Ctrl+Shift+Q to force quit.".into();
                } else {
                    self.running = false;
                }
            }
            EditorAction::ForceQuit => {
                self.running = false;
            }
            EditorAction::Undo => self.buf_mut().undo(),
            EditorAction::Redo => self.buf_mut().redo(),
            EditorAction::NextBuffer => {
                if self.buffers.len() > 1 {
                    self.active = (self.active + 1) % self.buffers.len();
                }
            }
            EditorAction::PrevBuffer => {
                if self.buffers.len() > 1 {
                    self.active = if self.active == 0 {
                        self.buffers.len() - 1
                    } else {
                        self.active - 1
                    };
                }
            }
            EditorAction::CommandPalette => {
                self.mode = Mode::Command;
                self.command_input.clear();
                self.status_msg = "> ".into();
            }
            EditorAction::None => {}
        }
    }

    pub fn handle_command_char(&mut self, c: char) {
        self.command_input.push(c);
    }

    pub fn handle_command_backspace(&mut self) {
        self.command_input.pop();
    }

    pub fn handle_command_escape(&mut self) {
        self.mode = Mode::Normal;
        self.status_msg.clear();
    }

    pub fn handle_command_enter(&mut self) {
        let cmd = self.command_input.clone();
        self.mode = Mode::Normal;
        self.execute_command(&cmd);
    }

    fn execute_command(&mut self, cmd: &str) {
        let cmd = cmd.trim();

        if self.status_msg.starts_with("Save as: ") {
            let path = PathBuf::from(cmd);
            match self.buf_mut().save_as(&path) {
                Ok(()) => self.status_msg = format!("Saved to {}", path.display()),
                Err(e) => self.status_msg = format!("Save error: {e}"),
            }
            return;
        }

        match cmd {
            "q" | "quit" => {
                if self.buffers.iter().any(|b| b.modified) {
                    self.status_msg = "Unsaved changes. Use q! to force quit.".into();
                } else {
                    self.running = false;
                }
            }
            "q!" | "quit!" => {
                self.running = false;
            }
            "w" | "save" => match self.buf_mut().save() {
                Ok(()) => self.status_msg = "Saved.".into(),
                Err(e) => self.status_msg = format!("Save error: {e}"),
            },
            "wq" => match self.buf_mut().save() {
                Ok(()) => self.running = false,
                Err(e) => self.status_msg = format!("Save error: {e}"),
            },
            _ if cmd.starts_with("o ") || cmd.starts_with("open ") => {
                let path_str = cmd.split_once(' ').map(|x| x.1).unwrap_or("");
                let path = PathBuf::from(path_str);
                match self.open_file(&path) {
                    Ok(()) => self.status_msg = format!("Opened {}", path.display()),
                    Err(e) => self.status_msg = format!("Open error: {e}"),
                }
            }
            _ => {
                self.status_msg = format!("Unknown command: {cmd}");
            }
        }
    }
}
