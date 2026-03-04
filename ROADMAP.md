# CheeryEditor — Roadmap

## Phase 1 — Core Editor

- [x] Rope-based buffer model (ropey) with undo/redo
- [x] Native window with GPU text rendering (wgpu + glyphon + winit)
- [x] Basic editing: insert, delete, cursor movement, page up/down
- [x] File I/O: open, save, save-as
- [x] Multi-buffer tab bar with Alt+Left/Right switching
- [x] Status bar with file path, modified indicator, cursor position
- [x] Command palette (Ctrl+P) with quit/save/open commands
- [x] Bundled monospace font (JetBrains Mono)
- [x] Unit tests for buffer operations
- [x] Stability/stress tests (unicode, large docs, cursor bounds, file roundtrip)
- [x] Performance benchmarks (criterion: insert, undo/redo, large doc editing, rope clone)
- [x] CI setup — GitHub Actions matrix (Linux, macOS, Windows)
- [ ] Text selection and clipboard

## Phase 2 — Syntax & Highlighting

- [ ] Tree-sitter integration for incremental parsing
- [ ] Syntax highlighting (Rust, Python, JS/TS, Go, C/C++, Markdown, TOML, JSON, YAML)
- [ ] Bracket matching and auto-indent
- [ ] Word wrap and soft-wrap modes

## Phase 3 — Navigation & Search

- [ ] Fuzzy file finder
- [ ] Project-wide text search (ripgrep-backed)
- [ ] Tree-sitter go-to-definition and find-references (local)
- [ ] Symbol outline / breadcrumb bar
- [ ] Jump list and marks

## Phase 4 — Git Integration

- [ ] Git status sidebar
- [ ] Inline diff and hunk staging/unstaging
- [ ] Blame annotations
- [ ] Commit, amend, and log viewer
- [ ] Merge conflict resolver (3-way diff)

## Phase 5 — Markdown & Prose

- [ ] Live Markdown preview
- [ ] Frontmatter parsing and highlighting
- [ ] Spell check
- [ ] Focus / zen mode

## Phase 6 — AI Agent Panel

- [ ] Attach to CLI agent process (stdin/stdout streaming)
- [ ] Agent output pane with ANSI rendering
- [ ] File-change review UI (accept/reject/edit per-hunk)
- [ ] Session history and bookmarking

## Phase 7 — Extensibility & Polish

- [ ] LSP client (completions, diagnostics, hover, rename)
- [ ] Vim and Emacs keybinding profiles
- [ ] Plugin API (Lua or WASM)
- [ ] Theming (base16, user-defined color schemes)
- [ ] Cross-platform packaging and release automation
