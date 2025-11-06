# equals.rs Agent Handbook

## Quick Start
- Build the CLI: `cargo build` (append `--release` for optimized binaries).
- Run the tool: `cargo run -- [flags]` (supports `--input`, `--output`, `--language`, `--markdown`).
- Execute tests: `cargo test`.
- Lint before submitting: `cargo clippy -- -D warnings`.
- Format all code: `cargo fmt` (use `cargo fmt --check` in CI-style validation).

## Repository Map
- `src/main.rs` — CLI entry point; parses flags with `clap` and wires parsing/evaluation pipeline.
- `src/parser.rs` & `src/markdown.rs` — implement the plain-text and Markdown document parsers.
- `src/document.rs` — document model with block evaluation utilities.
- `src/lang/mod.rs` & friends — language abstraction; currently only `python` is implemented.
- `Cargo.toml` — Rust 2024 crate; keep dependency list minimal.

## Development Workflow
- Keep the tree clean: fmt → clippy → test sequence before sending changes.
- Add unit tests beside the code they cover; the `lang` module already has good test examples.
- Prefer small, composable helpers when functions grow beyond ~40 lines.
- When adding a new language, implement the `Language` trait and register it in `get_language_spec`.
- Propagate errors with `Result`/`Option` and the `?` operator; avoid `unwrap`/`expect` outside tests.

## Code Style Notes
- Project uses the repo `rustfmt` config (120 character width); always run `cargo fmt`.
- Stick to idiomatic Rust naming: snake_case for functions/vars, CamelCase for types.
- Group imports by std, third-party, then local modules; keep `use` blocks sorted within each group.
- Document public structs/functions with rustdoc comments.
- Prefer readable, straight-line Rust; favor iterators and pattern matching over manual indexing.
- Capture tricky logic with short comments so the next agent can follow along quickly.

## Testing & Verification
- `cargo test` runs fast; use it after meaningful changes, especially in parsing/evaluation logic.
- For manual smoke tests, pipe sample markdown through the CLI: \
  `cat examples/sample.md | cargo run -- --markdown`.
- If you add fixtures, keep them under `tests/` or `examples/` and reference them from tests/README.
