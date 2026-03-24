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

## 2026-03-24: Reject Direct Integration Of Spicetify Internal APIs And audioFlux

**Context**: Issue `#1` suggested `spicetify-dj-info` and `audioFlux` as possible resources for BPM/key enrichment.

**Decision**: Do not integrate either source directly into MIX_SORTER's current workflow.

**Rationale**:
- `spicetify-dj-info` depends on Spotify desktop-client internals such as `Spicetify.CosmosAsync`, `Spicetify.Platform.AuthorizationAPI`, and the internal `spclient.wg.spotify.com/audio-attributes` endpoint, which are not part of MIX_SORTER's Rust CLI OAuth flow and are known to be brittle when Spotify changes internals
- `audioFlux` is a Python/C audio-analysis stack that expects local audio arrays or audio files; MIX_SORTER currently operates on Spotify playlists plus local metadata, not on a local audio library
- Exportify-style metadata imports fit the current product much better because they preserve the local-data-first architecture without introducing an unstable client dependency or a second runtime stack

**Alternatives Considered**:
- Use Spotify internal audio-feature endpoints as a fallback source: rejected because it would make the CLI depend on unsupported desktop-only behavior
- Add optional local audio analysis via `audioFlux`: rejected because there is no local audio-file workflow in the product today and the added Python/C toolchain would be disproportionate to the current scope

**Consequences**:
- Positive: the project stays aligned with its Rust CLI architecture and local-metadata contract
- Positive: fewer hidden breakages from Spotify client changes or cross-runtime packaging issues
- Negative: users still need to source metadata externally rather than deriving it automatically from Spotify playback or local audio analysis
