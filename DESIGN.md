# CheeryEditor — Design

## Architecture

```
main.rs          CLI entry, parse args, create Editor
   |
   v
app.rs           winit event loop, key → EditorAction mapping
   |
   v
editor.rs        Editor state machine (buffers, modes, commands)
   |
   v
buffer.rs        Rope-based text buffer, cursor, undo/redo
   |
renderer.rs      wgpu + glyphon rendering pipeline
   |
theme.rs         Color and font configuration
```

Editor and Renderer are fully decoupled. Editor has no knowledge of how it is rendered; Renderer has no knowledge of editing logic. App is the bridge: it reads state from Editor and passes it to Renderer to draw.

## Module Responsibilities

### buffer.rs

Text storage uses ropey's Rope data structure, giving O(log n) insert/delete for large files.

Undo/redo uses full snapshots (Snapshot = Rope + Cursor). The entire Rope is cloned and pushed onto the stack before each edit. This is simple and reliable for small files; for large files it can later be replaced with an incremental operation log (operation-based undo).

Cursor position uses (line, col) rather than byte offsets, so moving between lines naturally clamps to the end of the line without repeated line_to_char conversions.

### editor.rs

Editor holds a `Vec<Buffer>` and an `active` index, supporting multiple-buffer switching.

Two modes:
- **Normal** — regular editing; all keystrokes are dispatched through the `EditorAction` enum
- **Command** — bottom command-line input, supporting commands such as `q`/`w`/`wq`/`q!`/`open`/`save as`

Quit guard: `Ctrl+Q` refuses to quit and shows a prompt when there are unsaved changes; `Ctrl+Shift+Q` or `q!` is required to force quit.

### app.rs

Implements winit 0.30's `ApplicationHandler` trait. The window and rendering resources are lazily created in `resumed()` (as required by winit).

Key mapping is centralized in `map_key_to_action()`, which translates winit's `KeyEvent` into `EditorAction`. Modifier key state (Ctrl/Alt/Shift) is tracked via `ModifiersChanged` events.

Per-frame render flow:
1. Read the visible line range from Editor (based on scroll_offset and viewport_lines)
2. Build strings for the four text regions: main edit area, line-number gutter, tab bar, status bar
3. Hand off to Renderer to draw

Layout calculations are based on char_width for the monospace font, measured at startup by querying the width of an "M" character via glyphon.

### renderer.rs

Rendering pipeline: wgpu provides the GPU context; glyphon handles text shaping and rasterization.

At initialization, the JetBrains Mono font is compiled into the binary via `include_bytes!`, eliminating any runtime dependency on system fonts. The FontSystem is loaded once and reused throughout.

Per-frame flow:
1. `create_text_buffer()` — creates a glyphon Buffer for each text region, setting font, color, and width
2. `prepare()` — glyphon uploads text layout results to the GPU texture atlas
3. render pass — clears the background color, glyphon renders text
4. `trim()` — cleans up glyphs no longer needed in the texture atlas

On surface loss, the surface is reconfigured; on OutOfMemory, the editor exits immediately.

### theme.rs

Color values come in two types:
- `[f64; 4]` — for wgpu clear color (wgpu::Color requires f64)
- `[f32; 4]` — for glyphon text color

Currently there is only one hard-coded dark theme. It can later be extended to load from a configuration file.

## Key Design Decisions

### GPU Adapter Strategy

No hardware GPU required. Adapter selection strategy at startup:

1. `Backends::all()` — enable all backends, including GL
2. Try `HighPerformance` first — prefer the discrete GPU
3. On failure, try `LowPower` — accept integrated GPU or software renderers such as Mesa llvmpipe
4. On total failure, print a message and exit cleanly without panicking

This means that on servers or containers without a GPU, the editor can still run via CPU software rendering as long as Mesa llvmpipe is installed.

### Font Strategy

The font is compiled into the binary via `include_bytes!`, giving zero runtime dependencies. The trade-off is a ~270 KB increase in binary size.

glyphon's FontSystem also loads system fonts, but since rendering specifies `Family::Monospace`, the embedded JetBrains Mono is used in practice (because it is loaded first).

### Only Visible Lines Are Rendered

`draw()` extracts only the lines in the range `scroll_offset..scroll_offset+viewport_lines`, never building a render buffer for the entire file. Even for a million-line file, only a few dozen lines are processed per frame.

### Glyphon Buffer Rebuilt Each Frame

The current implementation recreates the glyphon Buffer every frame. This is the simplest approach and avoids the complexity of incremental updates. Because only visible lines are rendered (typically 30–50 lines), the performance cost is negligible.

If syntax highlighting is added later (different colors per token), incremental updates may be needed to avoid re-shaping the entire visible region every frame.

### Event-Driven Rendering

`request_redraw()` is called only after `KeyboardInput` and `Resized` events; there is no continuous refresh loop. CPU usage is zero when idle. If cursor blinking or animations are added later, a timer-driven redraw will need to be introduced.
