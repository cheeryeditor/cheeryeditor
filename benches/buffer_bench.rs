use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Buffer is not exported as a library, so we replicate the core structure
// for benchmarking. When a lib.rs is added, replace with `use cheeryeditor::Buffer`.
use ropey::Rope;

#[derive(Clone, Copy)]
struct Cursor {
    line: usize,
    col: usize,
}

struct BenchBuffer {
    text: Rope,
    cursor: Cursor,
    undo_stack: Vec<(Rope, Cursor)>,
}

impl BenchBuffer {
    fn new() -> Self {
        Self {
            text: Rope::new(),
            cursor: Cursor { line: 0, col: 0 },
            undo_stack: Vec::new(),
        }
    }

    fn from_str(s: &str) -> Self {
        Self {
            text: Rope::from_str(s),
            cursor: Cursor { line: 0, col: 0 },
            undo_stack: Vec::new(),
        }
    }

    fn insert_char(&mut self, ch: char) {
        self.undo_stack
            .push((self.text.clone(), self.cursor));
        let line_start = self.text.line_to_char(self.cursor.line);
        let idx = line_start + self.cursor.col;
        self.text.insert_char(idx, ch);
        if ch == '\n' {
            self.cursor.line += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
    }

    fn delete_char_backward(&mut self) {
        let line_start = self.text.line_to_char(self.cursor.line);
        let idx = line_start + self.cursor.col;
        if idx == 0 {
            return;
        }
        self.undo_stack
            .push((self.text.clone(), self.cursor));
        let ch = self.text.char(idx - 1);
        self.text.remove(idx - 1..idx);
        if ch == '\n' {
            self.cursor.line -= 1;
            let line_len = {
                let line = self.text.line(self.cursor.line);
                let len = line.len_chars();
                if len > 0 && line.char(len - 1) == '\n' {
                    len - 1
                } else {
                    len
                }
            };
            self.cursor.col = line_len;
        } else {
            self.cursor.col -= 1;
        }
    }

    fn undo(&mut self) {
        if let Some((text, cursor)) = self.undo_stack.pop() {
            self.text = text;
            self.cursor = cursor;
        }
    }
}

fn bench_sequential_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_insert");
    for &count in &[100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &n| {
            b.iter(|| {
                let mut buf = BenchBuffer::new();
                for _ in 0..n {
                    buf.insert_char('x');
                }
                black_box(&buf.text);
            });
        });
    }
    group.finish();
}

fn bench_insert_in_large_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_in_large_doc");
    for &line_count in &[1_000, 10_000, 100_000] {
        let content: String = (0..line_count)
            .map(|i| format!("Line {}: The quick brown fox jumps over the lazy dog.\n", i))
            .collect();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}lines", line_count)),
            &content,
            |b, content| {
                b.iter(|| {
                    let mut buf = BenchBuffer::from_str(content);
                    // Insert in the middle of the document
                    buf.cursor.line = buf.text.len_lines() / 2;
                    buf.cursor.col = 0;
                    for ch in "Hello, world!".chars() {
                        buf.insert_char(ch);
                    }
                    black_box(&buf.text);
                });
            },
        );
    }
    group.finish();
}

fn bench_undo_redo(c: &mut Criterion) {
    c.bench_function("undo_100_edits", |b| {
        b.iter(|| {
            let mut buf = BenchBuffer::new();
            for i in 0..100u8 {
                buf.insert_char((b'a' + i % 26) as char);
            }
            for _ in 0..100 {
                buf.undo();
            }
            black_box(&buf.text);
        });
    });
}

fn bench_mixed_editing(c: &mut Criterion) {
    c.bench_function("mixed_insert_delete_1000", |b| {
        b.iter(|| {
            let mut buf = BenchBuffer::new();
            for i in 0..1000 {
                if i % 3 == 0 {
                    buf.delete_char_backward();
                } else if i % 7 == 0 {
                    buf.insert_char('\n');
                } else {
                    buf.insert_char('a');
                }
            }
            black_box(&buf.text);
        });
    });
}

fn bench_rope_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_clone_for_undo");
    for &line_count in &[100, 1_000, 10_000] {
        let content: String = (0..line_count)
            .map(|i| format!("Line {}.\n", i))
            .collect();
        let rope = Rope::from_str(&content);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}lines", line_count)),
            &rope,
            |b, rope| {
                b.iter(|| {
                    black_box(rope.clone());
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_insert,
    bench_insert_in_large_document,
    bench_undo_redo,
    bench_mixed_editing,
    bench_rope_clone,
);
criterion_main!(benches);
