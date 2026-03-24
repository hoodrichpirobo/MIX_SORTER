# Development Workflow

## Working Style

- Prefer small, testable changes over broad rewrites
- Keep the CLI behavior deterministic and observable
- Preserve the local-data-first architecture unless there is a deliberate product decision to change it

## Quality Gates

- New behavior should be covered by focused tests when the code structure allows it
- For logic-heavy changes, prefer extracting testable functions over expanding `main`
- Before shipping changes, run:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features`
  - `cargo test`
- If a command cannot be run, record that explicitly in the session or track notes

## Test Strategy

- Target practical coverage around matching, normalization, Camelot conversion, and sorting rules
- Add regression tests for edge cases discovered in real playlists or local database mismatches
- Favor unit tests for pure logic and integration-style checks only where they add meaningful confidence

## Commit Conventions

- Commit after a coherent task or fix is complete
- Use descriptive messages with a scoped prefix when useful, for example:
  - `matcher: tighten artist filtering`
  - `sort: fix Camelot ordering regression`
  - `orchestrator(setup): initialize project docs`

## Implementation Protocol

- For work tracked under Orchestrator, maintain `spec.md` as the contract and `plan.md` as the execution checklist
- Mark tasks as `[~]` when in progress and `[x]` when complete
- Pause at the end of each phase for manual verification before moving on
- Record meaningful architecture or product decisions in `orchestrator/decisions.md`

## Rust-Specific Conventions

- Use `rustfmt` formatting defaults unless the project adopts a custom config
- Treat Clippy warnings as signals to address, not noise to ignore by default
- Prefer explicit, testable helper functions for matching and sorting logic
- Keep error handling user-facing at the CLI boundary and structured internally

## Manual Verification Checklist

- Authentication succeeds with the configured Spotify app
- Playlist parsing works for the supplied ID or URL
- Enrichment reports accurate match and miss counts
- Sorted output updates the intended playlist without losing tracks
- Unmatched tracks can be investigated from the terminal output
