# Architectural Decisions Record (ADR)

---

## 2026-03-24: Use Local Metadata As The Source Of Truth

**Context**: Spotify audio metadata that older DJ workflows depended on is no longer reliably available for this project.

**Decision**: Track enrichment will use `local_db.json` as the authoritative source for BPM and Camelot key data.

**Rationale**: This keeps the tool usable even when Spotify API capabilities change and gives the user direct control over metadata quality.

**Alternatives Considered**:
- Depend on Spotify audio feature endpoints: rejected because the workflow is already constrained by API availability
- Introduce a hosted database or external metadata provider: rejected because it adds operational complexity without solving the core personal-workflow need

**Consequences**:
- Positive: predictable metadata source, low operating cost, full local control
- Negative: database quality and completeness become the user's responsibility

## 2026-03-24: Keep MIX_SORTER As A CLI-First Tool

**Context**: The current project is implemented as a Rust command-line application and is used directly by the project owner.

**Decision**: Continue treating the CLI as the primary interface.

**Rationale**: The workflow is personal, scriptable, and already aligned with a terminal-driven toolchain.

**Alternatives Considered**:
- Build a GUI or web frontend: not justified by the current scope
- Split into a background service and thin client: premature for a single-user workflow

**Consequences**:
- Positive: lower maintenance burden, direct automation potential, faster iteration
- Negative: less approachable if the tool later expands beyond a technical user
