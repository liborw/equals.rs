# equals.rs Example Gallery

These snippets demonstrate how to drive the CLI in both plain text and Markdown modes for Python and Numbat code.

## 1. Plain Text (Python)
- Source: `examples/plain_python.py`
- Run: `cargo run -- --language python --input examples/plain_python.py`

## 2. Plain Text (Numbat)
- Source: `examples/plain_numbat.nbt`
- Run: `cargo run -- --language numbat --input examples/plain_numbat.nbt`

## 3. Markdown (Python)
- Source: `examples/markdown_python.md`
- Run: `cargo run -- --language python --markdown --input examples/markdown_python.md`

## 4. Markdown (Numbat)
- Source: `examples/markdown_numbat.md`
- Run: `cargo run -- --language numbat --markdown --input examples/markdown_numbat.md`

All commands write the evaluated document to stdout. Add `--output <file>` if you want to save the results.
