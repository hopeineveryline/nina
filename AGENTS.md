# AGENTS.md

## Nina repo guidance

- Do **not** use Oracle for this repository. It consistently hangs and slows work down.
- For verification and review, prefer local checks plus relevant gstack skills instead.
- Default verification stack for code changes:
  - `lsp_diagnostics` on touched files
  - targeted tests, then full `cargo test` when Rust behavior changes
  - `cargo check`
  - `cargo build`
- When extra review is needed, use gstack skills before any external long-running reviewer.
