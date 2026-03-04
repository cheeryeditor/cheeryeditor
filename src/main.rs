mod app;
mod buffer;
mod editor;
mod renderer;
mod theme;

use app::App;
use editor::Editor;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut editor = Editor::new();

    for arg in &args {
        let path = std::path::Path::new(arg);
        if let Err(e) = editor.open_file(path) {
            eprintln!("Failed to open {}: {e}", path.display());
        }
    }

    App::run(editor);
}
