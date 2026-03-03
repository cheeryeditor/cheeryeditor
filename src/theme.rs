#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: [f64; 4],
    pub fg: [f32; 4],
    pub gutter_fg: [f32; 4],
    pub gutter_bg: [f64; 4],
    pub cursor_color: [f32; 4],
    pub selection_bg: [f32; 4],
    pub status_bar_bg: [f64; 4],
    pub status_bar_fg: [f32; 4],
    pub tab_bar_bg: [f64; 4],
    pub tab_active_bg: [f64; 4],
    pub tab_fg: [f32; 4],
    pub font_size: f32,
    pub line_height: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: [0.15, 0.15, 0.18, 1.0],
            fg: [0.85, 0.85, 0.85, 1.0],
            gutter_fg: [0.45, 0.45, 0.50, 1.0],
            gutter_bg: [0.13, 0.13, 0.16, 1.0],
            cursor_color: [0.9, 0.9, 0.9, 0.9],
            selection_bg: [0.25, 0.40, 0.65, 0.5],
            status_bar_bg: [0.10, 0.10, 0.13, 1.0],
            status_bar_fg: [0.70, 0.70, 0.70, 1.0],
            tab_bar_bg: [0.12, 0.12, 0.15, 1.0],
            tab_active_bg: [0.15, 0.15, 0.18, 1.0],
            tab_fg: [0.70, 0.70, 0.70, 1.0],
            font_size: 16.0,
            line_height: 1.4,
        }
    }
}

impl Theme {
    pub fn line_height_px(&self) -> f32 {
        self.font_size * self.line_height
    }
}
