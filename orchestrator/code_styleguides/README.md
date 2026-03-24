# Code Style Guides

The bundled Orchestrator templates in this repository cover JavaScript, TypeScript, Python, and Go.

This project is written in Rust, so there was no matching template to copy directly.

For now, Rust coding standards are defined in `orchestrator/workflow.md`:

- format with `cargo fmt`
- lint with `cargo clippy --all-targets --all-features`
- prefer small, testable helper functions
- add regression tests for matching and sorting logic
