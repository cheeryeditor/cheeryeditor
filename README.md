# CheeryEditor

A lightweight editor for code, Markdown, and plain text.

> **Status:** Early development. Not ready for production use.

---

## Motivation

Most editors are either too heavy for everyday text editing, or too simple for serious work. CheeryEditor aims for the middle ground — fast and minimal, but with real tools: solid Git integration, code navigation, and natural support for AI agent workflows.

It covers the full range of text-based editing: source code, documentation, notes, configuration files, prose. No rich text, no formatting toolbars — just a fast, focused editing experience for anything that lives in a plain file.

---

## Planned Features

- Fast startup, low memory footprint, single binary
- Markdown editing with live preview and frontmatter support
- Syntax highlighting for code and common file formats
- Lightweight code navigation via Tree-sitter (go to definition, find references)
- First-class Git UI — hunk staging, blame, diff, history, merge resolver
- Agent panel — attach to a CLI agent, stream output, review file changes
- Optional LSP support
- Vim / Emacs keybinding profiles

---

## Getting Started

No releases yet. Build from source:

```bash
git clone https://github.com/cheeryeditor/cheeryeditor.git
cd cheeryeditor
cargo build --release
./target/release/cheeryeditor .
```

---

## Contributing

Open an issue before sending a PR — core design is still being worked out.

---

## License

MIT
