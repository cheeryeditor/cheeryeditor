use crate::editor::{Editor, EditorAction, Mode};
use crate::renderer::Renderer;
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Color as GlyphonColor, Family, Metrics, Shaping, TextArea,
    TextBounds,
};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowAttributes, WindowId},
};

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    editor: Editor,
    modifiers: ModifiersState,
    char_width: f32,
}

impl App {
    pub fn run(editor: Editor) {
        let event_loop = EventLoop::new().expect("failed to create event loop");
        let mut app = App {
            window: None,
            renderer: None,
            editor,
            modifiers: ModifiersState::empty(),
            char_width: 9.6,
        };
        event_loop.run_app(&mut app).expect("event loop error");
    }

    fn map_key_to_action(&self, event: &KeyEvent) -> EditorAction {
        let ctrl = self.modifiers.control_key();
        let alt = self.modifiers.alt_key();
        let shift = self.modifiers.shift_key();

        match &event.logical_key {
            Key::Character(c) if ctrl && c.as_str() == "q" => EditorAction::Quit,
            Key::Character(c) if ctrl && shift && c.as_str() == "Q" => EditorAction::ForceQuit,
            Key::Character(c) if ctrl && !shift && c.as_str() == "s" => EditorAction::Save,
            Key::Character(c) if ctrl && c.as_str() == "S" => EditorAction::SaveAs,
            Key::Character(c) if ctrl && c.as_str() == "z" => EditorAction::Undo,
            Key::Character(c) if ctrl && c.as_str() == "y" => EditorAction::Redo,
            Key::Character(c) if ctrl && c.as_str() == "p" => EditorAction::CommandPalette,
            Key::Named(NamedKey::ArrowRight) if alt => EditorAction::NextBuffer,
            Key::Named(NamedKey::ArrowLeft) if alt => EditorAction::PrevBuffer,
            Key::Named(NamedKey::Home) if ctrl => EditorAction::MoveToTop,
            Key::Named(NamedKey::End) if ctrl => EditorAction::MoveToBottom,
            Key::Named(NamedKey::ArrowLeft) => EditorAction::MoveLeft,
            Key::Named(NamedKey::ArrowRight) => EditorAction::MoveRight,
            Key::Named(NamedKey::ArrowUp) => EditorAction::MoveUp,
            Key::Named(NamedKey::ArrowDown) => EditorAction::MoveDown,
            Key::Named(NamedKey::Home) => EditorAction::Home,
            Key::Named(NamedKey::End) => EditorAction::End,
            Key::Named(NamedKey::PageUp) => EditorAction::PageUp,
            Key::Named(NamedKey::PageDown) => EditorAction::PageDown,
            Key::Named(NamedKey::Backspace) => EditorAction::Backspace,
            Key::Named(NamedKey::Delete) => EditorAction::Delete,
            Key::Named(NamedKey::Enter) => EditorAction::Newline,
            Key::Named(NamedKey::Tab) => EditorAction::InsertChar('\t'),
            Key::Character(c) if !ctrl && !alt => {
                let ch = c.chars().next().unwrap_or('\0');
                if ch != '\0' {
                    EditorAction::InsertChar(ch)
                } else {
                    EditorAction::None
                }
            }
            _ => EditorAction::None,
        }
    }

    fn handle_command_key(&mut self, event: &KeyEvent) {
        match &event.logical_key {
            Key::Named(NamedKey::Escape) => self.editor.handle_command_escape(),
            Key::Named(NamedKey::Enter) => self.editor.handle_command_enter(),
            Key::Named(NamedKey::Backspace) => self.editor.handle_command_backspace(),
            Key::Character(c) => {
                let ctrl = self.modifiers.control_key();
                let alt = self.modifiers.alt_key();
                if !ctrl && !alt {
                    if let Some(ch) = c.chars().next() {
                        self.editor.handle_command_char(ch);
                    }
                }
            }
            _ => {}
        }
    }

    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn draw(&mut self) {
        let renderer = match &mut self.renderer {
            Some(r) => r,
            None => return,
        };

        let width = renderer.width() as f32;
        let height = renderer.height() as f32;
        let theme = renderer.theme.clone();
        let line_h = theme.line_height_px();
        let char_w = self.char_width;

        // Layout
        let tab_bar_h = line_h + 8.0;
        let status_bar_h = line_h + 8.0;
        let gutter_digits = gutter_digit_count(self.editor.buf().line_count());
        let gutter_width = (gutter_digits as f32 + 2.0) * char_w;
        let text_area_top = tab_bar_h;
        let text_area_height = height - tab_bar_h - status_bar_h;
        let text_area_left = gutter_width;
        let text_area_width = width - gutter_width;

        let viewport_lines = (text_area_height / line_h).floor() as usize;
        self.editor.viewport_lines = viewport_lines;
        self.editor.buf_mut().ensure_cursor_visible(viewport_lines);

        let scroll = self.editor.buf().scroll_offset;
        let line_count = self.editor.buf().line_count();

        // Build visible text
        let mut visible_text = String::new();
        let end_line = (scroll + viewport_lines).min(line_count);
        for i in scroll..end_line {
            if i > scroll {
                visible_text.push('\n');
            }
            visible_text.push_str(&self.editor.buf().line_text(i));
        }

        // Build gutter text
        let mut gutter_text = String::new();
        for i in scroll..end_line {
            if i > scroll {
                gutter_text.push('\n');
            }
            let num = format!("{:>w$}", i + 1, w = gutter_digits);
            gutter_text.push_str(&num);
        }

        // Tab bar text
        let mut tab_text = String::new();
        for (i, buf) in self.editor.buffers.iter().enumerate() {
            let m = if buf.modified { "*" } else { "" };
            if i == self.editor.active {
                tab_text.push_str(&format!(" [{}{}] ", buf.name, m));
            } else {
                tab_text.push_str(&format!("  {}{}  ", buf.name, m));
            }
        }

        // Status bar text
        let status_left = if self.editor.mode == Mode::Command {
            format!("{}{}", self.editor.status_msg, self.editor.command_input)
        } else if !self.editor.status_msg.is_empty() {
            self.editor.status_msg.clone()
        } else {
            let path_display = self
                .editor
                .buf()
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| self.editor.buf().name.clone());
            format!(
                " {} {}",
                path_display,
                if self.editor.buf().modified { "[+]" } else { "" }
            )
        };
        let status_right = format!(
            "Ln {}, Col {}  ",
            self.editor.buf().cursor.line + 1,
            self.editor.buf().cursor.col + 1,
        );
        let status_text = format!("{status_left}    {status_right}");

        // Create glyphon buffers
        let main_buf =
            renderer.create_text_buffer(&visible_text, text_area_width, theme.fg);
        let gutter_buf =
            renderer.create_text_buffer(&gutter_text, gutter_width, theme.gutter_fg);
        let tab_buf = renderer.create_text_buffer(&tab_text, width, theme.tab_fg);
        let status_buf =
            renderer.create_text_buffer(&status_text, width, theme.status_bar_fg);

        let text_areas = vec![
            TextArea {
                buffer: &main_buf,
                left: text_area_left,
                top: text_area_top,
                scale: 1.0,
                bounds: TextBounds {
                    left: text_area_left as i32,
                    top: text_area_top as i32,
                    right: width as i32,
                    bottom: (text_area_top + text_area_height) as i32,
                },
                default_color: to_glyphon_color(theme.fg),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &gutter_buf,
                left: 4.0,
                top: text_area_top,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: text_area_top as i32,
                    right: gutter_width as i32,
                    bottom: (text_area_top + text_area_height) as i32,
                },
                default_color: to_glyphon_color(theme.gutter_fg),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &tab_buf,
                left: 0.0,
                top: 4.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: tab_bar_h as i32,
                },
                default_color: to_glyphon_color(theme.tab_fg),
                custom_glyphs: &[],
            },
            TextArea {
                buffer: &status_buf,
                left: 4.0,
                top: height - status_bar_h + 4.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: (height - status_bar_h) as i32,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: to_glyphon_color(theme.status_bar_fg),
                custom_glyphs: &[],
            },
        ];

        match renderer.render(&text_areas) {
            Ok(()) => {}
            Err(wgpu::SurfaceError::Lost) => {
                let w = renderer.width();
                let h = renderer.height();
                renderer.resize(w, h);
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                self.editor.running = false;
            }
            Err(e) => eprintln!("render error: {e}"),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("CheeryEditor")
            .with_inner_size(PhysicalSize::new(1280u32, 800u32));

        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .expect("no adapter found");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("cheeryeditor"),
                ..Default::default()
            }))
            .expect("request device failed");

        let mut renderer =
            Renderer::new(surface, &adapter, device, queue, size.width, size.height);
        self.char_width = renderer.char_width();
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size.width, size.height);
                }
                self.request_redraw();
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                if event.state == ElementState::Pressed {
                    if self.editor.mode == Mode::Command {
                        self.handle_command_key(&event);
                    } else {
                        let action = self.map_key_to_action(&event);
                        self.editor.handle_action(action);
                    }
                    if !self.editor.running {
                        event_loop.exit();
                    }
                    self.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            _ => {}
        }
    }
}

fn to_glyphon_color(c: [f32; 4]) -> GlyphonColor {
    GlyphonColor::rgba(
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
        (c[3] * 255.0) as u8,
    )
}

fn gutter_digit_count(line_count: usize) -> usize {
    if line_count == 0 {
        1
    } else {
        ((line_count as f64).log10().floor() as usize + 1).max(3)
    }
}
