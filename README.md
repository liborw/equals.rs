<p align="center">
  <img src="assets/logo.png" alt="equals.rs logo" width="240" height="140">
</p>

# equals.rs

A command-line helper that evaluates annotated code inside plain text or Markdown documents. It keeps prose intact, rewrites only the marked lines (using `#=` by default), and supports Python, Numbat, and Fend out of the box.

## Features

- ✅ Evaluate inline or fenced code blocks without leaving your editor.
- ✅ Detects the language automatically from file extensions (`.py`, `.pyw`, `.nbt`, `.nb`, `.fend`, `.fd`) or via `--language`.
- ✅ Understands Markdown fences and inline backticks, so prose stays untouched.
- ✅ Uses language-specific runners: Python via `python3`, Numbat via the `numbat` CLI.
- ✅ Produces minimal diffs by updating only lines that have changed outputs.

## Requirements

- Rust 1.80+ (the crate targets Rust 2024).
- Python 3 available on your `$PATH` (for Python snippets).
- The [`numbat`](https://github.com/sharkdp/numbat) executable for Numbat code.
- The [`fend`](https://github.com/printfn/fend) CLI for Fend expressions.

## Building

```bash
cargo build
```

## Running

```bash
# Plain text, language inferred from .py extension
cargo run -- --input examples/plain_python.py

# Markdown mode, language forced to numbat
cargo run -- --markdown --language numbat --input examples/markdown_numbat.md

# Plain text Fend example (language inferred from .fend)
cargo run -- --input examples/plain_fend.fend

# Read from stdin / write to stdout
cat examples/plain_numbat.nbt | cargo run
```

### Language Selection

| Extension             | Language |
|----------------------|----------|
| `.py`, `.pyw`        | python   |
| `.nb`, `.nbt`        | numbat   |
| `.fend`, `.fd`       | fend     |

Override detection any time with `--language <name>`.

## Workflow

1. Mark the expressions you want to evaluate with `#=` (or `let a = 2; a #=` in Numbat).
2. Run `cargo run -- --input your_file`.
3. Review the updated document; only the marked lines gain new results.

Markdown parsing handles:

- fenced code blocks (e.g. ```python … ```),
- inline backtick sections (`2 + 2 #=`),
- plain prose that should remain untouched.

## Examples

See the `examples/` directory for ready-to-run demos:

- `plain_python.py` – basic Python workflow.
- `plain_numbat.nbt` – physical units with Numbat.
- `plain_fend.fend` – Fend calculator snippets.
- `markdown_python.md` – Markdown + Python.
- `markdown_numbat.md` – Markdown + Numbat.

Each example is callable exactly as shown in `examples/README.md`.

## Editor Integrations

- **Neovim** — a bundled plugin lives in [`editors/neovim`](editors/neovim/README.md). Install it straight from GitHub with a spec like `{ "liborw/equals.rs", rtp = "editors/neovim" }` (lazy.nvim example) and run `:Equals` to evaluate the current buffer with automatic language/markdown flag detection plus `#=` highlighting.

## Testing

```bash
cargo fmt --all
cargo clippy -- -D warnings
cargo test --offline
```

The test suite exercises document parsing, language evaluators, and the CLI language detection helper.

## Continuous Integration

GitHub Actions (`.github/workflows/ci.yml`) runs `fmt`, `clippy`, `test`, and produces a release build artifact on every push/PR. Tagging a version (`v*`) also publishes a GitHub Release with the prebuilt Linux binary.

## Extending

To add a new language:

1. Implement the `Language` trait in `src/lang/`.
2. Register it in `get_language_spec` (`src/lang/mod.rs`).
3. Update `guess_language_from_path` if the language should be auto-detected by extension.
4. Add sample snippets under `examples/`.

Happy evaluating!
